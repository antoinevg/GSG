#![cfg_attr(feature = "nightly", feature(error_in_core))]
#![cfg_attr(feature = "nightly", feature(panic_info_message))]
#![no_std]

// - modules ------------------------------------------------------------------

pub mod class;
pub mod error;
pub mod log;
pub mod macros;
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

pub const EP_MAX_ENDPOINTS: usize = 16;
pub const EP_MAX_RECEIVE_LENGTH: usize = 64;

// - messages -----------------------------------------------------------------

// TODO use these instead of u8 in messages
#[derive(Debug)]
pub enum UsbInterface {
    Target,   // Usb0
    Host,     // Usb1
    Sideband, // Usb2
}

pub enum Message {
    // interrupts
    HandleInterrupt(pac::Interrupt),
    HandleUnknownInterrupt(usize),

    // timer events
    TimerEvent(usize),

    // usb events
    /// Receives a USB bus reset
    ///
    /// Contents is (UsbInterface)
    UsbBusReset(UsbInterface),

    /// Received a SETUP packet on USBx_EP_CONTROL
    ///
    /// Contents is (UsbInterface, SetupPacket)
    UsbReceiveSetupPacket(UsbInterface, hal::smolusb::control::SetupPacket),

    /// Transfer is complete on USBx_EP_IN
    ///
    /// Contents is (UsbInterface, endpoint)
    UsbTransferComplete(UsbInterface, u8),

    /// Received data on USBx_EP_OUT
    ///
    /// Contents is (UsbInterface, endpoint, bytes_read, buffer)
    UsbReceiveData(UsbInterface, u8, usize, [u8; EP_MAX_RECEIVE_LENGTH]),

    // misc
    ErrorMessage(&'static str),
    DebugMessage(&'static str),
}

impl core::fmt::Debug for Message {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            // interrupts
            Message::HandleInterrupt(interrupt) => write!(f, "HandleInterrupt({:?})", interrupt),
            Message::HandleUnknownInterrupt(interrupt) => {
                write!(f, "HandleUnknownInterrupt({})", interrupt)
            }

            // timer events
            Message::TimerEvent(n) => write!(f, "TimerEvent({})", n),

            // usb events
            Message::UsbBusReset(interface) => {
                write!(f, "UsbBusReset({:?})", interface)
            }
            Message::UsbReceiveSetupPacket(interface, _setup_packet) => {
                write!(f, "UsbReceiveSetupPacket({:?})", interface)
            }
            Message::UsbReceiveData(interface, endpoint, bytes_read, _buffer) => write!(
                f,
                "UsbReceiveData({:?}, {}, {})",
                interface, endpoint, bytes_read
            ),
            Message::UsbTransferComplete(interface, endpoint) => {
                write!(f, "UsbTransferComplete({:?}, {})", interface, endpoint)
            }

            // misc
            Message::ErrorMessage(message) => {
                write!(f, "ErrorMessage({})", message)
            }
            Message::DebugMessage(message) => {
                write!(f, "DebugMessage({})", message)
            }
        }
    }
}
