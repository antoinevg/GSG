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
pub use libgreat::firmware::BoardInformation;

// - constants ----------------------------------------------------------------

pub const SYSTEM_CLOCK_FREQUENCY: u32 = pac::clock::sysclk();
pub const BOARD_INFORMATION: BoardInformation = BoardInformation {
    board_id: [0x00, 0x00, 0x00, 0x00],
    version_string: "v2023.0.1\0",
    part_id: [0x30, 0xa, 0x00, 0xa0, 0x5e, 0x4f, 0x60, 0x00],
    serial_number: [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe6, 0x67, 0xcc, 0x57, 0x57, 0x53, 0x6f,
        0x30,
    ],
};

pub const EP_MAX_RECEIVE_LENGTH: usize = 64;

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
    Usb0ReceiveData(u8, usize, [u8; EP_MAX_RECEIVE_LENGTH]),

    /// Received data on USB1_EP_OUT
    ///
    /// Contents is (endpoint, bytes_read, buffer)
    Usb1ReceiveData(u8, usize, [u8; EP_MAX_RECEIVE_LENGTH]),

    /// Received data on USB2_EP_OUT
    ///
    /// Contents is (endpoint, bytes_read, buffer)
    Usb2ReceiveData(u8, usize, [u8; EP_MAX_RECEIVE_LENGTH]),

    // TODO
    TimerEvent(u32),
}
