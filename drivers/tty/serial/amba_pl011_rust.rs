// SPDX-License-Identifier: GPL-2.0

//! Driver for AMBA serial ports (PL011).
//!
//! Based on the C driver written by ARM Ltd/Deep Blue Solutions Ltd.

use core::ops::DerefMut;
use kernel::{
    amba,
    arch::processor::cpu_relax,
    bindings, c_str,
    clk::Clk,
    device,
    error::{code::*, Result},
    io_mem::IoMem,
    irq::{local_irq_restore, local_irq_save},
    module_amba_driver, new_device_data, new_mutex_pinned,
    prelude::*,
    serial::{
        ktermbits::Ktermios,
        pl011_config::*,
        tty::SerialStruct,
        uart_console::{self, flags, Console, ConsoleOps},
        uart_driver::UartDriver,
        uart_port::{PortRegistration, UartPort, UartPortOps},
    },
    sync::{self, Arc},
};

const UART_SIZE: usize = 0x200;
const UPIO_MEM: u8 = 2;
const UPIO_MEM32: u8 = 3;

pub const UPF_BOOT_AUTOCONF: u64 = 1_u64 << 28;

pub(crate) const UART_NR: usize = 14;
const AMBA_MAJOR: i32 = 204;
const AMBA_MINOR: i32 = 64;
const DEV_NAME: &CStr = c_str!("ttyAMA");
const DRIVER_NAME: &CStr = c_str!("ttyAMA");

#[derive(Debug, Clone, Copy)]
enum Regs {
    RegDr,
    RegStDmawm,
    RegStTimeout,
    RegFr,
    RegLcrhRx,
    RegLcrhTx,
    RegIbrd,
    RegFbrd,
    RegCr,
    RegIfls,
    RegImsc,
    RegRis,
    RegMis,
    RegIcr,
    RegDmacr,
    RegStXfcr,
    RegStXon1,
    RegStXon2,
    RegStXoff1,
    RegStXoff2,
    RegStItcr,
    RegStItip,
    RegStAbcr,
    RegStAbimsc,

    /* The size of the array - must be last */
    RegArraySize,
}

static PL0111_STD_OFFSETS: [u32; Regs::RegArraySize as usize] = {
    use Regs::*;
    let mut arr = [0; Regs::RegArraySize as usize];
    arr[RegDr as usize] = UART01X_DR;
    arr[RegFr as usize] = UART01X_FR;
    arr[RegLcrhRx as usize] = ST_UART011_LCRH_RX;
    arr[RegLcrhTx as usize] = ST_UART011_LCRH_TX;
    arr[RegIbrd as usize] = UART011_IBRD;
    arr[RegFbrd as usize] = UART011_FBRD;
    arr[RegCr as usize] = UART011_CR;
    arr[RegIfls as usize] = UART011_IFLS;
    arr[RegImsc as usize] = UART011_IMSC;
    arr[RegRis as usize] = UART011_RIS;
    arr[RegMis as usize] = UART011_MIS;
    arr[RegIcr as usize] = UART011_ICR;
    arr[RegDmacr as usize] = UART011_DMACR;
    arr
};

/// A static's struct with all port Data
struct Ports(Vec<Option<Arc<PL011DeviceData>>>);

impl Ports {
    fn find_free_port(&self) -> Option<usize> {
        if self.0.len() >= UART_NR {
            return None;
        }
        for i in 0..self.0.len() {
            if self.0[i].is_none() {
                return Some(i);
            }
        }
        return Some(self.0.len());
    }

    fn get_port(&self, index: usize) -> Option<Arc<PL011DeviceData>> {
        self.0.get(index).and_then(|port| port.clone())
    }

    fn set_port(&mut self, index: usize, port: Arc<PL011DeviceData>) -> Result<()> {
        if index >= self.0.len() {
            self.0.try_resize(index + 1, None)?;
        }
        self.0[index] = Some(port);
        Ok(())
    }

