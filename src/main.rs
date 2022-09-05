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
    use cortex_m_semihosting::{hprintln};
    
    use atsamd_hal::thumbv6m::{clock};
    use atsamd21j::gclk::clkctrl::GEN_A::GCLK1;
    use atsamd21j::gclk::genctrl::SRC_A::DFLL48M;
    use crate::setup;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        local_to_foo: i32,
        local_to_bar: i32,
        local_to_idle: i32,
    }

    #[init]
    fn init(mut cx: init::Context) -> (Shared, Local, init::Monotonics) {
        foo::spawn().unwrap();
        bar::spawn().unwrap();
        let mut clock = clock::GenericClockController::with_internal_8mhz(
            cx.device.GCLK, &mut cx.device.PM, &mut cx.device.SYSCTRL, &mut cx.device.NVMCTRL);
        let clk1 = clock.configure_gclk_divider_and_source(GCLK1,1,DFLL48M,false).unwrap();
        clock.tcc0_tcc1(&clk1);
        setup::blink(&cx.device.PM, &cx.device.PORT, &cx.device.TCC0);
        (
            Shared {},
            Local {
                local_to_foo: 0,
                local_to_bar: 0,
                local_to_idle: 0,
            },
            init::Monotonics(),
        )
    }

    #[idle(local = [local_to_idle])]
    fn idle(cx: idle::Context) -> !{
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
}
