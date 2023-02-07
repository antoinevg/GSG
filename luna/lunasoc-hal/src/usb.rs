//! Simple USB implementation

mod error;
pub use error::ErrorKind;

// - UsbInterface0 ------------------------------------------------------------

//use libgreat::smolusb::control::*;
use crate::smolusb::control::*; // TODO
use libgreat::Result;

use crate::pac;
use pac::interrupt::Interrupt;

use log::{trace, warn};

pub struct UsbInterface0 {
    pub device: pac::USB0,
    pub ep_control: pac::USB0_EP_CONTROL,
    pub ep_in: pac::USB0_EP_IN,
    pub ep_out: pac::USB0_EP_OUT,
    pub reset_count: usize,
}

impl UsbInterface0 {
    /// Create a new `Usb` from the [`USB`](pac::USB) peripheral.
    pub fn new(
        device: pac::USB0,
        ep_control: pac::USB0_EP_CONTROL,
        ep_in: pac::USB0_EP_IN,
        ep_out: pac::USB0_EP_OUT,
    ) -> Self {
        Self {
            device,
            ep_control,
            ep_in,
            ep_out,
            reset_count: 0,
        }
    }

    /// Release the [`USB`](pac::USB) peripheral and consume self.
    pub fn free(
        self,
    ) -> (
        pac::USB0,
        pac::USB0_EP_CONTROL,
        pac::USB0_EP_IN,
        pac::USB0_EP_OUT,
    ) {
        (self.device, self.ep_control, self.ep_in, self.ep_out)
    }

    /// Obtain a static `Usb` instance for use in e.g. interrupt handlers
    ///
    /// # Safety
    ///
    /// 'Tis thine responsibility, that which thou doth summon.
    pub unsafe fn summon() -> Self {
        Self {
            device: pac::Peripherals::steal().USB0,
            ep_control: pac::Peripherals::steal().USB0_EP_CONTROL,
            ep_in: pac::Peripherals::steal().USB0_EP_IN,
            ep_out: pac::Peripherals::steal().USB0_EP_OUT,
            reset_count: 0,
        }
    }
}

impl UsbInterface0 {
    /// Set the interface up for new connections
    pub fn connect(&mut self) -> u8 {
        // disconnect device controller
        self.device.connect.write(|w| w.connect().bit(false));

        // disable endpoint events
        self.disable_interrupt(Interrupt::USB0);
        self.disable_interrupt(Interrupt::USB0_EP_CONTROL);
        self.disable_interrupt(Interrupt::USB0_EP_IN);
        self.disable_interrupt(Interrupt::USB0_EP_OUT);

        // reset FIFOs
        self.ep_control.reset.write(|w| w.reset().bit(true));
        self.ep_in.reset.write(|w| w.reset().bit(true));
        self.ep_out.reset.write(|w| w.reset().bit(true));

        // connect device controller
        self.device.connect.write(|w| w.connect().bit(true));

        // 0: High, 1: Full, 2: Low, 3:SuperSpeed (incl SuperSpeed+)
        self.device.speed.read().speed().bits()
    }

    pub fn reset(&mut self) -> u8 {
        trace!("UsbInterface0::reset()");

        self.reset_count += 1;

        // disable endpoint events
        self.disable_interrupt(Interrupt::USB0_EP_CONTROL);
        self.disable_interrupt(Interrupt::USB0_EP_IN);
        self.disable_interrupt(Interrupt::USB0_EP_OUT);

        // reset device address to 0
        self.ep_control
            .address
            .write(|w| unsafe { w.address().bits(0) });

        // reset FIFOs
        self.ep_control.reset.write(|w| w.reset().bit(true));
        self.ep_in.reset.write(|w| w.reset().bit(true));
        self.ep_out.reset.write(|w| w.reset().bit(true));

        self.enable_interrupts();

        // TODO handle speed
        // 0: High, 1: Full, 2: Low, 3:SuperSpeed (incl SuperSpeed+)
        let speed = self.device.speed.read().speed().bits();
        trace!("Reset: {}", speed);
        speed
    }

