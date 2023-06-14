use crate::control::{Direction, SetupPacket};

use zerocopy::AsBytes;

use core::slice;

// - Read/Write ---------------------------------------------------------------

pub trait ControlRead {
    fn read_control(&self, buffer: &mut [u8]) -> usize;
}

pub trait EndpointRead {
    fn read(&self, endpoint: u8, buffer: &mut [u8]) -> usize;
}

// These two should be one trait
// TODO return bytes_written

pub trait EndpointWrite {
    fn write<'a, I>(&self, endpoint: u8, iter: I)
    where
        I: Iterator<Item = u8>;
}

pub trait EndpointWriteRef {
    fn write_ref<'a, I>(&self, endpoint: u8, iter: I)
    where
        I: Iterator<Item = &'a u8>;
}

// - UsbDriverOperations ------------------------------------------------------

pub trait UsbDriverOperations {
    /// Connect
    fn connect(&self) -> u8;
    /// Disconnect
    fn disconnect(&self);
    /// Reset
    fn reset(&self) -> u8;
    /// Bus Reset
    fn bus_reset(&self) -> u8;
    /// Acknowledge the status stage of an incoming control request.
    fn ack_status_stage(&self, packet: &SetupPacket);
    fn ack(&self, endpoint: u8, direction: Direction);
    fn set_address(&self, address: u8);
    /// Stall the current control request.
    /// TODO replace this with stall_endpoint_*
    fn stall_request(&self);
    /// Set the stall state for the given endpoint address
    /// TODO replace this with stall_endpoint_*
    fn stall_endpoint_address(&self, endpoint: u8, state: bool);
    /// Stall the given IN endpoint
    fn stall_endpoint_in(&self, endpoint: u8);
    /// Stall the given OUT endpoint
    fn stall_endpoint_out(&self, endpoint: u8);

    /// Clear any halt condition on the target endpoint, and clear the data toggle bit.
    fn clear_feature_endpoint_halt(&self, endpoint_address: u8);
}

pub trait UnsafeUsbDriverOperations {
    unsafe fn set_tx_ack_active(&self);
    unsafe fn clear_tx_ack_active(&self);
    unsafe fn is_tx_ack_active(&self) -> bool;
}

// convenience alias
pub trait UsbDriver:
    ControlRead
    + EndpointRead
    + EndpointWrite
    + EndpointWriteRef
    + UsbDriverOperations
    + UnsafeUsbDriverOperations
{
}

// - AsIterator ---------------------------------------------------------------

pub trait AsByteSliceIterator: AsBytes {
    fn as_iter(&self) -> slice::Iter<u8> {
        self.as_bytes().iter()
    }
}

trait AsByteIterator<'a> {
    type AsIter: Iterator<Item = &'a u8>;
    fn as_iter(&'a self) -> Self::AsIter;
}

trait AsIterator<'a> {
    type Item;
    type AsIter: Iterator<Item = Self::Item>;
    fn as_iter(&'a self) -> Self::AsIter;
}
