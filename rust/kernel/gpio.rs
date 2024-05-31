// SPDX-License-Identifier: GPL-2.0

//! Support for gpio device drivers.
//!
//! C header: [`include/linux/gpio/driver.h`](../../../../include/linux/gpio/driver.h)

use crate::{
    bindings, device, sync::LockClassKey,
    error::{code::*, Error, Result, from_result},
    types::ForeignOwnable, 
};
use core::{
    cell::UnsafeCell,
    marker::{PhantomData, PhantomPinned},
    pin::Pin,
};
use macros::vtable;

/// The direction of a gpio line.
pub enum LineDirection {
    /// Direction is input.
    In = bindings::GPIO_LINE_DIRECTION_IN as _,

    /// Direction is output.
    Out = bindings::GPIO_LINE_DIRECTION_OUT as _,
}

/// A gpio chip.
#[vtable]
pub trait Chip {
    /// Context data associated with the gpio chip.
    ///
    /// It determines the type of the context data passed to each of the methods of the trait.
    type Data: ForeignOwnable + Sync + Send;

    /// Returns the direction of the given gpio line.
    fn get_direction(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _offset: u32,
    ) -> Result<LineDirection> {
        Err(ENOTSUPP)
    }

    /// Configures the direction as input of the given gpio line.
    fn direction_input(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _offset: u32,
    ) -> Result {
        Err(EIO)
    }

    /// Configures the direction as output of the given gpio line.
    ///
    /// The value that will be initially output is also specified.
    fn direction_output(
        _data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
        _offset: u32,
        _value: bool,
    ) -> Result {
        Err(ENOTSUPP)
    }

    /// Returns the current value of the given gpio line.
    fn get(_data: <Self::Data as ForeignOwnable>::Borrowed<'_>, _offset: u32) -> Result<bool> {
        Err(EIO)
    }

    /// Sets the value of the given gpio line.
    fn set(_data: <Self::Data as ForeignOwnable>::Borrowed<'_>, _offset: u32, _value: bool) {}
}

/// A registration of a gpio chip.
///
/// # Examples
///
/// The following example registers an empty gpio chip.
///
/// ```
/// # use kernel::prelude::*;
/// use kernel::{
///     device::RawDevice,
///     gpio::{self, Registration},
/// };
///
/// struct MyGpioChip;
/// #[vtable]
/// impl gpio::Chip for MyGpioChip {
///     type Data = ();
/// }
///
/// fn example(parent: &dyn RawDevice) -> Result<Pin<Box<Registration<MyGpioChip>>>> {
///     let mut r = Pin::from(Box::try_new(Registration::new())?);
///     kernel::gpio_chip_register!(r.as_mut(), 32, None, parent, ())?;
///     Ok(r)
/// }
/// ```
pub struct Registration<T: Chip> {
    gc: UnsafeCell<bindings::gpio_chip>,
    parent: Option<device::Device>,
    _p: PhantomData<T>,
    _pin: PhantomPinned,
}

impl<T: Chip> Registration<T> {
    /// Creates a new [`Registration`] but does not register it yet.
    ///
    /// It is allowed to move.
    pub fn new() -> Self {
        Self {
            parent: None,
            gc: UnsafeCell::new(bindings::gpio_chip::default()),
            _pin: PhantomPinned,
            _p: PhantomData,
        }
    }

