// SPDX-License-Identifier: GPL-2.0
//! Rust dw_apb_gpio

#![no_std]

use core::ops::DerefMut;
use kernel::{
    prelude::*, 
    module_platform_driver,
    c_str, 
    of, 
    platform,
    error,
    gpio,
    device,
    fwnode::FwNodeHandle,
    device::Device,
    sync::{Arc,ArcBorrow},
};

use dw_apb_gpio::{ 
    DwGpioPort, 
    DwGpio, 
    DWAPB_MAX_PORTS, 
    DWAPB_MAX_GPIOS,
};

module_platform_driver! {
      type: DwGpioDriver,
      name: "gpio_designware",
      license: "GPL",
      initcall: "device",
}

// Linux Raw id table
kernel::module_of_id_table!(DW_GPIO_MOD_TABLE, DW_GPIO_OF_MATCH_TABLE);
// R4L IdArray table
kernel::define_of_id_table! {DW_GPIO_OF_MATCH_TABLE, (), [
    (of::DeviceId::Compatible(b"snps,dw-apb-gpio"), None),
]}

/// Due to the limitation of #[vtable], a Struct GpioPort wrapper is needed to encapsulate DwGpioPort.
#[derive(Clone)]
struct GpioPort {
    dw_gpio_port: DwGpioPort,
}

type PortRegistrations = gpio::Registration<GpioPort>;
type PortData = device::Data<PortRegistrations, (), GpioPort>;
type DeviceData = device::Data<(), (), DwGpio>;

struct DwGpioDriver;
impl platform::Driver for DwGpioDriver {
    // Linux Raw id table
    kernel::driver_of_id_table!(DW_GPIO_OF_MATCH_TABLE);
    type Data = Arc<DeviceData>;
    
    fn probe(pdev: &mut platform::Device, _id_info: Option<&Self::IdInfo>) -> Result<Self::Data> {
        dev_info!(pdev,"{} driver in Rust (probe)\n",pdev.name());
        let reg_base:*mut u8 = pdev.ioremap_resource(0)?;
        let dev = Device::from_dev(pdev);
        let dev_fwnode = FwNodeHandle::from(&dev).unwrap();
        let nports:usize = FwNodeHandle::child_count(dev_fwnode.clone())?;
            if nports > DWAPB_MAX_PORTS.try_into().unwrap() {
                return Err(error::code::EINVAL);
            }
        let mut gpio_prvdata = DwGpio::new(reg_base);

        for child in FwNodeHandle::children(dev_fwnode.clone()) {
            let idx: usize = FwNodeHandle::fwnode_property_read_u32(child.clone(), c_str!("reg"))?.try_into().unwrap();
            let mut ngpio: u32 = FwNodeHandle::fwnode_property_read_u32(child.clone(), c_str!("ngpios"))?;
                if ngpio == 0 {
                    ngpio = FwNodeHandle::fwnode_property_read_u32(child.clone(), c_str!("snps,nr-gpios"))?;
                    dev_err!(dev, "bst gpio port get nrgpio {} form snps,nr-gpios \n", ngpio);
                    if ngpio == 0 {
                        pr_info!("bst gpio port get ngpio error\n");
                        return Err(error::code::EINVAL);
                    }
                }
                if ngpio > DWAPB_MAX_GPIOS { ngpio = DWAPB_MAX_GPIOS;}

            let mut gpio_base: u32 = FwNodeHandle::fwnode_property_read_u32(child.clone(), c_str!("chipnum-base"))?;
                if FwNodeHandle::is_softnode(dev_fwnode.clone()) {
                    gpio_base = FwNodeHandle::fwnode_property_read_u32(child.clone(), c_str!("gpio-base"))?;
                }
            let port = GpioPort{ dw_gpio_port: DwGpioPort::new(reg_base, idx) };

            gpio_prvdata.set_port(idx, port.dw_gpio_port);
            
            let gpio_portdata = kernel::new_device_data!(
                gpio::Registration::<GpioPort>::new(),
                (),
                port,
                "port Registrations"
            )?;

            let arc_portdata:Arc<PortData> = Arc::<PortData>::from(gpio_portdata);

            // Register gpio chips for each port
            kernel::gpio_chip_register!(
                unsafe {Pin::new_unchecked(arc_portdata.registrations().ok_or(ENXIO)?.deref_mut()) },
                ngpio as u16,
                Some(gpio_base.try_into().unwrap()),
                pdev,
                arc_portdata.clone(),
            )?;

        }

        let devdata = kernel::new_device_data!(
            (),
            (),
            gpio_prvdata,
            "BST::Registrations"
        )?; 

        pr_info!("DW-APB-GPIO driver in Rust (probe)\n");
        Ok(devdata.into())
    }
}

impl Drop for DwGpioDriver {
    fn drop(&mut self) {
        pr_info!("DW-APB-GPIO driver in Rust (exit)\n");
    }
}

#[vtable]
impl gpio::Chip for GpioPort {
    type Data = Arc<PortData>;

    fn get_direction(data: ArcBorrow<'_, PortData>, offset: u32) -> Result<gpio::LineDirection> {
        pr_info!("bst gpio get_direction is supported. offset is {}\n",offset);
        let mut gpio = data.dw_gpio_port;
        let res = gpio.get_direction(offset).unwrap();
        if res == 1 { return Ok(gpio::LineDirection::Out); }
        else { return Ok(gpio::LineDirection::In); }
    }

    fn direction_input(data: ArcBorrow<'_, PortData>, offset: u32) -> Result {
        pr_info!("bst gpio direction_input is supported.\n");
        let mut gpio = data.dw_gpio_port;
        gpio.direction_input(offset)
    }

    fn direction_output(data: ArcBorrow<'_, PortData>, offset: u32, _value: bool) -> Result {
        pr_info!("bst gpio direction_output is supported.\n");
        let mut gpio = data.dw_gpio_port;
        gpio.direction_output(offset)
    }

    fn get(data: ArcBorrow<'_, PortData>, offset: u32) -> Result<bool> {
        pr_info!("bst gpio get_value is supported.\n");
        let mut gpio = data.dw_gpio_port;
        Ok(gpio.get_value(offset).unwrap() !=0)
    }

    fn set(data: ArcBorrow<'_, PortData>, offset: u32, value: bool) {
        pr_info!("bst gpio set_value is supported.\n");
        let mut gpio = data.dw_gpio_port;
        let bitmask = if value { 1 } else { 0 };
        let _ = gpio.set_value(offset, bitmask);
    }
}
