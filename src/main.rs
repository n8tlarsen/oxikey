#![no_std]
#![no_main]

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                     // use panic_abort as _; // requires nightly
                     // use panic_itm as _; // logs messages over ITM; requires ITM support
                     // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

pub mod setup;

#[rtic::app(device = atsamd21j, dispatchers = [ADC,AC,DAC])]
mod app {
    #[cfg(debug_assertions)]
    use cortex_m_semihosting::hprintln;

    use crate::setup;
    use atsamd_hal as hal;
    use hal::gpio::*;
    use hal::thumbv6m::{clock, usb::UsbBus};
    use usb_device::{
        bus::UsbBusAllocator,
        device::{UsbDevice, UsbDeviceBuilder},
        prelude::*,
    };
    use usbd_hid::descriptor::{generator_prelude::*, KeyboardReport};
    use usbd_hid::hid_class::{
        HIDClass, HidClassSettings, HidCountryCode, HidProtocol, HidSubClass, ProtocolModeConfig,
    };

    #[shared]
    struct Shared {
        usb_hid: HIDClass<'static, UsbBus>,
    }

    #[local]
    struct Local {
        local_to_foo: i32,
        local_to_bar: i32,
        local_to_idle: i32,
        usb_dev: UsbDevice<'static, UsbBus>,
    }

    #[init(local = [usb_alloc: Option<UsbBusAllocator<UsbBus>> = None])]
    fn init(mut cx: init::Context) -> (Shared, Local, init::Monotonics) {
        foo::spawn().unwrap();
        bar::spawn().unwrap();
        let mut clock = clock::GenericClockController::with_internal_32kosc(
            cx.device.GCLK,
            &mut cx.device.PM,
            &mut cx.device.SYSCTRL,
            &mut cx.device.NVMCTRL,
        );
        let gclk0 = clock.gclk0();
        let usb_clk = clock.usb(&gclk0).unwrap();
        clock.tcc0_tcc1(&gclk0);
        setup::blink(&cx.device.PM, &cx.device.PORT, &cx.device.TCC0);
        let pins = Pins::new(cx.device.PORT);
        *cx.local.usb_alloc = Some(UsbBusAllocator::new(UsbBus::new(
            &usb_clk,
            &mut cx.device.PM,
            pins.pa24,
            pins.pa25,
            cx.device.USB,
        )));
        let usb_hid = HIDClass::new_with_settings(
            cx.local.usb_alloc.as_ref().unwrap(),
            KeyboardReport::desc(),
            60,
            HidClassSettings {
                subclass: HidSubClass::Boot,
                protocol: HidProtocol::Keyboard,
                config: ProtocolModeConfig::ForceBoot,
                locale: HidCountryCode::US,
            },
        );
        let usb_dev = UsbDeviceBuilder::new(
            cx.local.usb_alloc.as_ref().unwrap(),
            UsbVidPid(0x16c0, 0x27dd),
        )
        .manufacturer("n8tlarsen")
        .product("Oxide Keyboard")
        .serial_number("TEST")
        .build();
        (
            Shared { usb_hid },
            Local {
                local_to_foo: 0,
                local_to_bar: 0,
                local_to_idle: 0,
                usb_dev,
            },
            init::Monotonics(),
        )
    }

    #[idle(local = [local_to_idle])]
    fn idle(cx: idle::Context) -> ! {
        let local_to_idle = cx.local.local_to_idle;
        *local_to_idle += 1;
        #[cfg(debug_assertions)]
        hprintln!("idle: local_to_idle = {}", local_to_idle).unwrap();
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(local = [local_to_foo])]
    fn foo(cx: foo::Context) {
        let local_to_foo = cx.local.local_to_foo;
        *local_to_foo += 1;
        #[cfg(debug_assertions)]
        hprintln!("foo: local_to_foo = {}", local_to_foo).unwrap();
    }

    #[task(local = [local_to_bar])]
    fn bar(cx: bar::Context) {
        let local_to_bar = cx.local.local_to_bar;
        *local_to_bar += 1;
        #[cfg(debug_assertions)]
        hprintln!("bar: local_to_bar = {}", local_to_bar).unwrap();
    }

    #[task(binds = USB, local = [usb_dev], shared =[usb_hid])]
    fn usb(mut cx: usb::Context) {
        let dev = cx.local.usb_dev;
        cx.shared.usb_hid.lock(|hid| {
            dev.poll(&mut [hid]);
        });
    }
}
