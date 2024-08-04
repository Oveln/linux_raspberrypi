// SPDX-License-Identifier: GPL-2.0

//! serial.
//!
//! This module contains the kernel APIs related to serial that have been ported or
//! wrapped for usage by Rust code in the kernel.

pub mod ktermbits;
pub mod tty;
pub mod uart_console;
pub mod uart_driver;
pub mod uart_port;

pub mod pl011_config;
