//! Driver for the  Synopsys DesignWare GPIO
//!

#![no_std]
#![feature(const_ptr_as_ref)]
#![feature(const_option)]
#![feature(const_nonnull_new)]

mod enum_primitive;

use enum_primitive::num::FromPrimitive;
use core::ptr::NonNull;
pub(crate) use osl::error::{to_error, Errno, Result};

use tock_registers::{
    interfaces::{Readable, Writeable},
    registers::{ReadWrite,ReadOnly,WriteOnly},
};

/// The maximum number of ports for dw_apb_gpio
pub const DWAPB_MAX_PORTS: usize = 4;
/// The maximum number of GPIOs for each port.
pub const DWAPB_MAX_GPIOS: u32 = 32;

#[repr(C)]
#[allow(non_snake_case)]
pub(crate) struct DwGpioRegisters {
    /// Distributor GPIOs Control Register.
    pub(crate) GPIO_SWPORTA_DR: ReadWrite<u32>,       // 0x00 - GPIO Software Port A Data Register
    pub(crate) GPIO_SWPORTA_DDR: ReadWrite<u32>,      // 0x04 - GPIO Software Port A Data Direction Register
    pub(crate) GPIO_SWPORTA_CTL: ReadWrite<u32>,      // 0x08 - GPIO Software Port A Control Register
    pub(crate) GPIO_SWPORTB_DR: ReadWrite<u32>,       // 0x0C - GPIO Software Port B Data Register
    pub(crate) GPIO_SWPORTB_DDR: ReadWrite<u32>,      // 0x10 - GPIO Software Port B Data Direction Register
    pub(crate) GPIO_SWPORTB_CTL: ReadWrite<u32>,      // 0x14 - GPIO Software Port B Control Register
    pub(crate) GPIO_SWPORTC_DR: ReadWrite<u32>,       // 0x18 - GPIO Software Port C Data Register
    pub(crate) GPIO_SWPORTC_DDR: ReadWrite<u32>,      // 0x1C - GPIO Software Port C Data Direction Register
    pub(crate) GPIO_SWPORTC_CTL: ReadWrite<u32>,      // 0x20 - GPIO Software Port C Control Register
    pub(crate) GPIO_SWPORTD_DR: ReadWrite<u32>,       // 0x24 - GPIO Software Port D Data Register
    pub(crate) GPIO_SWPORTD_DDR: ReadWrite<u32>,      // 0x28 - GPIO Software Port D Data Direction Register
    pub(crate) GPIO_SWPORTD_CTL: ReadWrite<u32>,      // 0x2C - GPIO Software Port D Control Register
    pub(crate) GPIO_INTEN: ReadWrite<u32>,            // 0x30 - GPIO Interrupt Enable Register
    pub(crate) GPIO_INTMASK: ReadWrite<u32>,          // 0x34 - GPIO Interrupt Mask Register
    pub(crate) GPIO_INTTYPE_LEVEL: ReadWrite<u32>,    // 0x38 - GPIO Interrupt Type Level Register
    pub(crate) GPIO_INTPOLARITY: ReadWrite<u32>,      // 0x3C - GPIO Interrupt Polarity Register
    pub(crate) GPIO_INTSTATUS: ReadOnly<u32>,         // 0x40 - GPIO Interrupt Status Register
    pub(crate) GPIO_RAW_INTSTATUS: ReadOnly<u32>,     // 0x44 - GPIO Raw Interrupt Status Register
    pub(crate) GPIO_DEBOUNCE: ReadWrite<u32>,         // 0x48 - GPIO Debounce Enable Register
    pub(crate) GPIO_PORTA_EOI: WriteOnly<u32>,        // 0x4C - GPIO Port A End of Interrupt Register
    pub(crate) GPIO_EXT_PORTA: ReadOnly<u32>,         // 0x50 - GPIO External Port A Register
    pub(crate) GPIO_EXT_PORTB: ReadOnly<u32>,         // 0x54 - GPIO External Port B Register
    pub(crate) GPIO_EXT_PORTC: ReadOnly<u32>,         // 0x58 - GPIO External Port C Register
    pub(crate) GPIO_EXT_PORTD: ReadOnly<u32>,         // 0x5C - GPIO External Port D Register
    pub(crate) GPIO_LS_SYNC: ReadWrite<u32>,          // 0x60 - GPIO Level-sensitive Synchronization Register
    pub(crate) GPIO_ID_CODE: ReadOnly<u32>,           // 0x64 - GPIO Identification Code Register
    pub(crate) GPIO_INT_BOTHEDGE: ReadWrite<u32>,     // 0x68 - GPIO Interrupt Both Edges Register
    pub(crate) GPIO_VER_ID_CODE: ReadOnly<u32>,       // 0x6C - GPIO Version ID Code Register
    pub(crate) GPIO_CONFIG_REG2: ReadOnly<u32>,       // 0x70 - GPIO Configuration Register 2
    pub(crate) GPIO_CONFIG_REG1: ReadOnly<u32>,       // 0x74 - GPIO Configuration Register 1
}

