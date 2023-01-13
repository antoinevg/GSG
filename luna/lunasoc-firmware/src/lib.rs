#![feature(error_in_core)]

#![no_std]

// - modules ------------------------------------------------------------------



// - aliases ------------------------------------------------------------------

pub use lunasoc_hal as hal;
pub use lunasoc_pac as pac;

// - constants ----------------------------------------------------------------

pub const SYSTEM_CLOCK_FREQUENCY: u32 = 10_000_000;

// - Error --------------------------------------------------------------------

#[derive(Debug)]
pub enum Error {
    InvalidControlRequest,
    Unknown,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self, f)
    }
}

impl core::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;
