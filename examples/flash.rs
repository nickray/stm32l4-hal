#![no_std]
#![no_main]
#![cfg(all(feature = "stm32l4x2", feature = "extra-traits"))]

// extern crate panic_halt;
extern crate panic_semihosting;

use cortex_m_rt::entry;
use stm32l4xx_hal::{prelude::*, stm32};
use stm32l4xx_hal as hal;

use cortex_m_semihosting::hprintln;
// use byteorder::ByteOrder;

#[entry]
fn main() -> ! {
    // setup
    let dp = stm32::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr
        .hsi48(true)
        // .sysclk(48.mhz())
        .freeze();
    // hprintln!("clocks = {:?}", clocks).unwrap();

    // let's go!
    let flash = hal::flash::Flash::new(dp.FLASH);

    // let boot_bits = flash.get_boot_bits();
    // hprintln!("boot_bits = {:?}", boot_bits).unwrap();

    let mut rng = hal::rng::Rng::new(dp.RNG, clocks);
    const TEST_SIZE: usize = 32;
    let mut random_test_data = [0u8; TEST_SIZE];
    rng
        .read(&mut random_test_data)
        .expect("could not get entropy from RNG peripheral");

    flash.unlock();

    let page = 100usize;
    flash
        .erase_page(page as u8)
        .expect("could not erase page");

    let faddr = 0x800_0000 + page*2048;
    // let test_data = [1u8, 2, 3, 4, 0xF, 0xB, 0xC, 0xD];
    flash
        // .write_native(faddr, &random_test_data)
        .write(faddr, &random_test_data)
        .expect("could not write to flash address");

    let mut buf = [0u8; TEST_SIZE];
    // flash.read_native(faddr, &mut buf);
    flash.read(faddr, &mut buf);
    assert_eq!(random_test_data, buf);
    hprintln!("successfully wrote/read {:?}", random_test_data).unwrap();

    loop {}
}

