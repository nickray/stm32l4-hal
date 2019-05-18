//! Flash memory

use crate::stm32::{flash, FLASH};
// use cortex_m_semihosting::hprintln;

//
// Copy-pasta from other HALs
//

// /// Extension trait to constrain the FLASH peripheral
pub trait FlashExt {
    /// Constrains the FLASH peripheral to play nicely with the other abstractions
    fn constrain(self) -> Parts;
}

impl FlashExt for FLASH {
    fn constrain(self) -> Parts {
        Parts {
            acr: ACR { _0: () },
        }
    }
}

/// Constrained FLASH peripheral
pub struct Parts {
    /// Opaque ACR register
    pub acr: ACR,
}

/// Opaque ACR register
pub struct ACR {
    _0: (),
}

impl ACR {
    pub(crate) fn acr(&mut self) -> &flash::ACR {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*FLASH::ptr()).acr }
    }
}


//
//
// from https://github.com/idubrov/stm32-hal/blob/master/src/flash.rs
//
//

pub struct FlashInstance;

use core::result::Result;
use byteorder::ByteOrder;


/// High-level API for the Flash memory
pub trait Flash where Self: Sized {

    fn is_locked(&self) -> bool;
    fn unlock(&self);
    fn lock(&self);

    /// check flash status
    fn status(&self) -> FlashResult;

    /// Unlocks the Flash.
    /// UnlockGuard locks the flash again when it goes out of scope.
    /// Except when flash was already unlocked, it is not unlocked, and won't be locked.
    unsafe fn unlock_guard(&self) -> UnlockResult<Self> {
        let locked = self.is_locked();
        if locked {
            self.unlock();
        }
        Ok(UnlockGuard { flash: self, should_lock: locked })
    }

    /// Erase specified flash page.
    fn erase_page(&self, page: u8) -> FlashResult;

    /// Program half-word (16-bit) value at a specified address. `address` must be an address of
    /// a location in the Flash memory aligned to two bytes.
    fn program_eight_bytes(&self, address: usize, data: [u8; 8]) -> FlashResult;

    fn read_eight_bytes(&self, address: usize) -> [u8; 8];

    /// Erase all Flash pages
    fn erase_all_pages(&self) -> FlashResult;
}

/// Flash operation error
#[derive(Copy, Clone, Debug)]
pub enum FlashError {
    /// Flash program and erase controller failed to unlock
    UnlockFailed,
    /// Timeout while waiting for the completion of the operation
    Timeout,
    /// Address to be programmed contains a value different from '0xFFFF' before programming
    ProgrammingError,
    /// Programming a write-protected address of the Flash memory
    WriteProtectionError,
    /// Programming and erase controller is busy
    Busy
}

/// A type alias for the result of a Flash operation.
pub type FlashResult = Result<(), FlashError>;

/// A type alias for the result of a Flash unlock method.
pub type UnlockResult<'a, FlashT> = Result<UnlockGuard<'a, FlashT>, FlashError>;

/// An RAII implementation of a "scoped unlock" of a Flash. When this structure is dropped (falls
/// out of scope), the Flash will be locked.
pub struct UnlockGuard<'a, FlashT: Flash> where FlashT: 'a {
    flash: &'a FlashT,
    should_lock: bool
}

impl<'a, FlashT: Flash> Drop for UnlockGuard<'a, FlashT> {
    fn drop(&mut self) {
        if self.should_lock {
            self.flash.lock();
        }
    }
}

impl<'a, FlashT: Flash> core::ops::Deref for UnlockGuard<'a, FlashT> {
    type Target = FlashT;

    fn deref(&self) -> &FlashT {
        self.flash
    }
}

// const FLASH_PAGE_SIZE: usize = 2048;
// const FLASH_PAGES: u8 = 128;
// const FLASH_ORIGIN: usize = 0x08000000;

const FLASH_KEY1: u32 = 0x4567_0123;
const FLASH_KEY2: u32 = 0xCDEF_89AB;


impl Flash for FLASH {
    fn is_locked(&self) -> bool {
        self.cr.read().lock().bit_is_set()
    }

    fn status(&self) -> FlashResult {
        let sr = self.sr.read();
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

    fn read_eight_bytes(&self, address: usize) -> [u8; 8] {
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

        buf
    }

    // FLASH only allows writing/reading double words (8 bytes) at a time
    fn program_eight_bytes(&self, address: usize, data: [u8; 8]) -> FlashResult {
        self.status()?;

        // enable programming
        self.cr.modify(|_, w| w.pg().set_bit());

        // write words consecutively
        let first_word = byteorder::NativeEndian::read_u32(&data[..4]);
        let second_word = byteorder::NativeEndian::read_u32(&data[4..]);
        unsafe {
            // Program the first word
            core::ptr::write_volatile(address as *mut u32, first_word);
            // Program the second word
            core::ptr::write_volatile((address + 4) as *mut u32, second_word);
        }

        // wait until done
        while self.sr.read().bsy().bit_is_set() {}

        // disable programming
        self.cr.modify(|_, w| w.pg().clear_bit());

        match self.sr.read().bits() {
            0 => Ok(()),
            _ => Err(FlashError::ProgrammingError),
        }
    }

    // TODO: use critical section
    fn erase_page(&self, page: u8) -> FlashResult {
        self.status()?;

        // enable page erase
        self.cr.modify(|_, w| w.per().set_bit());
        // set page number
        unsafe { self.cr.modify(|_, w| w.pnb().bits(page)); }
        // start erase page
        self.cr.modify(|_, w| w.start().set_bit());
        // wait until done
        while !self.sr.read().bsy().bit_is_clear() {}
        // disable page erase
        self.cr.modify(|_, w| w.per().clear_bit());

        Ok(())
    }

    fn erase_all_pages(&self) -> FlashResult {
        self.status()?;

        // enable mass erase
        self.cr.modify(|_, w| w.mer1().set_bit());
        // start mass erase
        self.cr.modify(|_, w| w.start().set_bit());
        // wait until done
        while !self.sr.read().bsy().bit_is_clear() {}
        // disable mass erase
        self.cr.modify(|_, w| w.mer1().clear_bit());

        Ok(())
    }

    /// unlocks the Flash.
    fn unlock(&self) {
        // ehh.. should check BSY here

        if self.is_locked() {
            unsafe {
                self.keyr.write(|w| w.keyr().bits(FLASH_KEY1));
                self.keyr.write(|w| w.keyr().bits(FLASH_KEY2));
            }
        }
    }

    /// locks the flash
    fn lock(&self) {
        self.cr.modify(|_, w| w.lock().set_bit());
    }
}
