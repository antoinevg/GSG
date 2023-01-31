#![feature(error_in_core)]
#![feature(panic_info_message)]
#![no_std]

pub mod gpio;
pub mod serial;
pub mod timer;
pub mod usb;

// export peripherals
pub use serial::Serial;
pub use timer::Timer;
pub use usb::UsbInterface0;

// re-export dependencies
pub use lunasoc_pac as pac;
pub use nb;

pub use embedded_hal as hal;
pub(crate) use embedded_hal_nb as hal_nb;

pub use embedded_hal_0 as hal_0;
