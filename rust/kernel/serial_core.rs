pub mod uart_port {
    use crate::{console, device::Device};
    use core::marker;
    use kernel::error::{self, Result};

    use crate::{container_of, types::Opaque};
    use bindings::{serial_struct, uart_ops, uart_port};

    pub unsafe trait RawUartPort {
        fn raw_uart_port(&self) -> *mut uart_port;
    }

    #[repr(transparent)]
    pub struct UartPort(pub(crate) Opaque<uart_port>);

    unsafe impl RawUartPort for UartPort {
        fn raw_uart_port(&self) -> *mut uart_port {
            &self as *const _ as *mut uart_port
        }
    }

    impl UartPort {
        pub fn from_ptr<'a>(ptr: *mut uart_port) -> &'a mut Self {
            unsafe { &mut *ptr.cast() }
        }

        pub fn new<T: UartOps>(ops: &T) -> Self {
            let mut uart_port = bindings::uart_port::default();
            uart_port.ops = unsafe { OperationsVtable::<T>::build() };
            Self(Opaque::new(uart_port))
        }
    }

    /// UART operations vtable
    /// * @tx_empty:      check if the UART TX FIFO is empty
    /// * @set_mctrl:    set the modem control register
    /// * @get_mctrl:    get the modem control register
    /// * @stop_tx:      stop transmitting
    /// * @start_tx:    start transmitting
    /// * @throttle:     stop receiving
    /// * @unthrottle:   start receiving
    /// * @send_xchar:  send a break character
    /// * @stop_rx:      stop receiving
    /// * @start_rx:    start receiving
    /// * @enable_ms:    enable modem status interrupts
    /// * @break_ctl:   set the break control
    /// * @startup:      start the UART
    /// * @shutdown:     shutdown the UART
    /// * @flush_buffer: flush the UART buffer
    /// * @set_termios: set the termios structure
    /// * @set_ldisc:    set the line discipline
    /// * @pm:            power management
    /// * @type:          get the type of the UART
    /// * @release_port: release the UART port
    /// * @request_port: request the UART port
    /// * @config_port:  configure the UART port
    /// * @verify_port:  verify the UART port
    /// * @ioctl:        ioctl handler
    pub trait UartOps {
        fn tx_empty(uart_port: &mut UartPort) -> u32;
        fn set_mctrl(uart_port: &mut UartPort, mctrl: u32);
        fn get_mctrl(uart_port: &mut UartPort) -> u32;
        fn stop_tx(uart_port: &mut UartPort);
        fn start_tx(uart_port: &mut UartPort);
        fn throttle(uart_port: &mut UartPort);
        fn unthrottle(uart_port: &mut UartPort);
        fn send_xchar(uart_port: &mut UartPort, ch: i8);
        fn stop_rx(uart_port: &mut UartPort);
        fn start_rx(uart_port: &mut UartPort);
        fn enable_ms(uart_port: &mut UartPort);
        fn break_ctl(uart_port: &mut UartPort, ctl: i32);
        fn startup(uart_port: &mut UartPort) -> i32;
        fn shutdown(uart_port: &mut UartPort);
        fn flush_buffer(uart_port: &mut UartPort);
        fn set_termios(
            uart_port: &mut UartPort,
            new: *mut bindings::ktermios,
            old: *const bindings::ktermios,
        );
        fn set_ldisc(uart_port: &mut UartPort, arg2: *mut bindings::ktermios);
        fn pm(uart_port: &mut UartPort, state: u32, oldstate: u32);
        fn type_(uart_port: &mut UartPort) -> *const i8;
        fn release_port(uart_port: &mut UartPort);
        fn request_port(uart_port: &mut UartPort) -> i32;
        fn config_port(uart_port: &mut UartPort, arg2: i32);
        fn verify_port(uart_port: &mut UartPort, arg2: *mut serial_struct) -> i32;
        fn ioctl(uart_port: &mut UartPort, arg2: u32, arg3: u64) -> i32;

        fn poll_init(uart_port: &mut UartPort) -> i32;
        fn poll_put_char(uart_port: &mut UartPort, arg2: u8);
        fn poll_get_char(uart_port: &mut UartPort) -> i32;
    }

    pub(crate) struct OperationsVtable<T>(marker::PhantomData<T>);

    impl<T: UartOps> OperationsVtable<T> {
        unsafe extern "C" fn tx_empty(uart_port: *mut uart_port) -> core::ffi::c_uint {
            let uart_port = UartPort::from_ptr(uart_port);
            T::tx_empty(uart_port)
        }

        unsafe extern "C" fn set_mctrl(uart_port: *mut uart_port, mctrl: core::ffi::c_uint) {
            let uart_port = UartPort::from_ptr(uart_port);
            T::set_mctrl(uart_port, mctrl)
        }

        unsafe extern "C" fn get_mctrl(uart_port: *mut uart_port) -> core::ffi::c_uint {
            let uart_port = UartPort::from_ptr(uart_port);
            T::get_mctrl(uart_port)
        }

        unsafe extern "C" fn stop_tx(uart_port: *mut uart_port) {
            let uart_port = UartPort::from_ptr(uart_port);
            T::stop_tx(uart_port)
        }

        unsafe extern "C" fn start_tx(uart_port: *mut uart_port) {
            let uart_port = UartPort::from_ptr(uart_port);
            T::start_tx(uart_port)
        }

        unsafe extern "C" fn throttle(uart_port: *mut uart_port) {
            let uart_port = UartPort::from_ptr(uart_port);
            T::throttle(uart_port)
        }

        unsafe extern "C" fn unthrottle(uart_port: *mut uart_port) {
            let uart_port = UartPort::from_ptr(uart_port);
            T::unthrottle(uart_port)
        }

        unsafe extern "C" fn send_xchar(uart_port: *mut uart_port, ch: core::ffi::c_char) {
            let uart_port = UartPort::from_ptr(uart_port);
            T::send_xchar(uart_port, ch)
        }

        unsafe extern "C" fn stop_rx(uart_port: *mut uart_port) {
            let uart_port = UartPort::from_ptr(uart_port);
            T::stop_rx(uart_port)
        }

        unsafe extern "C" fn start_rx(uart_port: *mut uart_port) {
            let uart_port = UartPort::from_ptr(uart_port);
            T::start_rx(uart_port)
        }

        unsafe extern "C" fn enable_ms(uart_port: *mut uart_port) {
            let uart_port = UartPort::from_ptr(uart_port);
            T::enable_ms(uart_port)
        }

        unsafe extern "C" fn break_ctl(uart_port: *mut uart_port, ctl: core::ffi::c_int) {
            let uart_port = UartPort::from_ptr(uart_port);
            T::break_ctl(uart_port, ctl)
        }

        unsafe extern "C" fn startup(uart_port: *mut uart_port) -> core::ffi::c_int {
            let uart_port = UartPort::from_ptr(uart_port);
            T::startup(uart_port)
        }

        unsafe extern "C" fn shutdown(uart_port: *mut uart_port) {
            let uart_port = UartPort::from_ptr(uart_port);
            T::shutdown(uart_port)
        }

        unsafe extern "C" fn flush_buffer(uart_port: *mut uart_port) {
            let uart_port = UartPort::from_ptr(uart_port);
            T::flush_buffer(uart_port)
        }

        unsafe extern "C" fn set_termios(
            uart_port: *mut uart_port,
            new: *mut bindings::ktermios,
            old: *const bindings::ktermios,
        ) {
            let uart_port = UartPort::from_ptr(uart_port);
            T::set_termios(uart_port, new, old)
        }

        unsafe extern "C" fn set_ldisc(uart_port: *mut uart_port, arg2: *mut bindings::ktermios) {
            let uart_port = UartPort::from_ptr(uart_port);
            T::set_ldisc(uart_port, arg2)
        }

        unsafe extern "C" fn pm(
            uart_port: *mut uart_port,
            state: core::ffi::c_uint,
            oldstate: core::ffi::c_uint,
        ) {
            let uart_port = UartPort::from_ptr(uart_port);
            T::pm(uart_port, state, oldstate)
        }

        unsafe extern "C" fn type_(uart_port: *mut uart_port) -> *const core::ffi::c_char {
            let uart_port = UartPort::from_ptr(uart_port);
            T::type_(uart_port)
        }

        unsafe extern "C" fn release_port(uart_port: *mut uart_port) {
            let uart_port = UartPort::from_ptr(uart_port);
            T::release_port(uart_port)
        }

        unsafe extern "C" fn request_port(uart_port: *mut uart_port) -> core::ffi::c_int {
            let uart_port = UartPort::from_ptr(uart_port);
            T::request_port(uart_port)
        }

        unsafe extern "C" fn config_port(uart_port: *mut uart_port, arg2: core::ffi::c_int) {
            let uart_port = UartPort::from_ptr(uart_port);
            T::config_port(uart_port, arg2)
        }

        unsafe extern "C" fn verify_port(
            uart_port: *mut uart_port,
            arg2: *mut serial_struct,
        ) -> core::ffi::c_int {
            let uart_port = UartPort::from_ptr(uart_port);
            T::verify_port(uart_port, arg2)
        }

        unsafe extern "C" fn ioctl(
            uart_port: *mut uart_port,
            arg2: core::ffi::c_uint,
            arg3: core::ffi::c_ulong,
        ) -> core::ffi::c_int {
            let uart_port = UartPort::from_ptr(uart_port);
            T::ioctl(uart_port, arg2, arg3)
        }

        unsafe extern "C" fn poll_init(uart_port: *mut uart_port) -> core::ffi::c_int {
            let uart_port = UartPort::from_ptr(uart_port);
            T::poll_init(uart_port)
        }

        unsafe extern "C" fn poll_put_char(uart_port: *mut uart_port, arg2: core::ffi::c_uchar) {
            let uart_port = UartPort::from_ptr(uart_port);
            T::poll_put_char(uart_port, arg2)
        }

        unsafe extern "C" fn poll_get_char(uart_port: *mut uart_port) -> core::ffi::c_int {
            let uart_port = UartPort::from_ptr(uart_port);
            T::poll_get_char(uart_port)
        }

        const VTABLE: bindings::uart_ops = bindings::uart_ops {
            tx_empty: Some(Self::tx_empty),
            set_mctrl: Some(Self::set_mctrl),
            get_mctrl: Some(Self::get_mctrl),
            stop_tx: Some(Self::stop_tx),
            start_tx: Some(Self::start_tx),
            throttle: Some(Self::throttle),
            unthrottle: Some(Self::unthrottle),
            send_xchar: Some(Self::send_xchar),
            stop_rx: Some(Self::stop_rx),
            start_rx: Some(Self::start_rx),
            enable_ms: Some(Self::enable_ms),
            break_ctl: Some(Self::break_ctl),
            startup: Some(Self::startup),
            shutdown: Some(Self::shutdown),
            flush_buffer: Some(Self::flush_buffer),
            set_termios: Some(Self::set_termios),
            set_ldisc: Some(Self::set_ldisc),
            pm: Some(Self::pm),
            type_: Some(Self::type_),
            release_port: Some(Self::release_port),
            request_port: Some(Self::request_port),
            config_port: Some(Self::config_port),
            verify_port: Some(Self::verify_port),
            ioctl: Some(Self::ioctl),
            poll_init: Some(Self::poll_init),
            poll_put_char: Some(Self::poll_put_char),
            poll_get_char: Some(Self::poll_get_char),
        };

        pub(crate) unsafe fn build() -> *const bindings::uart_ops {
            &Self::VTABLE as *const _
        }
    }
}

