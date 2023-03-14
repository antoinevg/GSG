#![allow(dead_code, unused_imports, unused_variables)] // TODO

use libgreat::error::{GreatError, Result};
use libgreat::gcp::{self, Verb};

use log::{debug, error};
use zerocopy::{AsBytes, BigEndian, FromBytes, LittleEndian, Unaligned, U32};

use core::any::Any;
use core::slice;

pub static CLASS: gcp::Class = gcp::Class {
    id: gcp::ClassId::greatdancer,
    name: "greatdancer",
    docs: CLASS_DOCS,
    verbs: &VERBS,
};

pub static CLASS_DOCS: &str = "Common API for updating firmware on a libgreat device.\0";

pub static VERBS: [Verb; 13] = [
    // - connection / disconnection
    Verb {
        id: 0x0,
        name: "connect\0",
        doc: "Set up the target port to connect to a host.\nEnables the target port's USB pull-ups.\0",
        in_signature: "<HH\0",
        in_param_names: "\0",
        out_signature: "ep0_max_packet_size, quirk_flags\0",
        out_param_names: "\0",
        //command_handler: dummy_handler, // connect
    },
    Verb {
        id: 0x1,
        name: "disconnect\0",
        doc: "Disconnect the target port from the host.\0",
        in_signature: "\0",
        in_param_names: "\0",
        out_signature: "\0",
        out_param_names: "\0",
        //command_handler: dummy_handler, // disconnect
    },
    Verb {
        id: 0x2,
        name: "bus_reset\0",
        doc: "Cause the target device to handle a bus reset.\0",
        in_signature: "\0",
        in_param_names: "\0",
        out_signature: "\0",
        out_param_names: "\0",
        //command_handler: dummy_handler, // bus_reset
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
        //command_handler: dummy_handler, // set_address
    },
    Verb {
        id: 0x4,
        name: "set_up_endpoints\0",
        doc: "Set up all of the non-control endpoints for the device.\0",
        in_signature: "<*(BHB)\0",
        in_param_names: "endpoint_descriptors\0",
        out_signature: "\0",
        out_param_names: "\0",
        //command_handler: dummy_handler, // set_up_endpoints
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
        //command_handler: dummy_handler, // get_status
    },
    Verb {
        id: 0x6,
        name: "read_setup\0",
        doc: "Read any pending setup packets recieved on the given endpoint.\0",
        in_signature: "<B\0",
        in_param_names: "endpoint_number\0",
        out_signature: "<8X\0",
        out_param_names: "raw_setup_packet\0",
        //command_handler: dummy_handler, // read_setup
    },
    Verb {
        id: 0x7,
        name: "stall_endpoint\0",
        doc: "Stall the endpoint with the provided address.\0",
        in_signature: "<B\0",
        in_param_names: "endpoint_address\0",
        out_signature: "\0",
        out_param_names: "\0",
        //command_handler: dummy_handler, // stall_endpoint
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
        //command_handler: dummy_handler, // send_on_endpoint
    },
    Verb {
        id: 0x9,
        name: "clean_up_transfer\0",
        doc: "Clean up any complete transfers on the given endpoint.\0",
        in_signature: "<B\0",
        in_param_names: "endpoint_address\0",
        out_signature: "\0",
        out_param_names: "\0",
        //command_handler: dummy_handler, // clean_up_transfer
    },
    Verb {
        id: 0xa,
        name: "start_nonblocking_read\0",
        doc: "Begin listening for data on the given OUT endpoint.\0",
        in_signature: "<B\0",
        in_param_names: "endpoint_number\0",
        out_signature: "\0",
        out_param_names: "\0",
        //command_handler: dummy_handler, // start_nonblocking_read
    },
    Verb {
        id: 0xb,
        name: "finish_nonblocking_read\0",
        doc: "Return the data read after a given non-blocking read.\0",
        in_signature: "<B\0",
        in_param_names: "endpoint_number\0",
        out_signature: "<*X\0",
        out_param_names: "read_data\0",
        //command_handler: dummy_handler, // finish_nonblocking_read
    },
    Verb {
        id: 0xc,
        name: "get_nonblocking_data_length\0",
        doc: "Return the amount of data read after a given non-blocking read.\0",
        in_signature: "<B\0",
        in_param_names: "endpoint_number\0",
        out_signature: "<I\0",
        out_param_names: "length\0",
        //command_handler: dummy_handler, // get_nonblocking_data_length
    },
];

fn dummy_handler<'a>(_arguments: &[u8], _context: &'a dyn Any) -> slice::Iter<'a, u8> {
    [].iter()
}

// - Greatdancer --------------------------------------------------------------

use crate::hal;
use core::cell::RefCell;
use hal::smolusb::device::UsbDevice;

pub struct Greatdancer<'a> {
    usb0: UsbDevice<'a, hal::Usb0>,
    state: RefCell<State>,
    _marker: core::marker::PhantomData<&'a ()>,
}

impl<'a> Greatdancer<'a> {
    pub fn new(usb0: UsbDevice<'a, hal::Usb0>) -> Self {
        Self {
            usb0,
            state: State::default().into(),
            _marker: core::marker::PhantomData,
        }
    }
}

#[derive(Default)]
struct State {
    foo: u32,
}

// - verb implementations: connection / disconnection -------------------------

impl<'a> Greatdancer<'a> {
    pub fn connect(&self, arguments: &[u8]) -> Result<impl Iterator<Item = u8> + 'a> {
        let iter = [].into_iter();
        Ok(iter)
    }

    pub fn disconnect(&self, arguments: &[u8]) -> Result<impl Iterator<Item = u8> + 'a> {
        let iter = [].into_iter();
        Ok(iter)
    }

    pub fn bus_reset(&self, arguments: &[u8]) -> Result<impl Iterator<Item = u8> + 'a> {
        self.usb0.reset();
        Ok([].into_iter())
    }
}

