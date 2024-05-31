# GPIO-dw-linux-adapter
Under the cross-kernel driver framework, the Linux driver adaptation layer implemented for designware apb GPIO

## Clone repo
In your linux directory, clone this project

```shell
cd `path to your kernel`/drivers/gpio
git clone git@github.com:happy-thw/gpio-dw-linux-adapter.git
```

## Linux support Cargo 
The cross-kernel driver framework follows a componentized design and uses cargo to resolve component dependencies,
so it is necessary to add R4L support for cargo construction.
reference link: https://github.com/guoweikang/osl


## Add Makefile for adapter dir

Add this line into linux/drivers/gpio/Makefile
``` shell
obj-$(CONFIG_GPIO_DWAPB_RUST)	+= gpio-dw-linux-adapter/
```

you can also replace `CONFIG_RUST` with your own defin, like this from original `GPIO_DWAPB` Kconfig

```shell
config GPIO_DWAPB_RUST
	tristate "Synopsys DesignWare APB GPIO driver in RUST"
	depends on RUST && !GPIO_DWAPB
	select GPIO_GENERIC
	select GPIOLIB_IRQCHIP
	help
	  Say Y or M here to build support for the Synopsys DesignWare APB
	  GPIO block.   
```

**note**: if you want to use RUST driver,remeber disable C driver

