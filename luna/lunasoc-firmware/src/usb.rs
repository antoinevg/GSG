//! Simple USB implementation

mod request;

pub use request::*;

// - UsbInterface0 ------------------------------------------------------------

use crate::pac;

pub struct UsbInterface0 {
    pub usb: pac::USB0,
    pub setup: pac::USB0_SETUP,
    pub ep0_in: pac::USB0_EP0_IN,
    pub ep0_out: pac::USB0_EP0_OUT,
}

impl UsbInterface0 {
    /// Acknowledge the status stage of an incoming control request.
    pub fn ack_status_stage(&self, setup_request: &SetupPacket) {
        // If this is an IN request, read a zero-length packet (ZLP) from the host..
        if (setup_request.request_type & MASK_DIRECTION_IN) != 0 {
            self.prime_receive(0);
        } else {
            // ... otherwise, send a ZLP.
            self.send_packet(0, &[]);
        }
    }

    /// Prepare endpoint to receive a single OUT packet.
    pub fn prime_receive(&self, endpoint: u8) {
        // clear receive buffer
        self.ep0_out.reset.write(|w| w.reset().bit(true));

        // select endpoint
        self.ep0_out
            .epno
            .write(|w| unsafe { w.epno().bits(endpoint) });

        // enable it to prime a read
        self.ep0_out.enable.write(|w| w.enable().bit(true));
    }

    pub fn send_control_response(&self, setup_request: &SetupPacket, buffer: &[u8]) {
        // if the host is requesting less than the maximum amount of data,
        // only respond with the amount requested
        let requested_length = setup_request.length as usize;
        let buffer = if requested_length < buffer.len() {
            &buffer[0..requested_length]
        } else {
            buffer
        };

        self.send_packet(0, buffer);
    }

    pub fn send_packet(&self, endpoint: u8, buffer: &[u8]) {
        // clear output buffer
        self.ep0_in.reset.write(|w| w.reset().bit(true));

        // send data
        for &word in buffer {
            self.ep0_in.data.write(|w| unsafe { w.data().bits(word) })
        }

        // finally, prime IN endpoint
        self.ep0_in
            .epno
            .write(|w| unsafe { w.epno().bits(endpoint) });
    }

    /// Stalls the current control request.
    pub fn stall_request(&self) {
        self.ep0_in.stall.write(|w| w.stall().bit(true));
        self.ep0_out.stall.write(|w| w.stall().bit(true));
    }
}