enum_from_primitive! {
    #[repr(u8)]
    enum Port{
        PortA = 0,
        PortB = 1,
        PortC = 2,
        PortD = 3,
    }
}

/// The Struct of GpioDriver Data,include PORT A~D
#[allow(dead_code)]
pub struct DwGpio{
    ports:[DwGpioPort;DWAPB_MAX_PORTS],
}

/// this is DwGpio's implementation
impl DwGpio {
    /// New a Gpio
    pub fn new(base_addr: *mut u8) -> Self {
        Self {
            ports:[DwGpioPort::new(base_addr, 0); DWAPB_MAX_PORTS]
        }
    }
    /// set gpio port value
    pub fn set_port(&mut self, idx: usize, port: DwGpioPort ) {
        if let Some(old_port) = self.ports.get_mut(idx) {
            *old_port = port;
        } else {
            panic!("Index out of bounds");
        }
    }

}

/// The Struct of DwGpioPort,point to DwGpioRegisters
#[derive(Copy,Clone)]
pub struct DwGpioPort {
    regs: NonNull<DwGpioRegisters>,
    idx: usize,          
}

// SAFETY: `DwGpioPort` holds a non-null pointer to a Gpio registers, which is safe to be used from any thread.
unsafe impl Send for DwGpioPort {}
// SAFETY: `DwGpioPort` holds a non-null pointer to a Gpio registers, references to which are safe to be used
// from any thread.
unsafe impl Sync for DwGpioPort {}

/// this is GpioDriver's implementation
impl DwGpioPort {
    /// New a GpioPort
    pub const fn new( base_addr:*mut u8, idx:usize ) -> Self {
        Self { regs:NonNull::new(base_addr).unwrap().cast(), 
               idx,
        }
    }

    const fn regs(&self) -> &DwGpioRegisters {
        // SAFETY: `self.regs` is non-null and valid.
        unsafe { self.regs.as_ref() }
    }

    /// Returns the direction of the given gpio line.
    pub fn get_direction(&mut self, offset: u32) -> Result<u32>{
        match Port::from_usize(self.idx) {
            Some(Port::PortA) => Ok((self.regs().GPIO_SWPORTA_DDR.get() >> offset) & 1), // Access DDR register for Port A
            Some(Port::PortB) => Ok((self.regs().GPIO_SWPORTB_DDR.get() >> offset) & 1), // Access DDR register for Port B
            Some(Port::PortC) => Ok((self.regs().GPIO_SWPORTC_DDR.get() >> offset) & 1), // Access DDR register for Port C
            Some(Port::PortD) => Ok((self.regs().GPIO_SWPORTD_DDR.get() >> offset) & 1), // Access DDR register for Port D
            None => to_error(Errno::InvalidArgs),                           // Handle invalid index case     
        }
    }