    /// Registers a gpio chip with the rest of the kernel.
    ///
    /// Users are encouraged to use the [`gpio_chip_register`] macro because it automatically
    /// defines the lock classes and calls the registration function.
    ///
    /// [`gpio_chip_register`]: crate::gpio_chip_register
    pub fn register(
        self: Pin<&mut Self>,
        gpio_count: u16,
        base: Option<i32>,
        parent: &dyn device::RawDevice,
        data: T::Data,
        lock_keys: [&'static LockClassKey; 2],
    ) -> Result {
        if self.parent.is_some() {
            // Already registered.
            return Err(EINVAL);
        }

        // SAFETY: We never move out of `this`.
        let this = unsafe { self.get_unchecked_mut() };
        {
            let gc = this.gc.get_mut();

            // Set up the callbacks.
            gc.request = Some(bindings::gpiochip_generic_request);
            gc.free = Some(bindings::gpiochip_generic_free);
            if T::HAS_GET_DIRECTION {
                gc.get_direction = Some(get_direction_callback::<T>);
            }
            if T::HAS_DIRECTION_INPUT {
                gc.direction_input = Some(direction_input_callback::<T>);
            }
            if T::HAS_DIRECTION_OUTPUT {
                gc.direction_output = Some(direction_output_callback::<T>);
            }
            if T::HAS_GET {
                gc.get = Some(get_callback::<T>);
            }
            if T::HAS_SET {
                gc.set = Some(set_callback::<T>);
            }

            // When a base is not explicitly given, use -1 for one to be picked.
            if let Some(b) = base {
                gc.base = b;
            } else {
                gc.base = -1;
            }

            gc.ngpio = gpio_count;
            gc.parent = parent.raw_device();
            gc.label = parent.name().as_char_ptr();

            // TODO: Define `gc.owner` as well.
        }

        let data_pointer = <T::Data as ForeignOwnable>::into_foreign(data);
        // SAFETY: `gc` was initilised above, so it is valid.
        let ret = unsafe {
            bindings::gpiochip_add_data_with_key(
                this.gc.get(),
                data_pointer as _,
                lock_keys[0].as_ptr(),
                lock_keys[1].as_ptr(),
            )
        };
        if ret < 0 {
            // SAFETY: `data_pointer` was returned by `into_foreign` above.
            unsafe { T::Data::from_foreign(data_pointer) };
            return Err(Error::from_errno(ret));
        }

        this.parent = Some(device::Device::from_dev(parent));
        Ok(())
    }
}

// SAFETY: `Registration` doesn't offer any methods or access to fields when shared between threads
// or CPUs, so it is safe to share it.
unsafe impl<T: Chip> Sync for Registration<T> {}

// SAFETY: Registration with and unregistration from the gpio subsystem can happen from any thread.
// Additionally, `T::Data` (which is dropped during unregistration) is `Send`, so it is ok to move
// `Registration` to different threads.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: Chip> Send for Registration<T> {}

impl<T: Chip> Default for Registration<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Chip> Drop for Registration<T> {
    /// Removes the registration from the kernel if it has completed successfully before.
    fn drop(&mut self) {
        if self.parent.is_some() {
            // Get a pointer to the data stored in chip before destroying it.
            // SAFETY: `gc` was during registration, which is guaranteed to have succeeded (because
            // `parent` is `Some(_)`, so it remains valid.
            let data_pointer = unsafe { bindings::gpiochip_get_data(self.gc.get()) };

            // SAFETY: By the same argument above, `gc` is still valid.
            unsafe { bindings::gpiochip_remove(self.gc.get()) };

            // Free data as well.
            // SAFETY: `data_pointer` was returned by `into_foreign` during registration.
            unsafe { <T::Data as ForeignOwnable>::from_foreign(data_pointer) };
        }
    }
}

/// Registers a gpio chip with the rest of the kernel.
///
/// It automatically defines the required lock classes.
#[macro_export]
macro_rules! gpio_chip_register {
    ($reg:expr, $count:expr, $base:expr, $parent:expr, $data:expr $(,)?) => {{
        static CLASS1: $crate::sync::LockClassKey = $crate::static_lock_class!();
        static CLASS2: $crate::sync::LockClassKey = $crate::static_lock_class!();
        $crate::gpio::Registration::register(
            $reg,
            $count,
            $base,
            $parent,
            $data,
            [&CLASS1, &CLASS2],
        )
    }};
}

unsafe extern "C" fn get_direction_callback<T: Chip>(
    gc: *mut bindings::gpio_chip,
    offset: core::ffi::c_uint,
) -> core::ffi::c_int {
    from_result(||{
        // SAFETY: The value stored as chip data was returned by `into_foreign` during registration.
        let data = unsafe { T::Data::borrow(bindings::gpiochip_get_data(gc)) };
        Ok(T::get_direction(data, offset)? as i32)
    })
}

unsafe extern "C" fn direction_input_callback<T: Chip>(
    gc: *mut bindings::gpio_chip,
    offset: core::ffi::c_uint,
) -> core::ffi::c_int {
    from_result(||{
        // SAFETY: The value stored as chip data was returned by `into_foreign` during registration.
        let data = unsafe { T::Data::borrow(bindings::gpiochip_get_data(gc)) };
        T::direction_input(data, offset)?;
        Ok(0)
    })
}

unsafe extern "C" fn direction_output_callback<T: Chip>(
    gc: *mut bindings::gpio_chip,
    offset: core::ffi::c_uint,
    value: core::ffi::c_int,
) -> core::ffi::c_int {
    from_result(||{
        // SAFETY: The value stored as chip data was returned by `into_foreign` during registration.
        let data = unsafe { T::Data::borrow(bindings::gpiochip_get_data(gc)) };
        T::direction_output(data, offset, value != 0)?;
        Ok(0)
    })
}

unsafe extern "C" fn get_callback<T: Chip>(
    gc: *mut bindings::gpio_chip,
    offset: core::ffi::c_uint,
) -> core::ffi::c_int {
    from_result(||{
        // SAFETY: The value stored as chip data was returned by `into_foreign` during registration.
        let data = unsafe { T::Data::borrow(bindings::gpiochip_get_data(gc)) };
        let v = T::get(data, offset)?;
        Ok(v as _)
    })
}

unsafe extern "C" fn set_callback<T: Chip>(
    gc: *mut bindings::gpio_chip,
    offset: core::ffi::c_uint,
    value: core::ffi::c_int,
) {
    // SAFETY: The value stored as chip data was returned by `into_foreign` during registration.
    let data = unsafe { T::Data::borrow(bindings::gpiochip_get_data(gc)) };
    T::set(data, offset, value != 0);
}

