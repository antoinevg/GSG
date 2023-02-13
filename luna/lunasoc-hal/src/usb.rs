//! Simple USB implementation

mod error;
pub use error::ErrorKind;

// - UsbInterface0 ------------------------------------------------------------

//use libgreat::smolusb::control::*;
use crate::smolusb::control::*; // TODO
use crate::smolusb::traits::{ControlRead, EndpointRead, EndpointWrite, EndpointWriteRef, UsbDriverOperations, UsbDriver}; // TODO

use crate::pac;
use pac::interrupt::Interrupt;

use log::{trace, warn};

pub struct Usb0 {
    pub device: pac::USB0,
    pub ep_control: pac::USB0_EP_CONTROL,
    pub ep_in: pac::USB0_EP_IN,
    pub ep_out: pac::USB0_EP_OUT,
    pub reset_count: usize,
}

impl Usb0 {
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

impl Usb0 {
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

    pub fn reset(&self) -> u8 {
        // TODO self.reset_count += 1;

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
        trace!("UsbInterface0::reset() -> {}", speed);
        speed
    }

    // TODO &mut self
    // TODO pass event to listen for
    pub fn enable_interrupts(&self) {
        // clear all event handlers
        self.clear_pending(Interrupt::USB0);
        self.clear_pending(Interrupt::USB0_EP_CONTROL);
        self.clear_pending(Interrupt::USB0_EP_IN);
        self.clear_pending(Interrupt::USB0_EP_OUT);

        // enable device controller events for bus reset signal
        self.enable_interrupt(Interrupt::USB0);
        self.enable_interrupt(Interrupt::USB0_EP_CONTROL);
        self.enable_interrupt(Interrupt::USB0_EP_IN);
        self.enable_interrupt(Interrupt::USB0_EP_OUT);
    }

    pub fn is_pending(&self, interrupt: Interrupt) -> bool {
        pac::csr::interrupt::pending(interrupt)
    }

    // TODO &mut self
    pub fn clear_pending(&self, interrupt: Interrupt) {
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

    // TODO &mut self
    pub fn enable_interrupt(&self, interrupt: Interrupt) {
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

    // TODO &mut self
    pub fn disable_interrupt(&self, interrupt: Interrupt) {
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

    pub fn ep_control_address(&self) -> u8 {
        self.ep_control.address.read().address().bits()
    }
}

// - trait: UsbDriverOperations -----------------------------------------------

impl UsbDriverOperations for Usb0 {
    /// Acknowledge the status stage of an incoming control request.
    fn ack_status_stage(&self, packet: &SetupPacket) {
        match Direction::from(packet.request_type) {
            // If this is an IN request, read a zero-length packet (ZLP) from the host..
            Direction::DeviceToHost => self.ep_out_prime_receive(0),
            // ... otherwise, send a ZLP.
            Direction::HostToDevice => self.write(0, [].into_iter()),
        }
    }

    fn set_address(&self, address: u8) {
        self.ep_control
            .address
            .write(|w| unsafe { w.address().bits(address & 0x7f) });
        self.ep_out
            .address
            .write(|w| unsafe { w.address().bits(address & 0x7f) });
    }

    /// Stalls the current control request.
    fn stall_request(&self) {
        self.ep_in.stall.write(|w| w.stall().bit(true));
        self.ep_out.stall.write(|w| w.stall().bit(true));
    }
}

// - trait: Read/Write traits -------------------------------------------------

impl ControlRead for Usb0 {
    fn read_control(&self, buffer: &mut [u8]) -> usize {
        // drain fifo
        let mut bytes_read = 0;
        let mut overflow = 0;
        while self.ep_control.have.read().have().bit() {
            if bytes_read >= buffer.len() {
                let _drain = self.ep_control.data.read().data().bits();
                overflow += 1;
            } else {
                buffer[bytes_read] = self.ep_control.data.read().data().bits();
                bytes_read += 1;
            }
        }

        trace!("  RX {} bytes + {} overflow", bytes_read, overflow,);

        bytes_read
    }
}

impl EndpointRead for Usb0 {
    fn read(&self, _endpoint: u8, buffer: &mut [u8]) -> usize {
        // drain fifo
        let mut bytes_read = 0;
        let mut overflow = 0;
        while self.ep_out.have.read().have().bit() {
            if bytes_read >= buffer.len() {
                let _drain = self.ep_out.data.read().data().bits();
                overflow += 1;
            } else {
                buffer[bytes_read] = self.ep_out.data.read().data().bits();
                bytes_read += 1;
            }
        }

        // re-enable endpoint after consuming all data
        self.ep_out.enable.write(|w| w.enable().bit(true));

        trace!("  RX {} bytes + {} overflow", bytes_read, overflow,);

        bytes_read
    }
}

impl EndpointWrite for Usb0 {
    fn write<I>(&self, endpoint: u8, iter: I)
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
}

impl EndpointWriteRef for Usb0 {
    fn write_ref<'a, I>(&self, endpoint: u8, iter: I)
    where
        I: Iterator<Item = &'a u8>,
    {
        // reset output fifo if needed
        if self.ep_in.have.read().have().bit() {
            trace!("  clear tx");
            self.ep_in.reset.write(|w| w.reset().bit(true));
        }

        // write data
        let mut bytes_written = 0;
        for &byte in iter {
            self.ep_in.data.write(|w| unsafe { w.data().bits(byte) });
            bytes_written += 1;
        }

        // finally, prime IN endpoint
        self.ep_in
            .epno
            .write(|w| unsafe { w.epno().bits(endpoint & 0xf) });

        trace!("  TX {} bytes", bytes_written);
    }
}

// mark implementation as complete
impl UsbDriver for Usb0 {}
