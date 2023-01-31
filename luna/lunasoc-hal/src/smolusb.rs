//! Simple peripheral-level USB stack

pub mod control;
pub mod error;
pub use error::ErrorKind;

use crate::Result;

// - SmolUsb ------------------------------------------------------------------

// TODO replace with Usb peripheral traits
use crate::UsbInterface0;

use crate::pac::Interrupt;

pub struct SmolUsb {
    peripheral: UsbInterface0
}

impl SmolUsb {
    pub fn new(peripheral: UsbInterface0) -> Self {
        Self {
            peripheral
        }
    }
}

impl SmolUsb {
    pub fn connect(&mut self) -> control::Speed {
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

pub struct EndpointControl {
}

impl EndpointControl {
    pub fn receive(&self) -> control::SetupPacket {
        unimplemented!();
    }
}

pub struct EndpointIn {
}

impl EndpointIn {
    pub fn send(&self, _buffer: &[u8]) -> Result<usize> {
        Ok(32)
    }
}


pub struct EndpointOut {
}