pub mod uart_driver {
    use crate::console::RawConsole;
    use crate::console::{self, ConsoleOps};
    use crate::error::{self, to_result};
    use crate::prelude::EINVAL;
    use crate::serial_core::uart_port::{self, RawUartPort};
    use crate::str::CString;
    use alloc::boxed::Box;
    use core::fmt;
    use core::pin::Pin;
    use kernel::error::Result;

    pub struct Options {
        minor: Option<i32>,
        major: Option<i32>,
        nr: Option<i32>,
    }

    impl Options {
        pub fn new() -> Options {
            Self {
                minor: None,
                major: None,
                nr: None,
            }
        }

        pub const fn minor(&mut self, v: i32) -> &mut Self {
            self.minor = Some(v);
            self
        }

        pub const fn major(&mut self, v: i32) -> &mut Self {
            self.major = Some(v);
            self
        }

        pub const fn nr(&mut self, v: i32) -> &mut Self {
            self.nr = Some(v);
            self
        }

        pub fn register<T: ConsoleOps>(
            &self,
            reg: Pin<&mut Registration<T>>,
            name: fmt::Arguments<'_>,
        ) -> Result {
            reg.register_with_options(name, self)
        }

        pub fn register_new<T: ConsoleOps>(
            &self,
            name: fmt::Arguments<'_>,
        ) -> Result<Pin<Box<Registration<T>>>> {
            // let mut r = Pin::from(Box::try_new(Registration::<T>::new())?);
            let mut r: Pin<Box<Registration<T>>> =
                Pin::from(Box::try_new(Registration::<T>::new())?);
            self.register(r.as_mut(), name)?;
            Ok(r)
        }
    }
    // uart driver reg
    pub struct Registration<A: ConsoleOps> {
        registered: bool,
        uart_driver: bindings::uart_driver,
        console: bindings::console,
        _marker: core::marker::PhantomData<A>,
    }

