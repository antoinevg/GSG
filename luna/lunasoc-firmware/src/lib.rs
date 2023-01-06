#![no_std]

// - aliases ------------------------------------------------------------------

pub use lunasoc_pac as pac;
pub use lunasoc_hal as hal;


// - constants ----------------------------------------------------------------

pub const SYSTEM_CLOCK_FREQUENCY: u32 = 10_000_000;


// - modules ------------------------------------------------------------------

// TODO move these into lunasoc-pac
pub mod csr;
pub mod minerva;
pub mod register {
    pub use crate::minerva;
}

// TODO move these into lunasoc-hal
hal::timer! { Timer: pac::TIMER, }
hal::uart!  { Uart:  pac::UART,  }
