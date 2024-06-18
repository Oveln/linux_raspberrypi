use core::{marker::Copy, pin::Pin};

use kernel::error::Result;
use kernel::prelude::InPlaceInit;
use kernel::{prelude::*, serial_core, console};
use kernel::serial_core::uart_port;
use kernel::sync::Arc;
use kernel::{
    amba, define_amba_id_table,
    error::{self, Error},
    module_amba_driver, pr_info, pr_warn,
    serial_core::uart_port::UartPort,
};

const UART_NR: usize = 14;

struct AmbaPorts {
    ports: Vec<Option<Arc<AmbaUartPort>>>,
}

impl AmbaPorts {
    fn get_port(&self, index: usize) -> Option<Arc<AmbaUartPort>> {
        if index >= UART_NR {
            pr_warn!("invalid uart index");
            return None;
        }
        match self.ports[index] {
            None => {
                pr_warn!("no uart port");
                return None;
            }
            Some(ref p) => Some(p.clone()),
        }
    }

    fn find_free_port(&mut self) -> Option<usize> {
        self.ports.iter().position(|x| x.is_none())
    }

    fn set_port(&mut self, index: usize, port: Arc<AmbaUartPort>) -> Result<()> {
        if index >= UART_NR {
            pr_warn!("invalid uart index");
            return Err(error::code::EINVAL);
        }
        self.ports[index] = Some(port);
        Ok(())
    }

    fn free_port(&mut self, index: usize) {
        self.ports[index] = None;
    }
}

// static AMBA_PORTS: AmbaPorts = AmbaPorts {
//     ports: Vec::try_with_capacity(UART_NR).unwrap(),
// };

#[pin_data]
struct AmbaUartPort {
    #[pin]
    uart_port: UartPort,
}

unsafe impl Sync for AmbaUartPort {}
unsafe impl Send for AmbaUartPort {}

impl AmbaUartPort {
    fn try_new() -> Result<Arc<Self>> {
        Ok(Arc::pin_init(pin_init!(Self {
            uart_port: UartPort::new::<AmbaUartOps>()
        }))?)
    }
}

struct AmbaUartOps {}

impl uart_port::UartOps for AmbaUartOps {
    fn tx_empty(uart_port: &mut UartPort) -> u32 {
        unimplemented!()
    }

    fn set_mctrl(uart_port: &mut UartPort, mctrl: u32) {
        unimplemented!()
    }

    fn get_mctrl(uart_port: &mut UartPort) -> u32 {
        unimplemented!()
    }

    fn stop_tx(uart_port: &mut UartPort) {
        unimplemented!()
    }

    fn start_tx(uart_port: &mut UartPort) {
        unimplemented!()
    }

    fn throttle(uart_port: &mut UartPort) {
        unimplemented!()
    }

    fn unthrottle(uart_port: &mut UartPort) {
        unimplemented!()
    }

    fn send_xchar(uart_port: &mut UartPort, ch: i8) {
        unimplemented!()
    }

    fn stop_rx(uart_port: &mut UartPort) {
        unimplemented!()
    }

    fn start_rx(uart_port: &mut UartPort) {
        unimplemented!()
    }

    fn enable_ms(uart_port: &mut UartPort) {
        unimplemented!()
    }

    fn break_ctl(uart_port: &mut UartPort, ctl: i32) {
        unimplemented!()
    }

    fn startup(uart_port: &mut UartPort) -> i32 {
        unimplemented!()
    }

    fn shutdown(uart_port: &mut UartPort) {
        unimplemented!()
    }

    fn flush_buffer(uart_port: &mut UartPort) {
        unimplemented!()
    }

    fn set_termios(
        uart_port: &mut UartPort,
        new: *mut serial_core::uart_port::ktermios,
        old: *const serial_core::uart_port::ktermios,
    ) {
        unimplemented!()
    }

    fn set_ldisc(uart_port: &mut UartPort, arg2: *mut serial_core::uart_port::ktermios) {
        unimplemented!()
    }

    fn pm(uart_port: &mut UartPort, state: u32, oldstate: u32) {
        unimplemented!()
    }

    fn type_(uart_port: &mut UartPort) -> *const i8 {
        unimplemented!()
    }

    fn release_port(uart_port: &mut UartPort) {
        unimplemented!()
    }

    fn request_port(uart_port: &mut UartPort) -> i32 {
        unimplemented!()
    }

    fn config_port(uart_port: &mut UartPort, arg2: i32) {
        unimplemented!()
    }

    fn verify_port(uart_port: &mut UartPort, arg2: *mut serial_core::uart_port::serial_struct) -> i32 {
        unimplemented!()
    }

    fn ioctl(uart_port: &mut UartPort, arg2: u32, arg3: u64) -> i32 {
        unimplemented!()
    }

    fn poll_init(uart_port: &mut UartPort) -> i32 {
        unimplemented!()
    }

    fn poll_put_char(uart_port: &mut UartPort, arg2: u8) {
        unimplemented!()
    }

    fn poll_get_char(uart_port: &mut UartPort) -> i32 {
        unimplemented!()
    }
}

struct ConsoleOps {}

#[vtable]
impl console::ConsoleOps for ConsoleOps {
    type DataType = UartDriver;
}

struct UartDriver {
    amba_ports: AmbaPorts,
    driver: Pin<Box<serial_core::uart_driver::Registration<ConsoleOps>>>
}

impl amba::Driver for UartDriver {
    type Data = ();

    define_amba_id_table! {(), [
        ({ id: 0x00041011, mask: 0x000fffff }, None),
    ]}

    fn probe(
        dev: &mut amba::Device,
        id_info: core::prelude::v1::Option<&Self::IdInfo>,
    ) -> Result<(), Error> {
        pr_warn!("UartDriver::probe");
        Ok(())
    }
}

impl Drop for UartDriver {
    fn drop(&mut self) {
        pr_info!("UartDriver::drop");
    }
}

module_amba_driver! {
    type: UartDriver,
    name: "uart_driver",
    author: "Oveln",
    license: "GPL v2",
    initcall: "arch",
}
