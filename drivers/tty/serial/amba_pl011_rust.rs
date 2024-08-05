// SPDX-License-Identifier: GPL-2.0

//! Driver for AMBA serial ports (PL011).
//!
//! Based on the C driver written by ARM Ltd/Deep Blue Solutions Ltd.

use core::ops::DerefMut;
use kernel::{
    amba, bindings, c_str,
    clk::Clk,
    device,
    error::{code::*, Result},
    io_mem::IoMem,
    module_amba_driver, new_device_data, new_mutex_pinned,
    prelude::*,
    serial::{
        ktermbits::Ktermios,
        pl011_config::*,
        tty::SerialStruct,
        uart_console::{flags, Console, ConsoleOps},
        uart_driver::UartDriver,
        uart_port::{PortRegistration, UartPort, UartPortOps},
    },
    sync::{self, Arc},
};

const UART_SIZE: usize = 0x200;
const UPIO_MEM: u32 = 2;
const UPIO_MEM32: u32 = 3;

pub const UPF_BOOT_AUTOCONF: u64 = 1_u64 << 28;

pub(crate) const UART_NR: usize = 14;
const AMBA_MAJOR: i32 = 204;
const AMBA_MINOR: i32 = 64;
const DEV_NAME: &CStr = c_str!("ttyAMA");
const DRIVER_NAME: &CStr = c_str!("ttyAMA");

/// A static's struct with all port Data
pub(crate) static mut PORTS: [Option<&UartPort>; UART_NR] = [None; UART_NR];