    /// Configures the direction as input of the given gpio line.
    pub fn direction_input(&mut self, offset: u32) -> Result<()> {
        match Port::from_usize(self.idx) {
            Some(Port::PortA) => Ok(self.regs().GPIO_SWPORTA_DDR.set(self.regs().GPIO_SWPORTA_DDR.get() & !( 1 << offset))), // 0 is input for Port A
            Some(Port::PortB) => Ok(self.regs().GPIO_SWPORTB_DDR.set(self.regs().GPIO_SWPORTB_DDR.get() & !( 1 << offset))), // 0 is input for Port B
            Some(Port::PortC) => Ok(self.regs().GPIO_SWPORTC_DDR.set(self.regs().GPIO_SWPORTC_DDR.get() & !( 1 << offset))), // 0 is input for Port C
            Some(Port::PortD) => Ok(self.regs().GPIO_SWPORTD_DDR.set(self.regs().GPIO_SWPORTD_DDR.get() & !( 1 << offset))), // 0 is input for Port D
            None => to_error(Errno::InvalidArgs),                             // Handle invalid index case     
        }
    }

    /// Configures the direction as output of the given gpio line.
    pub fn direction_output(&mut self, offset: u32) -> Result<()> {
        match Port::from_usize(self.idx) {
            Some(Port::PortA) => Ok(self.regs().GPIO_SWPORTA_DDR.set(self.regs().GPIO_SWPORTA_DDR.get() | ( 1 << offset))), // 1 is output for Port A
            Some(Port::PortB) => Ok(self.regs().GPIO_SWPORTB_DDR.set(self.regs().GPIO_SWPORTB_DDR.get() | ( 1 << offset))), // 1 is output for Port B
            Some(Port::PortC) => Ok(self.regs().GPIO_SWPORTC_DDR.set(self.regs().GPIO_SWPORTC_DDR.get() | ( 1 << offset))), // 1 is output for Port C
            Some(Port::PortD) => Ok(self.regs().GPIO_SWPORTD_DDR.set(self.regs().GPIO_SWPORTD_DDR.get() | ( 1 << offset))), // 1 is output for Port D
            None => to_error(Errno::InvalidArgs),                             // Handle invalid index case     
        }
    }

    /// Returns the current value of the given gpio line.
    pub fn get_value(&mut self, offset: u32) -> Result<u32> {
        match Port::from_usize(self.idx) {
            Some(Port::PortA) => Ok((self.regs().GPIO_EXT_PORTA.get() >> offset) & 1), // Access DR register for Port A
            Some(Port::PortB) => Ok((self.regs().GPIO_EXT_PORTB.get() >> offset) & 1), // Access DR register for Port B
            Some(Port::PortC) => Ok((self.regs().GPIO_EXT_PORTC.get() >> offset) & 1), // Access DR register for Port C
            Some(Port::PortD) => Ok((self.regs().GPIO_EXT_PORTD.get() >> offset) & 1), // Access DR register for Port D
            None => to_error(Errno::InvalidArgs),                             // Handle invalid index case     
        }
    }

    /// Sets the value of the given gpio line.
    pub fn set_value(&mut self, offset: u32, value:u32) -> Result<()> {
        match Port::from_usize(self.idx) {
            Some(Port::PortA) => Ok(self.regs().GPIO_SWPORTA_DR.set(self.regs().GPIO_SWPORTA_DR.get() & !( 1 << offset) | (value << offset))), // Set value for Port A
            Some(Port::PortB) => Ok(self.regs().GPIO_SWPORTB_DR.set(self.regs().GPIO_SWPORTB_DR.get() & !( 1 << offset) | (value << offset))), // Set value for Port B
            Some(Port::PortC) => Ok(self.regs().GPIO_SWPORTC_DR.set(self.regs().GPIO_SWPORTC_DR.get() & !( 1 << offset) | (value << offset))), // Set valuer for Port C
            Some(Port::PortD) => Ok(self.regs().GPIO_SWPORTD_DR.set(self.regs().GPIO_SWPORTD_DR.get() & !( 1 << offset) | (value << offset))), // Set value for Port D
            None => to_error(Errno::InvalidArgs),                             // Handle invalid index case     
        }
    }

    /// GPIO initialize
    pub fn gpio_init(&mut self) {
        // TODO:init gpio
        todo!();
    }

}
