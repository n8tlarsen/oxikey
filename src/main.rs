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
    use cortex_m_semihosting::{hprintln};
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
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        foo::spawn().unwrap();
        bar::spawn().unwrap();
        setup::set_clocks(&cx.device);
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
        hprintln!("idle: local_to_idle = {}", local_to_idle).unwrap();
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(local = [local_to_foo])]
    fn foo(cx: foo::Context) {
        let local_to_foo = cx.local.local_to_foo;
        *local_to_foo += 1;
        hprintln!("foo: local_to_foo = {}", local_to_foo).unwrap();
    }

    #[task(local = [local_to_bar])]
    fn bar(cx: bar::Context) {
        let local_to_bar = cx.local.local_to_bar;
        *local_to_bar += 1;
        hprintln!("bar: local_to_bar = {}", local_to_bar).unwrap();
    }
}
