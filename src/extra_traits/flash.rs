// use generic_array::{ArrayLength, GenericArray};

// TODOS:
// - resurrect read/write-native with GenericArray values
//   so read/write (general) can be implemented generically
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

// Is there any point in `pub trait XXX where Self: Sized`?

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

// pub trait Read<N: ArrayLength<u8>> {
pub trait Read {
    // - only useful if `.read()` can be generically implemented
    //   with zero cost in terms of `.read_native()
    // fn read_native(&self, address: usize, buf: &mut GenericArray<u8, N>);

    // - is it useful to let `.read()` use non-aligned/multiple addresses
    //   and sizes? can always round down address and round up size
    //   and then return the appropriate slice
    fn read(&self, address: usize, buf: &mut [u8]);
}

/// High-level API for the Flash memory
// pub trait WriteErase<N: ArrayLength<u8>> {
pub trait WriteErase {

    /// check flash status
    fn status(&self) -> FlashResult;

    /// Erase specified flash page.
    fn erase_page(&self, page: u8) -> FlashResult;

    // /// The smallest possible read, depends on platform
    // fn write_native(&self, address: usize, data: &mut GenericArray<u8, N>) -> FlashResult;

    fn write(&self, address: usize, data: &[u8]) -> FlashResult;

    // probably not so useful, as only applicable after mass erase
    // /// Faster programming
    // fn program_sixtyfour_bytes(&self, address: usize, data: [u8; 64]) -> FlashResult {

    /// Erase all Flash pages
    fn erase_all_pages(&self) -> FlashResult;
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

pub trait FlashOps: Locking + WriteErase + Read {}
