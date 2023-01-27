#![feature(error_in_core)]
#![no_std]

// - modules ------------------------------------------------------------------

pub mod log;

// - aliases ------------------------------------------------------------------

pub use lunasoc_hal as hal;
pub use lunasoc_pac as pac;

// - constants ----------------------------------------------------------------

pub const SYSTEM_CLOCK_FREQUENCY: u32 = 60_000_000;

// - Error --------------------------------------------------------------------

#[derive(Debug)]
pub enum FirmwareError {
    Unknown,
}

/*impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self, f)
    }
}*/
//impl core::error::Error for Error {}

// Ugly little hack for now - https://stackoverflow.com/questions/48430836/
/*impl<E: core::fmt::Display> core::convert::From<E> for FirmwareError {
    fn from(_error: E) -> Self {
        FirmwareError::Unknown
    }
}
*/
pub type FirmwareResult<T> = core::result::Result<T, FirmwareError>;