    fn free_port(&mut self, index: usize) {
        if index >= self.0.len() {
            return;
        }
        self.0[index] = None;
    }
}

pub(crate) static mut PORTS: Ports = Ports(Vec::new());

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

struct PL011UartPort<'a>(pub(crate) &'a mut UartPort);
impl PL011UartPort<'_> {
    fn write(&self, val: u32, reg: Regs) {
        dbg!("write {} to {:?}", val, reg.clone());
        let data = self.0.get_data::<PL011DeviceData>();
        let iomem = &data.resources().unwrap().base;
        let offset = PL0111_STD_OFFSETS[reg as usize] as usize;
        if self.0.iotype() == UPIO_MEM32 {
            iomem.try_writel_relaxed(val, offset);
        } else {
            iomem.try_writew_relaxed(val as u16, offset);
        }
    }
    fn read(&self, reg: Regs) -> u32 {
        let data = self.0.get_data::<PL011DeviceData>();
        let iomem = &data.resources().unwrap().base;
        let offset = PL0111_STD_OFFSETS[reg as usize] as usize;
        if self.0.iotype() == UPIO_MEM32 {
            iomem.try_readl_relaxed(offset).unwrap()
        } else {
            iomem.try_readw_relaxed(offset).unwrap().into()
        }
    }
    fn console_putchar(&self, ch: u8) {
        while (self.read(Regs::RegFr) & UART01X_FR_TXFF) != 0 {
            cpu_relax();
        }
        self.write(ch.into(), Regs::RegDr);
    }
    fn console_write(&self, s: *const u8, count: u32) {
        for i in 0..count {
            let ch = unsafe { *s.offset(i.try_into().unwrap()) };
            if ch == '\n' as u8 {
                self.console_putchar(ch);
            }
        }
    }
}

/// This is Struct of pl011_console
struct Pl011Console;
/// Implement supported `Pl011Console`'s operations here.
#[vtable]
impl ConsoleOps for Pl011Console {
    type Data = UartDriver;

    fn console_write(co: &Console, s: *const i8, count: u32) {
        dbg!("console_write\n");
        let data = unsafe { PORTS.get_port(co.index() as usize).unwrap() };
        let mut registrations = data.registrations().ok_or(ENXIO).unwrap();
        let mut port: PL011UartPort<'_> = PL011UartPort(registrations.mut_uart_port());
        let mut old_cr: u32 = 0;
        let mut new_cr: u32;
        let mut locked = true;

        let clk = port.0.get_dev().unwrap().clk_get().unwrap();
        let enabled_clk = clk.prepare_enable().unwrap();

        let flags = local_irq_save();
        if port.0.get_sysrq() != 0 {
            locked = true;
        } else {
            port.0.lock();
        }

        if !data.vendor.always_enabled {
            old_cr = port.read(Regs::RegCr);
            new_cr = old_cr & !UART011_CR_CTSEN;
            new_cr = new_cr | UART01X_CR_UARTEN | UART011_CR_TXE;
            port.write(new_cr, Regs::RegCr);
        }

        port.console_write(s as *const u8, count);

        while ((port.read(Regs::RegFr) ^ data.vendor.inv_fr) & data.vendor.fr_busy) != 0 {
            cpu_relax();
        }
        if !data.vendor.always_enabled {
            port.write(old_cr, Regs::RegCr);
        }
        if locked {
            port.0.unlock();
        }
        local_irq_restore(flags);
    }

    fn console_read(_co: &Console, _s: *mut i8, _count: u32) -> Result<i32> {
        dbg!("console_read ok");
        Ok(0)
    }