    // TODO pass event to listen for
    pub fn enable_interrupts(&mut self) {
        // clear all event handlers
        self.clear_pending(Interrupt::USB0);
        //self.clear_pending(Interrupt::USB0_EP_CONTROL);
        //self.clear_pending(Interrupt::USB0_EP_IN);
        self.clear_pending(Interrupt::USB0_EP_OUT);

        // enable device controller events for bus reset signal
        self.enable_interrupt(Interrupt::USB0);
        //self.enable_interrupt(Interrupt::USB0_EP_CONTROL);
        //self.enable_interrupt(Interrupt::USB0_EP_IN);
        self.enable_interrupt(Interrupt::USB0_EP_OUT);
    }

    pub fn is_pending(&self, interrupt: Interrupt) -> bool {
        match interrupt {
            Interrupt::USB0 => self.device.ev_pending.read().pending().bit_is_set(),
            Interrupt::USB0_EP_CONTROL => self.ep_control.ev_pending.read().pending().bit_is_set(),
            Interrupt::USB0_EP_IN => self.ep_in.ev_pending.read().pending().bit_is_set(),
            Interrupt::USB0_EP_OUT => self.ep_out.ev_pending.read().pending().bit_is_set(),
            _ => {
                warn!("Ignoring invalid interrupt is pending: {:?}", interrupt);
                false
            }
        }
    }

    pub fn clear_pending(&mut self, interrupt: Interrupt) {
        match interrupt {
            Interrupt::USB0 => self
                .device
                .ev_pending
                .modify(|r, w| w.pending().bit(r.pending().bit())),
            Interrupt::USB0_EP_CONTROL => self
                .ep_control
                .ev_pending
                .modify(|r, w| w.pending().bit(r.pending().bit())),
            Interrupt::USB0_EP_IN => self
                .ep_in
                .ev_pending
                .modify(|r, w| w.pending().bit(r.pending().bit())),
            Interrupt::USB0_EP_OUT => self
                .ep_out
                .ev_pending
                .modify(|r, w| w.pending().bit(r.pending().bit())),
            _ => {
                warn!("Ignoring invalid interrupt clear pending: {:?}", interrupt);
            }
        }
    }

    pub fn enable_interrupt(&mut self, interrupt: Interrupt) {
        match interrupt {
            Interrupt::USB0 => self.device.ev_enable.write(|w| w.enable().bit(true)),
            Interrupt::USB0_EP_CONTROL => self.ep_control.ev_enable.write(|w| w.enable().bit(true)),
            Interrupt::USB0_EP_IN => self.ep_in.ev_enable.write(|w| w.enable().bit(true)),
            Interrupt::USB0_EP_OUT => self.ep_out.ev_enable.write(|w| w.enable().bit(true)),
            _ => {
                warn!("Ignoring invalid interrupt enable: {:?}", interrupt);
            }
        }
    }

    pub fn disable_interrupt(&mut self, interrupt: Interrupt) {
        match interrupt {
            Interrupt::USB0 => self.device.ev_enable.write(|w| w.enable().bit(false)),
            Interrupt::USB0_EP_CONTROL => {
                self.ep_control.ev_enable.write(|w| w.enable().bit(false))
            }
            Interrupt::USB0_EP_IN => self.ep_in.ev_enable.write(|w| w.enable().bit(false)),
            Interrupt::USB0_EP_OUT => self.ep_out.ev_enable.write(|w| w.enable().bit(false)),
            _ => {
                warn!("Ignoring invalid interrupt enable: {:?}", interrupt);
            }
        }
    }

