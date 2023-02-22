#![feature(error_in_core)]
#![feature(panic_info_message)]
#![no_std]

// TODO move to libgreat when it's done
pub mod smolusb;

pub mod gpio;
pub mod serial;
pub mod timer;
pub mod usb;

// export peripherals
pub use serial::Serial;
pub use timer::Timer;
pub use usb::{Usb0, Usb1, Usb2};

// re-export dependencies
pub use libgreat::Result;

pub use lunasoc_pac as pac;

pub use embedded_hal as hal;
pub use embedded_hal_0 as hal_0;
pub(crate) use embedded_hal_nb as hal_nb;

pub use nb;
