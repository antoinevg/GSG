#![feature(error_in_core)]
#![feature(panic_info_message)]
#![no_std]

// - modules ------------------------------------------------------------------

pub mod class;
pub mod error;
pub mod log;
pub mod panic_log;

// - aliases ------------------------------------------------------------------

pub use lunasoc_hal as hal;
pub use lunasoc_pac as pac;

// - re-exports ---------------------------------------------------------------

pub use error::FirmwareError;
pub use libgreat::error::Result;

// - constants ----------------------------------------------------------------

pub const SYSTEM_CLOCK_FREQUENCY: u32 = pac::clock::sysclk();

// - messages -----------------------------------------------------------------

#[derive(Debug)]
pub enum Message {
    // interrupts
    HandleInterrupt(pac::Interrupt),
    HandleUnknownInterrupt(usize),

    // usb events
    /// Received a SETUP packet on USB0_EP_CONTROL
    Usb0ReceiveSetupPacket(hal::smolusb::control::SetupPacket),
    /// Received a SETUP packet on USB1_EP_CONTROL
    Usb1ReceiveSetupPacket(hal::smolusb::control::SetupPacket),
    /// Received a SETUP packet on USB2_EP_CONTROL
    Usb2ReceiveSetupPacket(hal::smolusb::control::SetupPacket),
    /// Received data on USB0_EP_OUT
    ///
    /// Contents is (endpoint, bytes_read, buffer)
    Usb0ReceiveData(u8, usize, [u8; 64]),
    /// Received data on USB1_EP_OUT
    ///
    /// Contents is (endpoint, bytes_read, buffer)
    Usb1ReceiveData(u8, usize, [u8; 64]),
    /// Received data on USB2_EP_OUT
    ///
    /// Contents is (endpoint, bytes_read, buffer)
    Usb2ReceiveData(u8, usize, [u8; 64]),

    // TODO
    TimerEvent(u32),
}
