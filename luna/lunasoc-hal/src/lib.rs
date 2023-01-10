#![no_std]

pub mod csr;

#[cfg(feature = "gpio")]
pub mod gpio;
pub mod timer;
pub mod uart;

//pub mod time;

// re-exports
pub use lunasoc_pac as pac;

pub use embedded_hal as hal;
pub use nb;
pub mod prelude {
    pub use embedded_hal::prelude::*;
}

// peripherals
pub use timer::Timer;
pub use uart::Uart;
