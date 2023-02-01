//! Simple peripheral-level USB stack

pub mod control;
pub mod descriptor;
pub mod error;
pub use error::ErrorKind;

use crate::Result;

/// USB Speed
///
/// Note: These match the gateware peripheral so the mapping isn't particularly meaningful in other contexts.
#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum Speed {
    Low = 2,        // 1.5 Mbps
    Full = 1,       //  12 Mbps
    High = 0,       // 480 Mbps
    SuperSpeed = 3, // 5/10 Gbps (includes SuperSpeed+)
}

impl From<u8> for Speed {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0 => Speed::High,
            1 => Speed::Full,
            2 => Speed::Low,
            3 => Speed::SuperSpeed,
            _ => unimplemented!(),
        }
    }
}

// - SmolUsb ------------------------------------------------------------------

// TODO replace with Usb peripheral traits
use crate::UsbInterface0;

use crate::pac::Interrupt;

pub struct Device {
    peripheral: UsbInterface0,
}

impl Device {
    pub fn new(peripheral: UsbInterface0) -> Self {
        Self { peripheral }
    }
}

impl Device {
    pub fn connect(&mut self) -> Speed {
        self.peripheral.connect().into()
    }

    pub fn reset(&mut self) {
        self.peripheral.reset();
    }

    // TODO this may not belong here...
    pub fn enable_interrupts(&mut self) {
        self.peripheral.clear_pending(Interrupt::USB0);
        //self.peripheral.clear_pending(Interrupt::USB0_EP_CONTROL);
        //self.peripheral.clear_pending(Interrupt::USB0_EP_IN);
        //self.peripheral.clear_pending(Interrupt::USB0_EP_OUT);

        self.peripheral.enable_interrupt(Interrupt::USB0);
        //self.peripheral.enable_interrupt(Interrupt::USB0_EP_CONTROL);
        //self.peripheral.enable_interrupt(Interrupt::USB0_EP_IN);
        //self.peripheral.enable_interrupt(Interrupt::USB0_EP_OUT);
    }

    // TODO this may not belong here either...
    pub fn disable_interrupts(&mut self) {
        self.peripheral.disable_interrupt(Interrupt::USB0);
        //self.peripheral.disable_interrupt(Interrupt::USB0_EP_CONTROL);
        //self.peripheral.disable_interrupt(Interrupt::USB0_EP_IN);
        //self.peripheral.disable_interrupt(Interrupt::USB0_EP_OUT);
    }
}

// - Endpoints ----------------------------------------------------------------

pub trait Endpoint {}

pub struct EndpointControl {}

impl EndpointControl {
    pub fn receive(&self) -> control::SetupPacket {
        unimplemented!();
    }

    pub fn address(&self) -> u8 {
        unimplemented!();
    }

    pub fn set_address(&self, _address: u8) {}
}

pub struct EndpointIn {}

impl EndpointIn {
    pub fn send(&self, _buffer: &[u8]) -> Result<usize> {
        Ok(32)
    }
}

pub struct EndpointOut {}
