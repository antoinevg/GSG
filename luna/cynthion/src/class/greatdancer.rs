#![allow(dead_code, unused_imports, unused_variables)] // TODO

use libgreat::error::{GreatError, Result};
use libgreat::gcp::Verb;

use log::{debug, error};
use zerocopy::{AsBytes, BigEndian, FromBytes, LittleEndian, Unaligned, U32};

use core::any::Any;
use core::slice;

pub static CLASS_DOCS: &str = "Common API for updating firmware on a libgreat device.\0";

fn dummy_handler<'a>(_arguments: &[u8], _context: &'a dyn Any) -> slice::Iter<'a, u8> {
    [].iter()
}

pub const fn verbs<'a>() -> [Verb<'a>; 13] {
    [
        // - connection / disconnection
        Verb {
            id: 0x0,
            name: "connect\0",
            doc: "Set up the target port to connect to a host.\nEnables the target port's USB pull-ups.\0",
            in_signature: "<HH\0",
            in_param_names: "\0",
            out_signature: "ep0_max_packet_size, quirk_flags\0",
            out_param_names: "\0",
            command_handler: dummy_handler, // connect
        },
        Verb {
            id: 0x1,
            name: "disconnect\0",
            doc: "Disconnect the target port from the host.\0",
            in_signature: "\0",
            in_param_names: "\0",
            out_signature: "\0",
            out_param_names: "\0",
            command_handler: dummy_handler, // disconnect
        },
        Verb {
            id: 0x2,
            name: "bus_reset\0",
            doc: "Cause the target device to handle a bus reset.\0",
            in_signature: "\0",
            in_param_names: "\0",
            out_signature: "\0",
            out_param_names: "\0",
            command_handler: dummy_handler, // bus_reset
        },

        // - enumeration / setup --
        Verb {
            id: 0x3,
            name: "set_address\0",
            doc: "Set the address of the target device.\nIf deferred is set this action won't complete until the setup phase ends.\0",
            in_signature: "<BB\0",
            in_param_names: "address, deferred\0",
            out_signature: "\0",
            out_param_names: "\0",
            command_handler: dummy_handler, // set_address
        },
        Verb {
            id: 0x4,
            name: "set_up_endpoints\0",
            doc: "Set up all of the non-control endpoints for the device.\0",
            in_signature: "<*(BHB)\0",
            in_param_names: "endpoint_descriptors\0",
            out_signature: "\0",
            out_param_names: "\0",
            command_handler: dummy_handler, // set_up_endpoints
        },

        // - status & control --
        Verb {
            id: 0x5,
            name: "get_status\0",
            doc: "Read one of the device's USB status registers.\0",
            in_signature: "<B\0",
            in_param_names: "register_type\0",
            out_signature: "<I\0",
            out_param_names: "register_value\0",
            command_handler: dummy_handler, // get_status
        },
        Verb {
            id: 0x6,
            name: "read_setup\0",
            doc: "Read any pending setup packets recieved on the given endpoint.\0",
            in_signature: "<B\0",
            in_param_names: "endpoint_number\0",
            out_signature: "<8X\0",
            out_param_names: "raw_setup_packet\0",
            command_handler: dummy_handler, // read_setup
        },
        Verb {
            id: 0x7,
            name: "stall_endpoint\0",
            doc: "Stall the endpoint with the provided address.\0",
            in_signature: "<B\0",
            in_param_names: "endpoint_address\0",
            out_signature: "\0",
            out_param_names: "\0",
            command_handler: dummy_handler, // stall_endpoint
        },

        // - data transfer --
        Verb {
            id: 0x8,
            name: "send_on_endpoint\0",
            doc: "Send the provided data on the given IN endpoint.\0",
            in_signature: "<B*X\0",
            in_param_names: "endpoint_number, data_to_send\0",
            out_signature: "\0",
            out_param_names: "\0",
            command_handler: dummy_handler, // send_on_endpoint
        },
        Verb {
            id: 0x9,
            name: "clean_up_transfer\0",
            doc: "Clean up any complete transfers on the given endpoint.\0",
            in_signature: "<B\0",
            in_param_names: "endpoint_address\0",
            out_signature: "\0",
            out_param_names: "\0",
            command_handler: dummy_handler, // clean_up_transfer
        },
        Verb {
            id: 0xa,
            name: "start_nonblocking_read\0",
            doc: "Begin listening for data on the given OUT endpoint.\0",
            in_signature: "<B\0",
            in_param_names: "endpoint_number\0",
            out_signature: "\0",
            out_param_names: "\0",
            command_handler: dummy_handler, // start_nonblocking_read
        },
        Verb {
            id: 0xb,
            name: "finish_nonblocking_read\0",
            doc: "Return the data read after a given non-blocking read.\0",
            in_signature: "<B\0",
            in_param_names: "endpoint_number\0",
            out_signature: "<*X\0",
            out_param_names: "read_data\0",
            command_handler: dummy_handler, // finish_nonblocking_read
        },
        Verb {
            id: 0xc,
            name: "get_nonblocking_data_length\0",
            doc: "Return the amount of data read after a given non-blocking read.\0",
            in_signature: "<B\0",
            in_param_names: "endpoint_number\0",
            out_signature: "<I\0",
            out_param_names: "length\0",
            command_handler: dummy_handler, // get_nonblocking_data_length
        },
    ]
}

