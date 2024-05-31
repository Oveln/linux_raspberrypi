// SPDX-License-Identifier: GPL-2.0

#![no_std]

use core::ops::Deref;
use kernel::{
    file::{self, File},
    io_buffer::{IoBufferReader, IoBufferWriter},
    miscdev, new_mutex,
    prelude::*,
    sync::{Arc, ArcBorrow, Mutex},
    error::code,
};
use led_pure_driver::Led;

module! {
    type: RustLed,
    name: "rust_led_adapter",
    author: "Oveln",
    description: "Rust",
    license: "GPL",
}

#[pin_data]
struct LedData {
    #[pin]
    led: Mutex<Led>,
}

impl LedData {
    fn try_new() -> Result<Arc<Self>> {
        pr_info!("Led device created\n");
        Ok(Arc::pin_init(pin_init!(Self {
            led <- new_mutex!(Led::new())
        }))?)
    }
}

struct RustFile;

#[vtable]
impl file::Operations for RustFile {
    type Data = Arc<LedData>;
    type OpenData = Arc<LedData>;

    fn open(shared: &Arc<LedData>, _file: &file::File) -> Result<Self::Data> {
        pr_info!("open in led device\n",);

        return Ok(shared.clone());
    }

    fn read(
        shared: ArcBorrow<'_, LedData>,
        _file: &File,
        writer: &mut impl IoBufferWriter,
        offset: u64,
    ) -> Result<usize> {
        Ok(0)
    }

    fn write(
        shared: ArcBorrow<'_, LedData>,
        _file: &File,
        reader: &mut impl IoBufferReader,
        offset: u64,
    ) -> Result<usize> {
        let mut led = shared.deref().led.lock();
        let input = reader.read_all()?;
        if input.len() != 2 {
            return Err(EINVAL);
        }
        match input[0] {
            b'0' => led.off(),
            b'1' => led.on(),
            _ => return Err(EINVAL),
        }
        Ok(input.len())
    }

    fn release(_data: Self::Data, _file: &File) {
        pr_info!("release in led device\n");
    }
}

struct RustLed {
    _dev: Pin<Box<miscdev::Registration<RustFile>>>,
}

impl kernel::Module for RustLed {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust Led init\n");

        let data: Arc<LedData> = LedData::try_new()?;

        let reg = miscdev::Registration::new_pinned(fmt!("rust_led"), data)?;

        Ok(RustLed { _dev: reg })
    }
}

impl Drop for RustLed {
    fn drop(&mut self) {
        pr_info!("Rust Led exit\n");
    }
}