    impl<A: ConsoleOps> Registration<A> {
        pub fn new() -> Registration<A> {
            Self {
                registered: false,
                uart_driver: bindings::uart_driver::default(),
                console: bindings::console::default(),
                _marker: core::marker::PhantomData,
            }
        }

        pub fn new_pinned(name: fmt::Arguments<'_>) -> Result<Pin<Box<Self>>> {
            Options::new().register_new(name)
        }

        pub fn register_with_options(
            self: Pin<&mut Self>,
            name: fmt::Arguments<'_>,
            opts: &Options,
        ) -> Result {
            let this = unsafe { self.get_unchecked_mut() };
            if this.registered {
                return Err(EINVAL);
            }
            // 17 is CON_PRINTBUFFER | CON_ANYTIME
            // -1 is for the index
            this.console = console::Console::<bindings::uart_driver>::new::<A>(
                name,
                17,
                -1,
                &this.uart_driver as *const _,
            )?
            .into_inner();

            let name = CString::try_from_fmt(name)?;

            this.uart_driver.cons = &this.console as *const _ as *mut bindings::console;
            // SERIAL_AMBA_MAJOR
            this.uart_driver.minor = opts.minor.unwrap_or(204);
            // SERIAL_AMBA_MINOR
            this.uart_driver.major = opts.major.unwrap_or(64);
            // UART_NR
            this.uart_driver.nr = opts.nr.unwrap_or(14);

            this.uart_driver.driver_name = name.as_char_ptr();
            this.uart_driver.dev_name = name.as_char_ptr();

            unsafe {
                let ret = bindings::uart_register_driver(
                    &this.uart_driver as *const _ as *mut bindings::uart_driver,
                );
                if ret < 0 {
                    return Err(error::Error::from_errno(ret));
                }
            };

            this.registered = true;

            Ok(())
        }

        pub fn unregister(self: Pin<&mut Self>) {
            let this = unsafe { self.get_unchecked_mut() };
            if this.registered {
                unsafe {
                    bindings::uart_unregister_driver(
                        &this.uart_driver as *const _ as *mut bindings::uart_driver,
                    );
                }
            }
        }

        pub fn add_port(self: Pin<&mut Self>, port: &uart_port::UartPort) -> Result {
            unsafe {
                let this = self.get_unchecked_mut();
                to_result(bindings::uart_add_one_port(
                    &this.uart_driver as *const _ as *mut bindings::uart_driver,
                    port as *const _ as *mut bindings::uart_port,
                ))
            }
        }

        pub fn remove_port(self: Pin<&mut Self>, port: &uart_port::UartPort) {
            unsafe {
                let this = self.get_unchecked_mut();
                bindings::uart_remove_one_port(
                    &this.uart_driver as *const _ as *mut bindings::uart_driver,
                    port as *const _ as *mut bindings::uart_port,
                )
            }
        }
    }
}
