// SPDX-License-Identifier: GPL-2.0

//! I2c kernel interface
//!
//! C header: [`include/linux/timekeeping.h`]

use crate::bindings;


/// Get Ktime
pub fn ktime_get() -> u64 {
    // SAFETY: call ffi and ptr is valid
    unsafe{bindings::ktime_get().try_into().unwrap()}
}

