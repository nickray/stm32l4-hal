#![no_std]
#![no_main]
#![cfg(all(feature = "stm32l4x2", feature = "extra-traits"))]

// extern crate panic_halt;
extern crate panic_semihosting;

use cortex_m_rt::entry;
use stm32l4xx_hal::{prelude::*, stm32};
use stm32l4xx_hal as hal;

use cortex_m_semihosting::hprintln;
use byteorder::ByteOrder;

#[entry]
fn main() -> ! {
    // setup
    let dp = stm32::Peripherals::take().unwrap();
    // let rcc = dp.RCC.constrain();
    // let clocks = rcc.cfgr
    //     // .sysclk(48.mhz())
    //     .freeze();
    // hprintln!("clocks = {:?}", clocks).unwrap();

    // let's go!
    let flash = hal::flash::Flash::new(dp.FLASH);

    let boot_bits = flash.get_boot_bits();
    hprintln!("boot_bits = {:?}", boot_bits).unwrap();

    flash.unlock();

    let page = 100usize;
    flash
        .erase_page(page as u8);
        // .expect("could not erase page {}", page);

    let faddr = 0x800_0000 + page*2048;
    let (word1, word2) = (1u32, 2u32);
    flash
        .write_native(faddr, word1, word2);
        // .expect("could not write to flash address {}", faddr);

    let buf = flash.read_native(faddr);
    let read = (
        byteorder::NativeEndian::read_u32(&buf[..4]),
        byteorder::NativeEndian::read_u32(&buf[4..]),
    );
    assert_eq!(
        (word1, word2), read
    );
    hprintln!("success: wrote {:?}, read {:?}", (word1, word2), read).unwrap();

    loop {}
}

