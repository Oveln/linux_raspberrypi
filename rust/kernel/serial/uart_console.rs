// SPDX-License-Identifier: GPL-2.0

//! Support for uart console.
//!
//! C header: ['include/linux/console.h']

// use super::uart_port::UartPort;

use crate::{
    bindings,
    error::{from_result, Result},
    types::ForeignOwnable,
};

use core::mem::MaybeUninit;

use macros::vtable;

use super::uart_driver::UartDriver;

/// General Console Flags associated with a [`Console`].
/// Bitmask for access mode flags.
///
/// # Examples
///
/// ```
/// use kernel::serial::uart_console;
/// # fn do_something() {}
/// # let flags = 0;
/// if (flags & uart_console::flags::CON_CONSDEV) == uart_console::flags::CON_BOOT {
///     do_something();
/// }
/// ```
pub mod flags {
    /// File is opened in append mode.
    pub const CON_PRINTBUFFER: u32 = bindings::cons_flags_CON_PRINTBUFFER;

    /// Indicates that the console driver is backing.
    pub const CON_CONSDEV: u32 = bindings::cons_flags_CON_CONSDEV;

    /// Indicates if a console is allowed to print records.
    pub const CON_ENABLED: u32 = bindings::cons_flags_CON_ENABLED;

    /// Marks the console driver as early console driver which
    /// is used during boot before the real driver becomes
    /// available.
    pub const CON_BOOT: u32 = bindings::cons_flags_CON_BOOT;

    /// A misnomed historical flag which tells the core code
    /// that the legacy ['console::write'] callback can be invoked
    /// on a CPU which is marked OFFLINE.
    pub const CON_ANYTIME: u32 = bindings::cons_flags_CON_ANYTIME;

    /// FIndicates a braille device which is exempt from
    /// receiving the printk spam for obvious reasons.
    pub const CON_BRL: u32 = bindings::cons_flags_CON_BRL;

    /// The console supports the extended output format of /dev/kmesg
    /// which requires a larger output buffer.
    pub const CON_EXTENDED: u32 = bindings::cons_flags_CON_EXTENDED;

    /// Indicates if a console is suspended. If true,
    /// the printing callbacks must not be called.
    pub const CON_SUSPENDED: u32 = bindings::cons_flags_CON_SUSPENDED;
}

/// Console's operations
#[vtable]
pub trait ConsoleOps {
    /// User data that will be accessible to all operations
    // type Data: ForeignOwnable + Send + Sync = ();
    type Data: Send + Sync = ();

    /// Write callback to output messages (Optional)
    fn console_write(_co: &Console, _s: *const i8, _count: u32);

    /// Read callback for console input (Optional)
    fn console_read(_co: &Console, _s: *mut i8, _count: u32) -> Result<i32>;

    /// Callback for matching a console (Optional)
    fn console_match(_co: &Console, _name: *mut i8, _idx: i32, _options: *mut i8) -> Result<i32>;

    /// The underlying TTY device driver (Optional)
    fn console_device(_co: &Console, _index: *mut i8) -> *mut bindings::tty_driver;
}

/// [`UartDriver`]'s console
///
/// # Invariants
///
/// `self.0` has always valid data.
pub struct Console(bindings::console);
impl Console {
    /// Create a new [`UartDriver`] Console
    pub const fn new<T: ConsoleOps>(name: [i8; 16usize], reg: &'static T::Data) -> Self {
        // SAFETY: `console` is a C structure holding data that has been initialized with 0s,
        // hence it is safe to use as-is.
        let mut console = unsafe { MaybeUninit::<bindings::console>::zeroed().assume_init() };
        console.name = name;
        console.write = Some(console_write_callback::<T>);
        console.read = Some(console_read_callback::<T>);
        console.match_ = Some(console_match_callback::<T>);
        console.device = Some(console_device_callback::<T>);
        console.data = reg as *const _ as _;
        Self(console)
    }

    /// Setup the console other config
    pub const fn with_config(
        mut self,
        flags: i16,
        index: i16,
        cflag: i32,
        ispeed: u32,
        ospeed: u32,
        seq: u64,
        dropped: u64,
    ) -> Self {
        self.0.flags = flags;
        self.0.index = index;
        self.0.cflag = cflag;
        self.0.ispeed = ispeed;
        self.0.ospeed = ospeed;
        self.0.seq = seq;
        self.0.dropped = dropped;
        self
    }

    /// Creates a reference to a [`Console`] from a valid pointer.
    ///
    /// # Safety
    ///
    /// Callers must ensure that `ptr` is valid, non-null, and has a non-zero reference count for
    /// the entire duration when the returned reference exists.
    pub unsafe fn from_raw<'a>(ptr: *mut bindings::console) -> &'a Self {
        // SAFETY: Guaranteed by the safety requirements of the function.
        unsafe { &*ptr.cast() }
    }

