use generic_array::{ArrayLength, GenericArray};

// TODOS:
// - introduce Locked/Unlocked states so that `.unlock()`
//   returns an Unlocked type on which write/erase can be called,
//   whereas the Locked type lacks thes methods
// - alternatively, write/erase could automatically unlock,
//   but seems nicer to be explicit about the locking
// - FlashErrors Busy and UnlockFailed should not occur
//   with the right API
// - move to extra_traits/blocking
// - extend to OptionBytesLocked/Unlocked
// - seems there is no compile time way to ensure read/write is
//   done only for multiples of the native READ/WRITE_SIZEs?

pub struct Locked;
pub struct Unlocked;

// TODO: The idea is that `Flash::new()` returns a `Locked` `flash`
// that implements `Read` and `Unlock` traits.
// Then `flash.unlock()` returns an `Unlocked` flash, which implements
// `Read`, `WriteErase` and `Lock` traits.
// Calling `flash.lock()` reverts the `flash` to its previous state.
//
// Need to make sure the original flash instances is actually locked,
// so the state machine does not start out of whack with reality.
pub enum FlashStates {
    Locked,
    Unlocked,
}

pub trait Read<ReadSize: ArrayLength<u8>> {
    /// for HALs to implement
    /// e.g. if FLASH can be read in as double 32 bit words (blocks of 8 bytes):
    ///
    ///     impl Read<generic_array::typenum::U8> for Flash {
    ///         fn read_native(...);
    ///     }
    ///
    /// TODO: can we typecheck/typehint whether `address` must be aligned?
    fn read_native(&self, address: usize, array: &mut GenericArray<u8, ReadSize>);

    /// read a buffer of bytes from memory
    /// checks that the address and buffer size are multiples of native
    /// FLASH ReadSize.
    fn read(&self, address: usize, buf: &mut [u8]) {
        // TODO: offer a version without restrictions?
        // can round down address, round up buffer length,
        // but where to get the buffer from?
        assert!(buf.len() % ReadSize::to_usize() == 0);
        assert!(address % ReadSize::to_usize() == 0);

        for i in (0..buf.len()).step_by(8) {
            self.read_native(address + i, GenericArray::from_mut_slice(&mut buf[i..i + 8]));
        }
    }
}

pub trait WriteErase<EraseSize: ArrayLength<u8>, WriteSize: ArrayLength<u8>> {

    /// check flash status
    fn status(&self) -> FlashResult;

    /// Erase specified flash page.
    fn erase_page(&self, page: u8) -> FlashResult;

    /// The smallest possible write, depends on platform
    /// TODO: can we typecheck/typehint whether `address` must be aligned?
    fn write_native(&self, address: usize, array: &GenericArray<u8, WriteSize>) -> FlashResult;

    fn write(&self, address: usize, data: &[u8]) -> FlashResult {
        assert!(data.len() % WriteSize::to_usize() == 0);
        assert!(address % WriteSize::to_usize() == 0);

        for i in (0..data.len()).step_by(8) {
            self.write_native(address + i, GenericArray::from_slice(&data[i..i + 8]))?;
        }

        Ok(())
    }

    // probably not so useful, as only applicable after mass erase
    // /// Faster programming
    // fn program_sixtyfour_bytes(&self, address: usize, data: [u8; 64]) -> FlashResult {

    /// Erase all Flash pages
    fn erase_all_pages(&self) -> FlashResult;
}


// pub type UnlockResult<'a, FlashT> = Result<UnlockGuard<'a, FlashT>, FlashError>;

pub struct UnlockGuard<'a, FlashT: Locking> where FlashT: 'a {
    flash: &'a FlashT,
    should_lock: bool
}

impl<'a, FlashT: Locking> Drop for UnlockGuard<'a, FlashT> {
    fn drop(&mut self) {
        if self.should_lock {
            self.flash.lock();
        }
    }
}

impl<'a, FlashT: Locking> core::ops::Deref for UnlockGuard<'a, FlashT> {
    type Target = FlashT;

    fn deref(&self) -> &FlashT {
        self.flash
    }
}

pub trait Locking where Self: Sized {
    fn is_locked(&self) -> bool;
    fn unlock(&self);
    fn lock(&self);

    fn unlock_guard(&self) -> UnlockGuard<Self> {
        let locked = self.is_locked();
        // unlocking an unlocked flash stalls...
        if locked {
            self.unlock();
        }
        UnlockGuard { flash: self, should_lock: locked }
    }
}

/// Flash operation error
#[derive(Copy, Clone, Debug)]
pub enum FlashError {
    /// Flash program and erase controller failed to unlock
    UnlockFailed,
    /// Address to be programmed contains a value different from '0xFFFF' before programming
    ProgrammingError,
    /// Programming a write-protected address of the Flash memory
    WriteProtectionError,
    /// Programming and erase controller is busy
    Busy
}

/// A type alias for the result of a Flash operation.
pub type FlashResult = Result<(), FlashError>;

// pub trait FlashOps: Locking + WriteErase + Read {}