// - Context ------------------------------------------------------------------

use crate::hal;
use hal::smolusb::device::UsbDevice;

pub struct Context<'a> {
    //usb1: &'a UsbDevice<'a, hal::Usb1>,
    _marker: core::marker::PhantomData<&'a ()>,
}

impl<'a> Context<'a> {
    //pub fn new(usb1: &'a UsbDevice<'a, hal::Usb1>) -> Self {
    pub fn new() -> Self {
        Self {
            //usb1
            _marker: core::marker::PhantomData
        }
    }
}

// - verb implementations: connection / disconnection -------------------------

pub fn connect<'a>(
    arguments: &[u8],
    _context: &'a Context,
) -> Result<impl Iterator<Item = u8> + 'a> {
    let iter = [].into_iter();
    Ok(iter)
}

pub fn disconnect<'a>(
    arguments: &[u8],
    _context: &'a Context,
) -> Result<impl Iterator<Item = u8> + 'a> {
    let iter = [].into_iter();
    Ok(iter)
}

pub fn bus_reset<'a>(
    arguments: &[u8],
    _context: &'a Context,
) -> Result<impl Iterator<Item = u8> + 'a> {
    let iter = [].into_iter();
    Ok(iter)
}

// - verb implementations: enumeration / setup --------------------------------

pub fn set_address<'a>(
    arguments: &[u8],
    _context: &'a Context,
) -> Result<impl Iterator<Item = u8> + 'a> {
    let iter = [].into_iter();
    Ok(iter)
}

pub fn set_up_endpoints<'a>(
    arguments: &[u8],
    _context: &'a Context,
) -> Result<impl Iterator<Item = u8> + 'a> {
    let iter = [].into_iter();
    Ok(iter)
}

// - verb implementations: status & control -----------------------------------

pub fn get_status<'a>(
    arguments: &[u8],
    _context: &'a Context,
) -> Result<impl Iterator<Item = u8> + 'a> {
    let iter = [].into_iter();
    Ok(iter)
}

pub fn read_setup<'a>(
    arguments: &[u8],
    _context: &'a Context,
) -> Result<impl Iterator<Item = u8> + 'a> {
    let iter = [].into_iter();
    Ok(iter)
}

pub fn stall_endpoint<'a>(
    arguments: &[u8],
    _context: &'a Context,
) -> Result<impl Iterator<Item = u8> + 'a> {
    let iter = [].into_iter();
    Ok(iter)
}

// - verb implementations: data transfer --------------------------------------

pub fn send_on_endpoint<'a>(
    arguments: &[u8],
    _context: &'a Context,
) -> Result<impl Iterator<Item = u8> + 'a> {
    let iter = [].into_iter();
    Ok(iter)
}

pub fn clean_up_transfer<'a>(
    arguments: &[u8],
    _context: &'a Context,
) -> Result<impl Iterator<Item = u8> + 'a> {
    let iter = [].into_iter();
    Ok(iter)
}

pub fn start_nonblocking_read<'a>(
    arguments: &[u8],
    _context: &'a Context,
) -> Result<impl Iterator<Item = u8> + 'a> {
    let iter = [].into_iter();
    Ok(iter)
}

pub fn finish_nonblocking_read<'a>(
    arguments: &[u8],
    _context: &'a Context,
) -> Result<impl Iterator<Item = u8> + 'a> {
    let iter = [].into_iter();
    Ok(iter)
}

pub fn get_nonblocking_data_length<'a>(
    arguments: &[u8],
    _context: &'a Context,
) -> Result<impl Iterator<Item = u8> + 'a> {
    let iter = [].into_iter();
    Ok(iter)
}
