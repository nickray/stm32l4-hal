// use generic_array::{ArrayLength, GenericArray};

pub struct FlashInstance;

// Is there any point in `pub trait XXX where Self: Sized`?

pub trait Locking {
    fn is_locked(&self) -> bool;
    fn unlock(&self);
    fn lock(&self);
}

// pub trait Read<N: ArrayLength<u8>> {
pub trait Read {
    // // the smallest possible read, depends on platform
    // // not sure this is so useful in the trait - maybe suggest
    // // trait implementors just add it as non-trait method with
    // // appropriate N, and normal array of u8
    // fn read_native(&self, address: usize) -> GenericArray<u8, N>;

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
