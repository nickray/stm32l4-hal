//! Flash memory

use crate::stm32::FLASH;

#[cfg(feature = "extra-traits")]
use byteorder::ByteOrder;

#[cfg(feature = "extra-traits")]
use crate::hal::flash::{FlashError, FlashResult, Read, WriteErase, Locking};

// #[cfg(feature = "extra-traits")]
// use generic_array::{ArrayLength, GenericArray};

#[allow(dead_code)]
pub struct Flash {
    flash: FLASH
}

impl Flash {
    // the new constructor approach
    pub fn new(flash: FLASH) -> Flash {
        Self {
            flash: flash
        }
    }
}


// inspired by https://github.com/idubrov/stm32-hal/blob/master/src/flash.rs

pub const FLASH_ORIGIN: usize = 0x08000000;

#[cfg(feature = "extra-traits")]
const FLASH_KEY1: u32 = 0x4567_0123;
#[cfg(feature = "extra-traits")]
const FLASH_KEY2: u32 = 0xCDEF_89AB;

#[cfg(all(feature = "stm32l4x2", feature="extra-traits"))]
impl Locking for Flash {
    fn is_locked(&self) -> bool {
        self.flash.cr.read().lock().bit_is_set()
    }

    /// unlocks the Flash.
    fn unlock(&self) {
        // ehh.. should check BSY here
        // peripheral stalls if it's already locked, so check
        if self.is_locked() {
            unsafe {
                self.flash.keyr.write(|w| w.keyr().bits(FLASH_KEY1));
                self.flash.keyr.write(|w| w.keyr().bits(FLASH_KEY2));
            }
        }
    }

    /// locks the flash
    fn lock(&self) {
        self.flash.cr.modify(|_, w| w.lock().set_bit());
    }
}

#[cfg(feature = "stm32l4x2")]
pub const READ_SIZE: usize = 8;
#[cfg(feature = "stm32l4x2")]
pub const WRITE_SIZE: usize = 8;
#[cfg(feature = "stm32l4x2")]
pub const PAGE_SIZE: usize = 2048;

// use itertools::Itertools;

#[cfg(all(feature = "stm32l4x2", feature="extra-traits"))]
impl Flash {
    // fn read_native(&self, address: usize) -> GenericArray<u8, generic_array::typenum::U8> {
    pub fn read_native(&self, address: usize) -> [u8; 8] {
        let mut buf = [0u8; 8];

        unsafe {
            byteorder::NativeEndian::write_u32(
                &mut buf[..4],
                core::ptr::read_volatile(address as *mut u32),
            );
            byteorder::NativeEndian::write_u32(
                &mut buf[4..],
                core::ptr::read_volatile((address + 4) as *mut u32),
            );
        }

        //buf.into()
        buf
    }

    // FLASH only allows writing/reading double words (8 bytes) at a time
    // fn write_native(&self, address: usize,
    //                 data: &mut GenericArray<u8, generic_array::typenum::U8>) -> FlashResult {
    pub fn write_native(&self, address: usize,
                    first_word: u32, second_word: u32) -> FlashResult {
        self.status()?;

        // enable programming
        self.flash.cr.modify(|_, w| w.pg().set_bit());

        // write words consecutively
        unsafe {
            // Program the first word
            core::ptr::write_volatile(address as *mut u32, first_word);
            // Program the second word
            core::ptr::write_volatile((address + 4) as *mut u32, second_word);
        }

        // wait until done
        while self.flash.sr.read().bsy().bit_is_set() {}

        // disable programming
        self.flash.cr.modify(|_, w| w.pg().clear_bit());

        match self.flash.sr.read().bits() {
            0 => Ok(()),
            _ => Err(FlashError::ProgrammingError),
        }
    }

}

#[cfg(all(feature = "stm32l4x2", feature="extra-traits"))]
// impl Read<generic_array::typenum::U8> for Flash {
impl Read for Flash {
    // TODO: move this in trait definition itself?
    fn read(&self, address: usize, buf: &mut [u8]) {
        // let's hope this is optimized away in release builds
        assert!(buf.len() % 8 == 0);
        assert!(address % 8 == 0);

        unsafe {
            for i in (0..buf.len()).step_by(4) {
                byteorder::NativeEndian::write_u32(
                    &mut buf[i..i + 4],
                    core::ptr::read_volatile((address + i) as *mut u32),
                );
            }
        }
    }

}

#[cfg(all(feature = "stm32l4x2", feature="extra-traits"))]
impl WriteErase for Flash {
    fn status(&self) -> FlashResult {
        let sr = self.flash.sr.read();
        if sr.bsy().bit_is_set() {
            Err(FlashError::Busy)
        } else if sr.progerr().bit_is_set() {
            Err(FlashError::ProgrammingError)
        } else if sr.wrperr().bit_is_set() {
            Err(FlashError::WriteProtectionError)
        } else {
            Ok(())
        }
    }

    fn write(&self, address: usize, data: &[u8]) -> FlashResult {
        // let's hope this is optimized away in release builds
        assert!(data.len() % 8 == 0);
        assert!(address % 8 == 0);

        for i in (0..data.len()).step_by(8) {
            let first_word = byteorder::NativeEndian::read_u32(&data[i..i + 4]);
            let second_word = byteorder::NativeEndian::read_u32(&data[i + 4..i + 8]);
            self.write_native(address + i, first_word, second_word)?;
        }

        Ok(())
    }

    // TODO: use critical section
    fn erase_page(&self, page: u8) -> FlashResult {
        self.status()?;

        // enable page erase
        self.flash.cr.modify(|_, w| w.per().set_bit());
        // set page number
        unsafe { self.flash.cr.modify(|_, w| w.pnb().bits(page)); }
        // start erase page
        self.flash.cr.modify(|_, w| w.start().set_bit());
        // wait until done
        while !self.flash.sr.read().bsy().bit_is_clear() {}
        // disable page erase
        self.flash.cr.modify(|_, w| w.per().clear_bit());

        Ok(())
    }

    fn erase_all_pages(&self) -> FlashResult {
        self.status()?;

        // enable mass erase
        self.flash.cr.modify(|_, w| w.mer1().set_bit());
        // start mass erase
        self.flash.cr.modify(|_, w| w.start().set_bit());
        // wait until done
        while !self.flash.sr.read().bsy().bit_is_clear() {}
        // disable mass erase
        self.flash.cr.modify(|_, w| w.mer1().clear_bit());

        Ok(())
    }

}
