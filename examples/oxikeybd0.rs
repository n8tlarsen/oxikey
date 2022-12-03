#![no_std]
#![no_main]

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                     // use panic_abort as _; // requires nightly
                     // use panic_itm as _; // logs messages over ITM; requires ITM support
                     // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

#[rtic::app(device = atsamd21g, dispatchers = [ADC,AC,DAC])]
mod app {
    #[cfg(debug_assertions)]
    use cortex_m_semihosting::hprintln;

    use atsamd_hal as hal;
    use hal::gpio::*;
    use hal::thumbv6m::{clock, usb::UsbBus};
    use hal::time::Hertz;
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
        keys: [oxikey::Debounce<8>; 21],
        local_to_bar: i32,
        local_to_idle: i32,
        usb_dev: UsbDevice<'static, UsbBus>,
    }

    #[init(local = [usb_alloc: Option<UsbBusAllocator<UsbBus>> = None])]
    fn init(mut cx: init::Context) -> (Shared, Local, init::Monotonics) {
        bar::spawn().unwrap();
        // Setup Clocks
        let mut clock = clock::GenericClockController::with_internal_32kosc(
            cx.device.GCLK,
            &mut cx.device.PM,
            &mut cx.device.SYSCTRL,
            &mut cx.device.NVMCTRL,
        );
        let gclk0 = clock.gclk0();
        cx.core.SYST.set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);
        cx.core.SYST.set_reload((<hal::clock::GClock as Into<Hertz>>::into(gclk0).0)/2000u32);
        let usb_clk = clock.usb(&gclk0).unwrap();
        // Setup Input Pins
        let pins = Pins::new(cx.device.PORT);
        pins.pa15.into_pull_up_input(); // outer top
        pins.pa14.into_pull_up_input(); // outer home
        pins.pa13.into_pull_up_input(); // outer bottom
        pins.pa12.into_pull_up_input(); // pinky top
        pins.pa11.into_pull_up_input(); // pinky home
        pins.pa10.into_pull_up_input(); // pinky bottom
        pins.pa09.into_pull_up_input(); // ring top
        pins.pa07.into_pull_up_input(); // ring home
        pins.pa06.into_pull_up_input(); // ring bottom
        pins.pa05.into_pull_up_input(); // middle top
        pins.pa04.into_pull_up_input(); // middle home
        pins.pa03.into_pull_up_input(); // middle bottom
        pins.pa02.into_pull_up_input(); // index top
        pins.pa01.into_pull_up_input(); // index home
        pins.pa28.into_pull_up_input(); // index bottom
        pins.pa20.into_pull_up_input(); // inner top
        pins.pa19.into_pull_up_input(); // inner home
        pins.pa18.into_pull_up_input(); // inner bottom
        pins.pa08.into_pull_up_input(); // near thumb
        pins.pa16.into_pull_up_input(); // home thumb
        pins.pa17.into_pull_up_input(); // far  thumb
        // Setup USB
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
            UsbVidPid(0x1209, 0x0001),
        )
        .manufacturer("n8tlarsen")
        .product("Oxide Keyboard")
        .serial_number("TEST")
        .build();
        // Return Setup
        (
            Shared { usb_hid },
            Local {
                keys: Default::default(),
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

    #[task(local = [local_to_bar])]
    fn bar(cx: bar::Context) {
        let local_to_bar = cx.local.local_to_bar;
        *local_to_bar += 1;
        #[cfg(debug_assertions)]
        hprintln!("bar: local_to_bar = {}", local_to_bar).unwrap();
    }

    #[task(binds = SysTick, local = [keys])]
    fn get_keys(cx: get_keys::Context) {
        // Safe because this is a read within interrupt context
        let mut port_a_in = unsafe { (*atsamd21g::PORT::PTR).in0.read().bits() };
        let mut keys: [bool; 21] = Default::default();
        for i in 0..32 {
            match i {
                1..=20 => {
                    keys[i-i] = cx.local.keys[i-1].sample((port_a_in & 0x1u32) == 1);
                }
                28 => {
                    keys[20] = cx.local.keys[20].sample((port_a_in & 0x1u32) == 1);
                }
                _ => (),
            }
            port_a_in = port_a_in >> 1;
        }
        
    }

    #[task(binds = USB, local = [usb_dev], shared =[usb_hid])]
    fn usb(mut cx: usb::Context) {
        let dev = cx.local.usb_dev;
        cx.shared.usb_hid.lock(|hid| {
            dev.poll(&mut [hid]);
        });
    }
}
