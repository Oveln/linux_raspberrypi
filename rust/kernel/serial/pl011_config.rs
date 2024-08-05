// SPDX-License-Identifier: GPL-2.0

//! This is Pl011 ARM Device Data

/// Define constants for UART registers
// Data read or written from the interface
pub const UART01X_DR: u32 = 0x00;
// Receive status register (Read)
pub const UART01X_RSR: u32 = 0x04;
// Error clear register (Write)
pub const UART01X_ECR: u32 = 0x04;
// Line control register, high byte
pub const UART010_LCRH: u32 = 0x08;
// DMA watermark configure register
pub const ST_UART011_DMAWM: u32 = 0x08;
// Line control register, middle byte
pub const UART010_LCRM: u32 = 0x0C;
// Timeout period register
pub const ST_UART011_TIMEOUT: u32 = 0x0C;
// Line control register, low byte
pub const UART010_LCRL: u32 = 0x10;
// Control register
pub const UART010_CR: u32 = 0x14;
// Flag register (Read only)
pub const UART01X_FR: u32 = 0x18;
// Interrupt identification register (Read)
pub const UART010_IIR: u32 = 0x1C;
// Interrupt clear register (Write)
pub const UART010_ICR: u32 = 0x1C;
// Rx line control register
pub const ST_UART011_LCRH_RX: u32 = 0x1C;
// IrDA low power counter register
pub const UART01X_ILPR: u32 = 0x20;
// Integer baud rate divisor register
pub const UART011_IBRD: u32 = 0x24;
// Fractional baud rate divisor register
pub const UART011_FBRD: u32 = 0x28;
// Line control register
pub const UART011_LCRH: u32 = 0x2c;
// Tx Line control register
pub const ST_UART011_LCRH_TX: u32 = 0x2c;
// Control register
pub const UART011_CR: u32 = 0x30;
// Interrupt fifo level select
pub const UART011_IFLS: u32 = 0x34;
// Interrupt mask
pub const UART011_IMSC: u32 = 0x38;
// Raw interrupt status
pub const UART011_RIS: u32 = 0x3c;
// Masked interrupt status
pub const UART011_MIS: u32 = 0x40;
// Interrupt clear register
pub const UART011_ICR: u32 = 0x44;
// DMA control register
pub const UART011_DMACR: u32 = 0x48;
// XON/XOFF control register
pub const ST_UART011_XFCR: u32 = 0x50;
// XON1 register
pub const ST_UART011_XON1: u32 = 0x54;
// XON2 register
pub const ST_UART011_XON2: u32 = 0x58;
// XON1 register
pub const ST_UART011_XOFF1: u32 = 0x5C;
// XON2 register
pub const ST_UART011_XOFF2: u32 = 0x60;
// Integration test control register
pub const ST_UART011_ITCR: u32 = 0x80;
// Integration test input register
pub const ST_UART011_ITIP: u32 = 0x84;
// Autobaud control register
pub const ST_UART011_ABCR: u32 = 0x100;
// Autobaud interrupt mask/clear register
pub const ST_UART011_ABIMSC: u32 = 0x15C;

// Constants for UART011 IFLS (Interrupt FIFO Level Select) register
pub const UART011_IFLS_RX1_8: u32 = 0 << 3;
pub const UART011_IFLS_RX2_8: u32 = 1 << 3;
pub const UART011_IFLS_RX4_8: u32 = 2 << 3;
pub const UART011_IFLS_RX6_8: u32 = 3 << 3;
pub const UART011_IFLS_RX7_8: u32 = 4 << 3;
pub const UART011_IFLS_TX1_8: u32 = 0 << 0;
pub const UART011_IFLS_TX2_8: u32 = 1 << 0;
pub const UART011_IFLS_TX4_8: u32 = 2 << 0;
pub const UART011_IFLS_TX6_8: u32 = 3 << 0;
pub const UART011_IFLS_TX7_8: u32 = 4 << 0;

pub const UART011_FR_RI: u32 = 0x100;
pub const UART011_FR_TXFE: u32 = 0x080;
pub const UART011_FR_RXFF: u32 = 0x040;
pub const UART01X_FR_TXFF: u32 = 0x020;
pub const UART01X_FR_RXFE: u32 = 0x010;
pub const UART01X_FR_BUSY: u32 = 0x008;
pub const UART01X_FR_DCD: u32 = 0x004;
pub const UART01X_FR_DSR: u32 = 0x002;
pub const UART01X_FR_CTS: u32 = 0x001;

pub const UART011_CR_CTSEN: u32 = 0x8000;
pub const UART01X_CR_UARTEN: u32 = 0x0001;
pub const UART011_CR_TXE: u32 = 0x0100;

#[derive(Default, Copy, Clone)]
pub struct VendorData {
    pub ifls: u32,
    pub fr_busy: u32,
    pub fr_dsr: u32,
    pub fr_cts: u32,
    pub fr_ri: u32,
    pub inv_fr: u32,
    pub access_32b: bool,
    pub oversampling: bool,
    pub dma_threshold: bool,
    pub cts_event_workaround: bool,
    pub always_enabled: bool,
    pub fixfixed_options: bool,
}
