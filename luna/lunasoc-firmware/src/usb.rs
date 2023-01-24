//! Simple USB implementation

mod control;

pub use control::*;

// - UsbInterface0 ------------------------------------------------------------

use crate::pac;
use crate::{Error, Result};

use log::trace;

pub struct UsbInterface0 {
    pub usb: pac::USB0,
    pub ep_setup: pac::USB0_SETUP,
    pub ep_in: pac::USB0_EP_IN,
    pub ep_out: pac::USB0_EP_OUT,
}

impl UsbInterface0 {
    /// Set the interface up for new connections
    pub fn connect(&self) {
        // clear all event handlers
        self.ep_setup.ev_pending.modify(|r, w| w.pending().bit(r.pending().bit()));
        self.ep_in.ev_pending.modify(|r, w| w.pending().bit(r.pending().bit()));
        self.ep_out.ev_pending.modify(|r, w| w.pending().bit(r.pending().bit()));

        // disable all events
        self.ep_setup.ev_enable.write(|w| w.enable().bit(false));
        self.ep_in.ev_enable.write(|w| w.enable().bit(false));
        self.ep_out.ev_enable.write(|w| w.enable().bit(false));

        // disconnect device controller
        self.usb.connect.write(|w| w.connect().bit(false));

        // reset device address to 0
        self.ep_setup.address.write(|w| unsafe { w.address().bits(0) });

        // clear FIFOs
        self.ep_setup.reset.write(|w| w.reset().bit(true));
        self.ep_in.reset.write(|w| w.reset().bit(true));
        self.ep_out.reset.write(|w| w.reset().bit(true));

        // connect device controller
        self.usb.connect.write(|w| w.connect().bit(true));
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

        // enable it to prime a read
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

        self.ep_in_send_packet(0, buffer);
    }

    pub fn ep_in_send_packet(&self, endpoint: u8, buffer: &[u8]) {
        // clear output buffer
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
        while !self.ep_setup.have.read().have().bit() {
            counter += 1;
            if counter > 60_000_000 {
                return Err(Error::Timeout);
            }
        }

        // drain fifo
        let mut bytes_read = 0;
        let mut overflow = 0;
        let mut drain = 0;
        while self.ep_setup.have.read().have().bit() {
            if bytes_read >= buffer.len() {
                // drain
                drain = self.ep_setup.data.read().data().bits();
                overflow += 1;
            } else {
                buffer[bytes_read] = self.ep_setup.data.read().data().bits();
                bytes_read += 1;
            }
        }

        trace!("  RX {} + {} bytes: {:x?} - {:x}", bytes_read, overflow, buffer, drain);

        Ok(())
    }

    /// Stalls the current control request.
    pub fn stall_request(&self) {
        self.ep_in.stall.write(|w| w.stall().bit(true));
        self.ep_out.stall.write(|w| w.stall().bit(true));
    }
}
