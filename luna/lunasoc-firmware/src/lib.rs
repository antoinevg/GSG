#![feature(error_in_core)]
#![no_std]

// - modules ------------------------------------------------------------------

pub mod log;

// - aliases ------------------------------------------------------------------

pub use lunasoc_hal as hal;
pub use lunasoc_pac as pac;

// - constants ----------------------------------------------------------------

pub const SYSTEM_CLOCK_FREQUENCY: u32 = pac::clock::sysclk();

// - messages -----------------------------------------------------------------

#[derive(Debug)]
pub enum Message {
    // interrupts
    Interrupt(pac::Interrupt),
    UnknownInterrupt(usize),

    // usb events
    /// Received a SETUP packet on USB0_EP_CONTROL
    Usb0ReceivedSetupPacket(hal::smolusb::control::SetupPacket),
    /// Received a SETUP packet on USB1_EP_CONTROL
    Usb1ReceivedSetupPacket(hal::smolusb::control::SetupPacket),
    /// Received data on USB0_EP_OUT
    ///
    /// Contents is (endpoint, bytes_read, buffer)
    Usb0ReceivedData(u8, usize, [u8; 64]),
    /// Received data on USB1_EP_OUT
    ///
    /// Contents is (endpoint, bytes_read, buffer)
    Usb1ReceivedData(u8, usize, [u8; 64]),

    // TODO
    TimerEvent(u32),
}

// - Error --------------------------------------------------------------------

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ErrorKind {
    Unknown,
}

// trait: core::error::Error
impl core::error::Error for ErrorKind {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        use ErrorKind::*;
        match self {
            Unknown => "TODO Unknown",
        }
    }
}

// trait:: core::fmt::Display
impl core::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self, f)
    }
}

// trait: libgreat::error::Error
impl libgreat::error::Error for ErrorKind {
    type Error = ErrorKind; // TODO can we just say `Self`?
    fn kind(&self) -> Self::Error {
        *self
    }
}