    /// Returns a raw pointer to the inner C struct.
    #[inline]
    pub const fn as_ptr(&self) -> *mut bindings::console {
        &self.0 as *const _ as _
    }

    pub fn index(&self) -> i16 {
        self.0.index
    }
}

unsafe impl Send for Console {}
unsafe impl Sync for Console {}

unsafe extern "C" fn console_write_callback<T: ConsoleOps>(
    co: *mut bindings::console,
    s: *const core::ffi::c_char,
    count: core::ffi::c_uint,
) {
    let co = unsafe { Console::from_raw(co) };
    T::console_write(co, s, count);
}

unsafe extern "C" fn console_read_callback<T: ConsoleOps>(
    co: *mut bindings::console,
    s: *mut core::ffi::c_char,
    count: core::ffi::c_uint,
) -> core::ffi::c_int {
    from_result(|| {
        // SAFETY: The value stored as chip data was returned by `into_foreign` during registration.
        let co = unsafe { Console::from_raw(co) };
        T::console_read(co, s, count)
    })
}

unsafe extern "C" fn console_match_callback<T: ConsoleOps>(
    co: *mut bindings::console,
    name: *mut core::ffi::c_char,
    idx: core::ffi::c_int,
    options: *mut core::ffi::c_char,
) -> core::ffi::c_int {
    from_result(|| {
        let co = unsafe { Console::from_raw(co) };
        T::console_match(co, name, idx, options)
    })
}

unsafe extern "C" fn console_device_callback<T: ConsoleOps>(
    co: *mut bindings::console,
    index: *mut core::ffi::c_int,
) -> *mut bindings::tty_driver {
    let co = unsafe { Console::from_raw(co) };
    T::console_device(co, index as *mut i8)
}

// pub mod uart_driver {
//     use crate::console::RawConsole;
//     use crate::console::{self, ConsoleOps};
//     use crate::error::{self, to_result};
//     use crate::prelude::EINVAL;
//     use crate::serial_core::uart_port::{self, RawUartPort};
//     use crate::str::CString;
//     use core::fmt;
//     use core::pin::Pin;
//     use kernel::error::Result;

//     pub struct Options {
//         minor: i32,
//         major: i32,
//         nr: i32,
//     }
//     // uart driver reg
//     pub struct Registration<A: ConsoleOps> {
//         registered: bool,
//         uart_driver: bindings::uart_driver,
//         console: console::Registration<bindings::uart_driver, A>,
//     }

//     impl<A: ConsoleOps> Registration<A> {
//         pub fn register_with_options(
//             self: Pin<&mut Self>,
//             name: fmt::Arguments<'_>,
//             opts: &Options,
//         ) -> Result {
//             let this = unsafe { self.get_unchecked_mut() };
//             if this.registered {
//                 return Err(EINVAL);
//             }
//             // 17 is CON_PRINTBUFFER | CON_ANYTIME
//             // -1 is for the index
//             let console = console::Registration::<bindings::uart_driver, A>::new(
//                 name,
//                 17,
//                 -1,
//                 &this.uart_driver as *const _,
//             )?;

//             let name = CString::try_from_fmt(name)?;

//             this.uart_driver.cons = console.raw_console();
//             this.uart_driver.minor = opts.minor;
//             this.uart_driver.major = opts.major;
//             this.uart_driver.nr = opts.nr;

//             unsafe {
//                 let ret = bindings::uart_register_driver(
//                     &this.uart_driver as *const _ as *mut bindings::uart_driver,
//                 );
//                 if ret < 0 {
//                     return Err(error::Error::from_errno(ret));
//                 }
//             };

//             this.registered = true;

//             Ok(())
//         }

//         pub fn unregister(self: Pin<&mut Self>) {
//             let this = unsafe { self.get_unchecked_mut() };
//             if this.registered {
//                 unsafe {
//                     bindings::uart_unregister_driver(
//                         &this.uart_driver as *const _ as *mut bindings::uart_driver,
//                     );
//                 }
//             }
//         }

//         pub fn add_port(self: Pin<&mut Self>, port: &uart_port::UartPort) -> Result {
//             unsafe {
//                 let this = self.get_unchecked_mut();
//                 to_result(bindings::uart_add_one_port(
//                     &this.uart_driver as *const _ as *mut bindings::uart_driver,
//                     port.raw_uart_port(),
//                 ))
//             }
//         }

//         pub fn remove_port(self: Pin<&mut Self>, port: &uart_port::UartPort) {
//             unsafe {
//                 let this = self.get_unchecked_mut();
//                 bindings::uart_remove_one_port(
//                     &this.uart_driver as *const _ as *mut bindings::uart_driver,
//                     port.raw_uart_port(),
//                 )
//             }
//         }
//     }
// }
