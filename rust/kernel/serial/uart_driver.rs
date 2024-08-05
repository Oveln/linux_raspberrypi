// SPDX-License-Identifier: GPL-2.0

//! Support for uart driver.
//!
//! C header: ['include/linux/serial_core.h']

use super::uart_console::Console;

use core::mem::MaybeUninit;

use crate::{
    bindings,
    error::{to_result, Result},
    pr_info,
    str::CStr,
    ThisModule,
};

/// Wraps the kernel's `struct uart_driver`.
///
/// # Invariants
///
/// The pointer is non-null and valid, and has a non-zero reference count.
#[repr(transparent)]
pub struct UartDriver(bindings::uart_driver);

impl UartDriver {
    /// Create a new [`UartDriver`]
    pub const fn new(
        owner: &'static ThisModule,
        driver_name: &'static CStr,
        dev_name: &'static CStr,
        reg: &Console,
    ) -> Self {
        // SAFETY: `console` is a C structure holding data that has been initialized with 0s,
        // hence it is safe to use as-is.
        let mut uart_driver =
            unsafe { MaybeUninit::<bindings::uart_driver>::zeroed().assume_init() };
        uart_driver.owner = owner.as_ptr();
        uart_driver.driver_name = driver_name.as_char_ptr();
        uart_driver.dev_name = dev_name.as_char_ptr();
        uart_driver.cons = reg.as_ptr();
        // uart_driver.state = null_mut();
        // uart_driver.tty_driver = null_mut();
        Self(uart_driver)
    }

    /// Setup the console other config
    pub const fn with_config(mut self, major: i32, minor: i32, nr: i32) -> Self {
        self.0.major = major;
        self.0.minor = minor;
        self.0.nr = nr;
        self
    }

    /// Creates a reference to a [`UartDrive`] from a valid pointer.
    ///
    /// # Safety
    ///
    /// Callers must ensure that `ptr` is valid, non-null, and has a non-zero reference count for
    /// the entire duration when the returned reference exists.
    pub unsafe fn from_raw<'a>(ptr: *mut bindings::uart_driver) -> &'a Self {
        // SAFETY: Guaranteed by the safety requirements of the function.
        unsafe { &*ptr.cast() }
    }

    /// Returns a raw pointer to the inner C struct.
    #[inline]
    pub const fn as_ptr(&self) -> *mut bindings::uart_driver {
        &self.0 as *const _ as *mut _
    }

    /// Use `uart_unregister_driver` to registers a driver with the uart core layer.
    ///
    /// Register a uart driver with the core driver. We in turn register with thetty layer,
    /// and initialise the core driver per-port state.
    pub fn register(&self) -> Result {
        unsafe { to_result(bindings::uart_register_driver(self.as_ptr())) }
    }

    pub fn is_registered(&self) -> bool {
        !self.0.state.is_null()
    }
}

impl Drop for UartDriver {
    fn drop(&mut self) {
        pr_info!("UartDriver Drop!");
        unsafe { bindings::uart_unregister_driver(&mut self.0) };
    }
}

unsafe impl Send for UartDriver {}
unsafe impl Sync for UartDriver {}

/// UartDriverRef
pub struct UartDriverRef(*mut bindings::uart_driver);

impl UartDriverRef {
    #[warn(dead_code)]
    fn as_ptr(&self) -> *mut bindings::uart_driver {
        self.0
    }
}

impl From<bindings::uart_driver> for UartDriver {
    fn from(value: bindings::uart_driver) -> Self {
        Self(value)
    }
}
impl From<*mut bindings::uart_driver> for UartDriverRef {
    fn from(value: *mut bindings::uart_driver) -> Self {
        Self(value)
    }
}
