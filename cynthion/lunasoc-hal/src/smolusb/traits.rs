use crate::smolusb::control::{Direction, SetupPacket};

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
    /// Stalls the current control request.
    fn stall_request(&self);
    /// Sets the stall state for the given endpoint address
    fn stall_endpoint(&self, endpoint: u8, state: bool);
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