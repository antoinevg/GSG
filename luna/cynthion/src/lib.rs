#![cfg_attr(feature = "nightly", feature(error_in_core))]
#![cfg_attr(feature = "nightly", feature(panic_info_message))]
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
pub use libgreat::error::GreatResult;
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

pub enum Message {
    // interrupts
    HandleInterrupt(pac::Interrupt),
    HandleUnknownInterrupt(usize),

    // timer events
    TimerEvent(usize),

    // usb events
    /// Receives a USB bus reset
    ///
    /// Contents is (interface)
    UsbBusReset(u8),

    /// Received a SETUP packet on USBx_EP_CONTROL
    ///
    /// Contents is (interface, setup_packet)
    UsbReceiveSetupPacket(u8, hal::smolusb::control::SetupPacket),

    /// Received data on USBx_EP_OUT
    ///
    /// Contents is (interface, endpoint, bytes_read, buffer)
    UsbReceiveData(u8, u8, usize, [u8; EP_MAX_RECEIVE_LENGTH]),

    /// Transfer is complete on USBx_EP_IN
    ///
    /// Contents is (interface, endpoint)
    UsbTransferComplete(u8, u8),

    // misc
    ErrorMessage(&'static str),
}

impl core::fmt::Debug for Message {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Message::HandleInterrupt(interrupt) => write!(f, "HandleInterrupt({:?})", interrupt),
            Message::HandleUnknownInterrupt(interrupt) => {
                write!(f, "HandleUnknownInterrupt({})", interrupt)
            }
            Message::TimerEvent(n) => write!(f, "TimerEvent({})", n),
            Message::UsbBusReset(interface) => {
                write!(f, "UsbBusReset({})", interface)
            }
            Message::UsbReceiveSetupPacket(interface, _setup_packet) => {
                write!(f, "UsbReceiveSetupPacket({})", interface)
            }
            Message::UsbReceiveData(interface, endpoint, bytes_read, _buffer) => write!(
                f,
                "UsbReceiveData({}, {}, {})",
                interface, endpoint, bytes_read
            ),
            Message::UsbTransferComplete(interface, endpoint) => {
                write!(f, "UsbTransferComplete({}, {})", interface, endpoint)
            }
            Message::ErrorMessage(message) => {
                write!(f, "ErrorMessage({})", message)
            }
        }
    }
}