    pub fn ep_control_read_packet(&self, buffer: &mut [u8]) -> Result<()> {
        // block until setup data is available
        let mut counter = 0;
        while !self.ep_control.have.read().have().bit() {
            counter += 1;
            if counter > 60_000_000 {
                return Err(&ErrorKind::Timeout);
            }
        }

        // drain fifo
        let mut bytes_read = 0;
        let mut overflow = 0;
        let mut drain = 0;
        while self.ep_control.have.read().have().bit() {
            if bytes_read >= buffer.len() {
                // drain
                drain = self.ep_control.data.read().data().bits();
                overflow += 1;
            } else {
                buffer[bytes_read] = self.ep_control.data.read().data().bits();
                bytes_read += 1;
            }
        }

        if bytes_read > 0 {
            trace!(
                "  RX {} bytes + {} overflow - {:x?} - {:x}",
                bytes_read,
                overflow,
                buffer,
                drain
            );
        }

        Ok(())
    }

    pub fn ep_out_read(&self, endpoint: u8, buffer: &mut [u8]) -> Result<()> {
        // do we need to prime it after receiving the interrupt?
        self.ep_out_prime_receive(endpoint); // TODO ???

        // drain fifo
        let mut bytes_read = 0;
        let mut overflow = 0;
        let mut drain = 0;
        while self.ep_out.have.read().have().bit() {
            if bytes_read >= buffer.len() {
                // drain
                drain = self.ep_out.data.read().data().bits();
                overflow += 1;
            } else {
                buffer[bytes_read] = self.ep_out.data.read().data().bits();
                bytes_read += 1;
            }
        }

        if bytes_read > 0 {
            trace!(
                "  RX {} bytes + {} overflow - {:x?} - {:x}",
                bytes_read,
                overflow,
                buffer,
                drain
            );
        }

        Ok(())
    }

    pub fn ep_in_write<I>(&self, endpoint: u8, iter: I)
    where
        I: Iterator<Item = u8>,
    {
        // reset output fifo if needed
        if self.ep_in.have.read().have().bit() {
            trace!("  clear tx");
            self.ep_in.reset.write(|w| w.reset().bit(true));
        }

        // write data
        let mut bytes_written = 0;
        for byte in iter {
            self.ep_in.data.write(|w| unsafe { w.data().bits(byte) });
            bytes_written += 1;
        }

        // finally, prime IN endpoint
        self.ep_in
            .epno
            .write(|w| unsafe { w.epno().bits(endpoint & 0xf) });

        trace!("  TX {} bytes", bytes_written);
    }

    /// Acknowledge the status stage of an incoming control request.
    pub fn ack_status_stage(&self, packet: &SetupPacket) {
        match Direction::from(packet.request_type) {
            // If this is an IN request, read a zero-length packet (ZLP) from the host..
            Direction::DeviceToHost => self.ep_out_prime_receive(0),
            // ... otherwise, send a ZLP.
            Direction::HostToDevice => self.ep_in_write(0, [].into_iter()),
        }
    }

    /// Prepare endpoint to receive a single OUT packet.
    pub fn ep_out_prime_receive(&self, endpoint: u8) {
        // clear receive buffer
        self.ep_out.reset.write(|w| w.reset().bit(true));

        // select endpoint
        self.ep_out
            .epno
            .write(|w| unsafe { w.epno().bits(endpoint) });

        // prime endpoint
        self.ep_out.prime.write(|w| w.prime().bit(true));

        // enable it
        self.ep_out.enable.write(|w| w.enable().bit(true));
    }

    /// Stalls the current control request.
    pub fn stall_request(&self) {
        self.ep_in.stall.write(|w| w.stall().bit(true));
        self.ep_out.stall.write(|w| w.stall().bit(true));
    }

    pub fn ep_control_address(&self) -> u8 {
        self.ep_control.address.read().address().bits()
    }

    pub fn ep_control_set_address(&self, address: u8) {
        self.ep_control
            .address
            .write(|w| unsafe { w.address().bits(address & 0x7f) });
    }
}