// - verb implementations: enumeration / setup --------------------------------

impl<'a> Greatdancer<'a> {
    pub fn set_address(&self, arguments: &[u8]) -> Result<impl Iterator<Item = u8> + 'a> {
        let iter = [].into_iter();
        Ok(iter)
    }

    pub fn set_up_endpoints(&self, arguments: &[u8]) -> Result<impl Iterator<Item = u8> + 'a> {
        let iter = [].into_iter();
        Ok(iter)
    }
}

// - verb implementations: status & control -----------------------------------

impl<'a> Greatdancer<'a> {
    pub fn get_status(&self, arguments: &[u8]) -> Result<impl Iterator<Item = u8> + 'a> {
        let iter = [].into_iter();
        Ok(iter)
    }

    pub fn read_setup(&self, arguments: &[u8]) -> Result<impl Iterator<Item = u8> + 'a> {
        let iter = [].into_iter();
        Ok(iter)
    }

    pub fn stall_endpoint(&self, arguments: &[u8]) -> Result<impl Iterator<Item = u8> + 'a> {
        let iter = [].into_iter();
        Ok(iter)
    }
}

// - verb implementations: data transfer --------------------------------------

impl<'a> Greatdancer<'a> {
    pub fn send_on_endpoint(&self, arguments: &[u8]) -> Result<impl Iterator<Item = u8> + 'a> {
        let iter = [].into_iter();
        Ok(iter)
    }

    pub fn clean_up_transfer(&self, arguments: &[u8]) -> Result<impl Iterator<Item = u8> + 'a> {
        let iter = [].into_iter();
        Ok(iter)
    }

    pub fn start_nonblocking_read(
        &self,
        arguments: &[u8],
    ) -> Result<impl Iterator<Item = u8> + 'a> {
        let iter = [].into_iter();
        Ok(iter)
    }

    pub fn finish_nonblocking_read(
        &self,
        arguments: &[u8],
    ) -> Result<impl Iterator<Item = u8> + 'a> {
        let iter = [].into_iter();
        Ok(iter)
    }

    pub fn get_nonblocking_data_length(
        &self,
        arguments: &[u8],
    ) -> Result<impl Iterator<Item = u8> + 'a> {
        let iter = [].into_iter();
        Ok(iter)
    }
}

// - dispatch -----------------------------------------------------------------

use libgreat::gcp::{iter_to_response, GcpResponse, GCP_MAX_RESPONSE_LENGTH};

use core::{array, iter};

impl<'a> Greatdancer<'a> {
    pub fn dispatch(
        &self,
        verb_id: u32,
        arguments: &[u8],
        response_buffer: [u8; GCP_MAX_RESPONSE_LENGTH],
    ) -> Result<GcpResponse> {
        match verb_id {
            0x0 => {
                // greatdancer::connect
                let iter = self.connect(arguments)?;
                let response = unsafe { iter_to_response(iter, response_buffer) };
                Ok(response)
            }
            0x1 => {
                // greatdancer::disconnect
                let iter = self.disconnect(arguments)?;
                let response = unsafe { iter_to_response(iter, response_buffer) };
                Ok(response)
            }
            0x2 => {
                // greatdancer::bus_reset
                let iter = self.bus_reset(arguments)?;
                let response = unsafe { iter_to_response(iter, response_buffer) };
                Ok(response)
            }
            0x3 => {
                // greatdancer::set_address
                let iter = self.set_address(arguments)?;
                let response = unsafe { iter_to_response(iter, response_buffer) };
                Ok(response)
            }
            0x4 => {
                // greatdancer::set_up_endpoints
                let iter = self.set_up_endpoints(arguments)?;
                let response = unsafe { iter_to_response(iter, response_buffer) };
                Ok(response)
            }
            0x5 => {
                // greatdancer::get_status
                let iter = self.get_status(arguments)?;
                let response = unsafe { iter_to_response(iter, response_buffer) };
                Ok(response)
            }
            0x6 => {
                // greatdancer::read_setup
                let iter = self.read_setup(arguments)?;
                let response = unsafe { iter_to_response(iter, response_buffer) };
                Ok(response)
            }
            0x7 => {
                // greatdancer::stall_endpoint
                let iter = self.stall_endpoint(arguments)?;
                let response = unsafe { iter_to_response(iter, response_buffer) };
                Ok(response)
            }
            0x8 => {
                // greatdancer::send_on_endpoint
                let iter = self.send_on_endpoint(arguments)?;
                let response = unsafe { iter_to_response(iter, response_buffer) };
                Ok(response)
            }
            0x9 => {
                // greatdancer::clean_up_transfer
                let iter = self.clean_up_transfer(arguments)?;
                let response = unsafe { iter_to_response(iter, response_buffer) };
                Ok(response)
            }
            0xa => {
                // greatdancer::start_nonblocking_read
                let iter = self.start_nonblocking_read(arguments)?;
                let response = unsafe { iter_to_response(iter, response_buffer) };
                Ok(response)
            }
            0xb => {
                // greatdancer::finish_nonblocking_read
                let iter = self.finish_nonblocking_read(arguments)?;
                let response = unsafe { iter_to_response(iter, response_buffer) };
                Ok(response)
            }
            0xc => {
                // greatdancer::get_nonblocking_data_length
                let iter = self.get_nonblocking_data_length(arguments)?;
                let response = unsafe { iter_to_response(iter, response_buffer) };
                Ok(response)
            }

            _ => Err(&GreatError::Message("class: greatdancer - verb not found")),
        }
    }
}