    fn console_match(_co: &Console, _name: *mut i8, _idx: i32, _options: *mut i8) -> Result<i32> {
        dbg!("console_match ok");
        Ok(0)
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
        dbg!("tx_empty\n");
        0
    }
    fn set_mctrl(_: &UartPort, _: u32) {
        dbg!("set_mctrl\n");
    }
    fn get_mctrl(_: &UartPort) -> u32 {
        dbg!("get_mctrl\n");
        0
    }
    fn stop_tx(_: &UartPort) {
        dbg!("stop_tx\n");
    }
    fn start_tx(_: &UartPort) {
        dbg!("start_tx\n");
    }
    fn throttle(_: &UartPort) {
        dbg!("throttle\n");
    }
    fn unthrottle(_: &UartPort) {
        dbg!("unthrottle\n");
    }
    fn send_xchar(_: &UartPort, _: i8) {
        dbg!("send_xchar\n");
    }
    fn stop_rx(_: &UartPort) {
        dbg!("stop_rx\n");
    }
    fn start_rx(_: &UartPort) {
        dbg!("start_rx\n");
    }
    fn break_ctl(_: &UartPort, _: i32) {
        dbg!("break_ctl\n");
    }
    fn startup(_: &UartPort) -> i32 {
        dbg!("startup\n");
        0
    }
    fn shutdown(_: &UartPort) {
        dbg!("shutdown\n");
    }
    fn flush_buffer(_: &UartPort) {
        dbg!("flush_buffer\n");
    }
    fn set_termios(_: &UartPort, _: &mut Ktermios, _: &Ktermios) {
        dbg!("set_termios\n");
    }
    fn set_ldisc(_: &UartPort, _: &mut Ktermios) {
        dbg!("set_ldisc\n");
    }
    fn pm(_: &UartPort, _: u32, _: u32) {
        dbg!("pm\n");
    }
    fn enable_ms(_: &UartPort) {
        dbg!("enable_ms\n");
    }
    fn port_type(_: &UartPort) -> *const i8 {
        dbg!("port_type\n");
        0 as *const i8
    }
    fn release_port(_: &UartPort) {
        dbg!("release_port\n");
    }
    fn request_port(_: &UartPort) -> i32 {
        dbg!("request_port\n");
        0
    }
    fn config_port(_: &UartPort, _: i32) {
        dbg!("config_port\n");
    }
    fn verify_port(_: &UartPort, _: &mut SerialStruct) -> i32 {
        dbg!("verify_port\n");
        0
    }
    fn ioctl(_: &UartPort, _: u32, _: u64) -> i32 {
        dbg!("ioctl\n");
        0
    }
    fn poll_init(_: &UartPort) -> i32 {
        dbg!("poll_init\n");
        0
    }
    fn poll_put_char(_: &UartPort, _: u8) {
        dbg!("poll_put_char\n");
    }
    fn poll_get_char(_: &UartPort) -> i32 {
        dbg!("poll_get_char\n");
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

        let portnr = unsafe { PORTS.find_free_port().ok_or(ENXIO)? };
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
        unsafe { PORTS.set_port(portnr, arc_portdata.clone()) };
        drop(registration);
        dbg!("********* PL011 registered *********\n");
        Ok(arc_portdata)
    }

    fn remove(data: &Self::Data) {
        dbg!("********* PL011 remove *********\n");
        let portnr: usize = data
            .registrations()
            .ok_or(ENXIO)
            .unwrap()
            .ref_uart_port()
            .get_portnr() as usize;
        unsafe { PORTS.free_port(portnr) }
        dbg!("********* PL011 remove end *********\n");
    }
}

module_amba_driver! {
    type: PL011Device,
    name: "pl011_uart_rust",
    author: "Oveln",
    license: "GPL",
    initcall: "arch",
}

// /// pl011 register write
// fn pl011_write(val: u32, membase: *mut u8, reg: usize, iotype: u8) {
//     let addr = membase.wrapping_add(reg);
//     if iotype == UPIO_MEM32 as u8 {
//         unsafe { bindings::writel_relaxed(val as _, addr as _) };
//     } else {
//         unsafe { bindings::writew_relaxed(val as _, addr as _) };
//     }
// }
