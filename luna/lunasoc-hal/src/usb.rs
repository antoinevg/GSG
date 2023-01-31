//! Simple USB implementation

mod error;
pub use error::ErrorKind;

// - UsbInterface0 ------------------------------------------------------------

use libgreat::smolusb::control::*;
use libgreat::Result;

use crate::pac;

use log::trace;

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
        ep_setup: pac::USB0_EP_CONTROL,
        ep_in: pac::USB0_EP_IN,
        ep_out: pac::USB0_EP_OUT,
    ) -> Self {
        Self {
            device,
            ep_control: ep_setup,
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
    pub fn connect(&self) -> u8 {
        // disconnect device controller
        self.device.connect.write(|w| w.connect().bit(false));

        // disable endpoint events
        self.device.ev_enable.write(|w| w.enable().bit(false));
        self.ep_control.ev_enable.write(|w| w.enable().bit(false));
        self.ep_in.ev_enable.write(|w| w.enable().bit(false));
        self.ep_out.ev_enable.write(|w| w.enable().bit(false));

        // reset FIFOs
        self.ep_control.reset.write(|w| w.reset().bit(true));
        self.ep_in.reset.write(|w| w.reset().bit(true));
        self.ep_out.reset.write(|w| w.reset().bit(true));

        self.listen();

        // connect device controller
        self.device.connect.write(|w| w.connect().bit(true));

        // 0: High, 1: Full, 2: Low, 3:SuperSpeed (incl SuperSpeed+)
        self.device.speed.read().speed().bits()
    }

    pub fn reset(&mut self) -> u8 {
        trace!("UsbInterface0::reset()");

        self.reset_count += 1;

        // disable endpoint events
        self.ep_control.ev_enable.write(|w| w.enable().bit(false));
        self.ep_in.ev_enable.write(|w| w.enable().bit(false));
        self.ep_out.ev_enable.write(|w| w.enable().bit(false));

        // reset device address to 0
        self.ep_control
            .address
            .write(|w| unsafe { w.address().bits(0) });

        // reset FIFOs
        self.ep_control.reset.write(|w| w.reset().bit(true));
        self.ep_in.reset.write(|w| w.reset().bit(true));
        self.ep_out.reset.write(|w| w.reset().bit(true));

        self.listen();

        // TODO handle speed
        // 0: High, 1: Full, 2: Low, 3:SuperSpeed (incl SuperSpeed+)
        let speed = self.device.speed.read().speed().bits();
        trace!("Reset: {}", speed);
        speed
    }

    // TODO pass event to listen for
    pub fn listen(&self) {
        // clear all event handlers
        self.device
            .ev_pending
            .modify(|r, w| w.pending().bit(r.pending().bit()));
        /*self.ep_setup
            .ev_pending
            .modify(|r, w| w.pending().bit(r.pending().bit()));
        self.ep_in
            .ev_pending
            .modify(|r, w| w.pending().bit(r.pending().bit()));
        self.ep_out
            .ev_pending
            .modify(|r, w| w.pending().bit(r.pending().bit()));*/

        // enable device controller events for bus reset signal
        self.device.ev_enable.write(|w| w.enable().bit(true));
        //self.ep_in.ev_enable.write(|w| w.enable().bit(true));
        //self.ep_out.ev_enable.write(|w| w.enable().bit(true));
        //self.ep_setup.ev_enable.write(|w| w.enable().bit(true));
    }

    /// Acknowledge the status stage of an incoming control request.
    pub fn ack_status_stage(&self, packet: &SetupPacket) {
        // If this is an IN request, read a zero-length packet (ZLP) from the host..
        if (packet.request_type & MASK_DIRECTION_IN) != 0 {
            self.ep_out_prime_receive(0);
        } else {
            // ... otherwise, send a ZLP.
            self.ep_in_send_packet(0, &[]);
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

    pub fn ep_in_send_control_response(&self, packet: &SetupPacket, buffer: &[u8]) {
        // if the host is requesting less than the maximum amount of data,
        // only respond with the amount requested
        let requested_length = packet.length as usize;
        let buffer = if requested_length < buffer.len() {
            &buffer[0..requested_length]
        } else {
            buffer
        };

        // handle case where device is asking for _more_
        /*let mut response_buffer = [0_u8; 128];
        let buffer = if requested_length > buffer.len() {
            for i in 0..buffer.len() {
                response_buffer[i] = buffer[i];
            }
            if requested_length > response_buffer.len() {
                &response_buffer
            } else {
                &response_buffer[0..requested_length]
            }
        } else {
            buffer
        };*/

        self.ep_in_send_packet(0, buffer);
    }

    pub fn ep_in_send_packet(&self, endpoint: u8, buffer: &[u8]) {
        // reset output fifo if needed
        if self.ep_in.have.read().have().bit() {
            trace!("  clear tx");
            self.ep_in.reset.write(|w| w.reset().bit(true));
        }

        // send data
        for &word in buffer {
            self.ep_in.data.write(|w| unsafe { w.data().bits(word) });
        }

        // finally, prime IN endpoint
        self.ep_in
            .epno
            .write(|w| unsafe { w.epno().bits(endpoint & 0xf) });

        trace!("  TX {} bytes: {:x?}", buffer.len(), buffer);
    }

    pub fn ep_setup_read_packet(&self, buffer: &mut [u8]) -> Result<()> {
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

        trace!(
            "  RX {} + {} bytes: {:x?} - {:x}",
            bytes_read,
            overflow,
            buffer,
            drain
        );

        Ok(())
    }

    /// Stalls the current control request.
    pub fn stall_request(&self) {
        self.ep_in.stall.write(|w| w.stall().bit(true));
        self.ep_out.stall.write(|w| w.stall().bit(true));
    }

    pub fn ep_setup_address(&self) -> u8 {
        self.ep_control.address.read().address().bits()
    }

    pub fn ep_setup_set_address(&self, address: u8) {
        self.ep_control
            .address
            .write(|w| unsafe { w.address().bits(address & 0x7f) });
    }
}
