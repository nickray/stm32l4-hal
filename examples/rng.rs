#![no_std]
#![no_main]

// pick a panicking behavior
extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics

use core::fmt;
use cortex_m_rt::entry;

use stm32l4_hal as hal;
use crate::hal::stm32;
use crate::hal::prelude::*;
use crate::hal::delay::Delay;
use crate::hal::serial::Serial;

macro_rules! uprint {
    ($serial:expr, $($arg:tt)*) => {
        fmt::write($serial, format_args!($($arg)*)).ok()
    };
}

macro_rules! uprintln {
    ($serial:expr, $fmt:expr) => {
        uprint!($serial, concat!($fmt, "\n"))
    };
    ($serial:expr, $fmt:expr, $($arg:tt)*) => {
        uprint!($serial, concat!($fmt, "\n"), $($arg)*)
    };
}

#[entry]
fn main() -> ! {

    let core = cortex_m::Peripherals::take().unwrap();
    let device = stm32::Peripherals::take().unwrap();

    let mut flash = device.FLASH.constrain();
    let mut rcc = device.RCC.constrain();

    let clocks = rcc.cfgr
        .hsi48(true)  // needed for RNG + USB
        // why does clock configuration need flash?
        // --> because flash read latency has to be set accordingly!
        .freeze(&mut flash.acr);

    // setup usart
    let mut gpioa = device.GPIOA.split(&mut rcc.ahb2);
    let tx = gpioa.pa9.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
    let rx = gpioa.pa10.into_af7(&mut gpioa.moder, &mut gpioa.afrh);

    let baud_rate = 9_600;  // 115_200;
    let serial = Serial::usart1(
        device.USART1,
        (tx, rx),
        baud_rate.bps(),
        clocks,
        &mut rcc.apb2
    );
    let (mut tx, _) = serial.split();

    // uprintln!(&mut tx, "clocks: {:?}", clocks);
    // uprintln!(&mut tx, "is hsi48 on {}", rcc.crrcr.is_hsi48on());

    // setup led
    let mut gpiob = device.GPIOB.split(&mut rcc.ahb2);
    let mut led_pin = gpiob.pb3.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    let mut timer = Delay::new(core.SYST, clocks);

    // setup rng
    let mut rng = device.RNG.enable(&mut rcc.ahb2, clocks);

    // HUH?! why can RNG.enable even use reference to clocks, after Delay::new consumes them?!
    // hmm... maybe because of the #[derive(Clone, Copy)]

    let short_time: u32 = 100;
    let long_time: u32 = 800;

    // let mut i: u64 = 0;
    loop {
        // play with the LED
        for _ in 0..5 {
            led_pin.set_high();
            timer.delay_ms(short_time);

            led_pin.set_low();
            timer.delay_ms(short_time);
        }

        // print some stuff to USART
        // uprintln!(&mut tx,
        //     "iteration {} :: rng enabled {}, clock error {}, seed error {}, data ready {}",
        //     i,
        //     rng.is_enabled(),
        //     rng.is_clock_error(),
        //     rng.is_seed_error(),
        //     rng.is_data_ready(),
        // );

        // output some random data
        const N: usize = 5;
        let mut random_bytes = [0u8; N];
        rng.read(&mut random_bytes).expect("missing random data for some reason");
        uprintln!(
            &mut tx,
            "{} random u8 values: {:?}",
            N, random_bytes
        );

        timer.delay_ms(long_time);
        // i += 1;
    }
}