/// This amba_uart_console static's struct
static AMBA_CONSOLE: Console = {
    let name: [i8; 16usize] = [
        't' as _, 't' as _, 'y' as _, 'A' as _, 'M' as _, 'A' as _, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    Console::new::<Pl011Console>(name, &UART_DRIVER).with_config(
        (flags::CON_PRINTBUFFER | flags::CON_ANYTIME) as _,
        -1,
        0,
        0,
        0,
        0,
        0,
    )
};

/// This uart_driver static's struct
pub(crate) static UART_DRIVER: UartDriver =
    UartDriver::new(&THIS_MODULE, DRIVER_NAME, DEV_NAME, &AMBA_CONSOLE).with_config(
        AMBA_MAJOR,
        AMBA_MINOR,
        UART_NR as _,
    );

/// This is Struct of pl011_console
struct Pl011Console;
/// Implement supported `Pl011Console`'s operations here.
#[vtable]
impl ConsoleOps for Pl011Console {
    type Data = UartDriver;

    fn console_write(co: &Console, _s: *const i8, _count: u32) {
        pr_info!("console_write ok");
    }

    fn console_read(_co: &Console, _s: *mut i8, _count: u32) -> Result<i32> {
        pr_info!("console_read ok");
        Ok(0)
    }

    fn console_match(_co: &Console, _name: *mut i8, _idx: i32, _options: *mut i8) -> Result<i32> {
        pr_info!("console_match ok");
        Ok(0)
    }

    fn console_device(_co: &Console, _index: *mut i8) -> *mut bindings::tty_driver {
        pr_info!("console_device ok");
        todo!()
    }
}

pub(crate) static VENDOR_DATA: VendorData = VendorData {
    ifls: UART011_IFLS_RX4_8 | UART011_IFLS_TX4_8,
    fr_busy: UART01X_FR_BUSY,
    fr_dsr: UART01X_FR_DSR,
    fr_cts: UART01X_FR_CTS,
    fr_ri: UART011_FR_RI,
    inv_fr: 0,
    access_32b: false,
    oversampling: false,
    dma_threshold: false,
    cts_event_workaround: false,
    always_enabled: false,
    fixfixed_options: false,
};

#[derive(Copy, Clone)]
struct PL011Data {
    // reg_offset: u16,
    im: u32,
    old_status: u32,
    fifosize: u32,
    // fixed_baud: u32,
    type_: &'static str,
    vendor: &'static VendorData,
}

struct PL011Resources {
    base: IoMem<UART_SIZE>,
    parent_irq: u32,
}

type PL011Registrations = PortRegistration<PL011Device>;
type PL011DeviceData = device::Data<PL011Registrations, PL011Resources, PL011Data>;

// Linux Raw id table
kernel::define_amba_id_table! {MY_AMDA_ID_TABLE, (), [
    ({id: 0x00041011, mask: 0x000fffff}, None),
]}
// Linux Raw id table
kernel::module_amba_id_table!(UART_MOD_TABLE, MY_AMDA_ID_TABLE);

struct PL011Device;
#[vtable]
impl UartPortOps for PL011Device {
    type Data = Arc<PL011DeviceData>;
    fn tx_empty(_: &UartPort) -> u32 {
        0
    }
    fn set_mctrl(_: &UartPort, _: u32) {}
    fn get_mctrl(_: &UartPort) -> u32 {
        0
    }
    fn stop_tx(_: &UartPort) {}
    fn start_tx(_: &UartPort) {}
    fn throttle(_: &UartPort) {}
    fn unthrottle(_: &UartPort) {}
    fn send_xchar(_: &UartPort, _: i8) {}
    fn stop_rx(_: &UartPort) {}
    fn start_rx(_: &UartPort) {}
    fn break_ctl(_: &UartPort, _: i32) {}
    fn startup(_: &UartPort) -> i32 {
        0
    }
    fn shutdown(_: &UartPort) {}
    fn flush_buffer(_: &UartPort) {}
    fn set_termios(_: &UartPort, _: &mut Ktermios, _: &Ktermios) {}
    fn set_ldisc(_: &UartPort, _: &mut Ktermios) {}
    fn pm(_: &UartPort, _: u32, _: u32) {}
    fn enable_ms(_: &UartPort) {}
    fn port_type(_: &UartPort) -> *const i8 {
        unimplemented!()
    }
    fn release_port(_: &UartPort) {}
    fn request_port(_: &UartPort) -> i32 {
        0
    }
    fn config_port(_: &UartPort, _: i32) {}
    fn verify_port(_: &UartPort, _: &mut SerialStruct) -> i32 {
        0
    }
    fn ioctl(_: &UartPort, _: u32, _: u64) -> i32 {
        0
    }
    fn poll_init(_: &UartPort) -> i32 {
        0
    }
    fn poll_put_char(_: &UartPort, _: u8) {}
    fn poll_get_char(_: &UartPort) -> i32 {
        0
    }
}
impl amba::Driver for PL011Device {
    type Data = Arc<PL011DeviceData>;

    kernel::driver_amba_id_table!(MY_AMDA_ID_TABLE);
    fn probe(adev: &mut amba::Device, _data: Option<&Self::IdInfo>) -> Result<Self::Data> {
        dev_info!(adev, "{} PL011 (probe)\n", adev.name());
        dbg!("********** PL011 (probe) *********\n");

        let dev = device::Device::from_dev(adev);

        let portnr = pl011_find_free_port()?;
        dev_info!(adev, "portnr is {}\n", portnr);
        let clk = dev.clk_get().unwrap(); // 获得clk
        let fifosize = if adev.revision_get().unwrap() < 3 {
            16
        } else {
            32
        };
        let iotype = UPIO_MEM as u8;
        let reg_base = adev.take_resource().ok_or(ENXIO)?;
        let reg_mem: IoMem<UART_SIZE> = unsafe { IoMem::try_new(&reg_base)? };
        let mapbase = reg_base.get_offset();
        let membase = reg_mem.get();
        let irq = adev.irq(0).ok_or(ENXIO)?;

        dev_info!(adev, "fifosize is {}\n", fifosize);
        dev_info!(adev, "mapbase is 0x{:x}\n", mapbase);
        dev_info!(adev, "membase is 0x{:p}\n", membase);
        dev_info!(adev, "irq is {}\n", irq);
        let has_sysrq = 1;
        let flags = UPF_BOOT_AUTOCONF;
        let port = UartPort::new().setup(
            membase,
            mapbase,
            irq,
            iotype,
            flags,
            has_sysrq,
            fifosize,
            portnr as _,
        );

        let data = new_device_data!(
            PL011Registrations::new(port),
            PL011Resources {
                base: reg_mem,
                parent_irq: irq,
            },
            PL011Data {
                im: 0,
                old_status: 0,
                fifosize,
                type_: "PL011",
                vendor: &VENDOR_DATA,
            },
            "pl011"
        )?;

        let arc_portdata: Arc<PL011DeviceData> = Arc::from(data);

        if !UART_DRIVER.is_registered() {
            UART_DRIVER.register()?;
        }
        let mut registration = arc_portdata.registrations().ok_or(ENXIO)?;
        let registration_mut = unsafe { Pin::new_unchecked(registration.deref_mut()) };
        registration_mut.register(adev, &UART_DRIVER, portnr, arc_portdata.clone());
        // unsafe { PORTS[portnr] = Some()) }
        drop(registration);
        dbg!("********* PL011 registered *********\n");
        Ok(arc_portdata)
    }
}

module_amba_driver! {
    type: PL011Device,
    name: "pl011_uart_rust",
    author: "Oveln",
    license: "GPL",
    initcall: "arch",
}

/// Find available driver ports sequentially.
fn pl011_find_free_port() -> Result<usize> {
    for (index, port) in unsafe { PORTS.iter().enumerate() } {
        if let None = port {
            return Ok(index);
        }
    }
    return Err(EBUSY);
}

/// pl011 register write
fn pl011_write(val: u32, membase: *mut u8, reg: usize, iotype: u8) {
    let addr = membase.wrapping_add(reg);
    if iotype == UPIO_MEM32 as u8 {
        unsafe { bindings::writel_relaxed(val as _, addr as _) };
    } else {
        unsafe { bindings::writew_relaxed(val as _, addr as _) };
    }
}
