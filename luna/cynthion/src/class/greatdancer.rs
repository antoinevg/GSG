#![allow(dead_code, unused_imports, unused_variables)] // TODO

use libgreat::error::{GreatError, GreatResult};
use libgreat::gcp::{self, Verb};

use log::{debug, error, warn};
use zerocopy::{AsBytes, BigEndian, FromBytes, LittleEndian, Unaligned, U16, U32};

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
    },
    Verb {
        id: 0x1,
        name: "disconnect\0",
        doc: "Disconnect the target port from the host.\0",
        in_signature: "\0",
        in_param_names: "\0",
        out_signature: "\0",
        out_param_names: "\0",
    },
    Verb {
        id: 0x2,
        name: "bus_reset\0",
        doc: "Cause the target device to handle a bus reset.\0",
        in_signature: "\0",
        in_param_names: "\0",
        out_signature: "\0",
        out_param_names: "\0",
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
    },
    Verb {
        id: 0x4,
        name: "set_up_endpoints\0",
        doc: "Set up all of the non-control endpoints for the device.\0",
        in_signature: "<*(BHB)\0",
        in_param_names: "endpoint_descriptors\0",
        out_signature: "\0",
        out_param_names: "\0",
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
    },
    Verb {
        id: 0x6,
        name: "read_setup\0",
        doc: "Read any pending setup packets recieved on the given endpoint.\0",
        in_signature: "<B\0",
        in_param_names: "endpoint_number\0",
        out_signature: "<8X\0",
        out_param_names: "raw_setup_packet\0",
    },
    Verb {
        id: 0x7,
        name: "stall_endpoint\0",
        doc: "Stall the endpoint with the provided address.\0",
        in_signature: "<B\0",
        in_param_names: "endpoint_address\0",
        out_signature: "\0",
        out_param_names: "\0",
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
    },
    Verb {
        id: 0x9,
        name: "clean_up_transfer\0",
        doc: "Clean up any complete transfers on the given endpoint.\0",
        in_signature: "<B\0",
        in_param_names: "endpoint_address\0",
        out_signature: "\0",
        out_param_names: "\0",
    },
    Verb {
        id: 0xa,
        name: "start_nonblocking_read\0",
        doc: "Begin listening for data on the given OUT endpoint.\0",
        in_signature: "<B\0",
        in_param_names: "endpoint_number\0",
        out_signature: "\0",
        out_param_names: "\0",
    },
    Verb {
        id: 0xb,
        name: "finish_nonblocking_read\0",
        doc: "Return the data read after a given non-blocking read.\0",
        in_signature: "<B\0",
        in_param_names: "endpoint_number\0",
        out_signature: "<*X\0",
        out_param_names: "read_data\0",
    },
    Verb {
        id: 0xc,
        name: "get_nonblocking_data_length\0",
        doc: "Return the amount of data read after a given non-blocking read.\0",
        in_signature: "<B\0",
        in_param_names: "endpoint_number\0",
        out_signature: "<I\0",
        out_param_names: "length\0",
    },
];

// - Greatdancer --------------------------------------------------------------

use crate::hal;
use core::cell::RefCell;
use hal::smolusb::device::UsbDevice;

// TODO unify with GCP_MAX_RESPONSE_LENGTH
const MAX_PACKET_BUFFER_LENGTH: usize = 128;

pub struct Greatdancer<'a> {
    usb0: UsbDevice<'a, hal::Usb0>,
    packet_buffer: [u8; MAX_PACKET_BUFFER_LENGTH],
    state: RefCell<State>,
    _marker: core::marker::PhantomData<&'a ()>,
}

impl<'a> Greatdancer<'a> {
    pub fn new(usb0: UsbDevice<'a, hal::Usb0>) -> Self {
        Self {
            usb0,
            packet_buffer: [0; MAX_PACKET_BUFFER_LENGTH],
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
    /// Connect the USB interface.
    pub fn connect(&self, arguments: &[u8]) -> GreatResult<impl Iterator<Item = u8> + 'a> {
        #[repr(C)]
        #[derive(FromBytes, Unaligned)]
        struct Args {
            ep0_max_packet_size: U16<LittleEndian>,
            quirk_flags: U16<LittleEndian>,
        }
        let _args = Args::read_from(arguments).ok_or(GreatError::BadMessage)?;
        let iter = [].into_iter();
        Ok(iter)
    }

    /// Terminate all existing communication and disconnects the USB interface.
    pub fn disconnect(&self, arguments: &[u8]) -> GreatResult<impl Iterator<Item = u8> + 'a> {
        let iter = [].into_iter();
        Ok(iter)
    }

    /// Perform a USB bus reset.
    pub fn bus_reset(&self, arguments: &[u8]) -> GreatResult<impl Iterator<Item = u8> + 'a> {
        self.usb0.reset();
        Ok([].into_iter())
    }
}

// - verb implementations: enumeration / setup --------------------------------

impl<'a> Greatdancer<'a> {
    pub fn set_address(&self, arguments: &[u8]) -> GreatResult<impl Iterator<Item = u8> + 'a> {
        #[repr(C)]
        #[derive(FromBytes, Unaligned)]
        struct Args {
            address: u8,
            deferred: u8,
        }
        let _args = Args::read_from(arguments).ok_or(GreatError::BadMessage)?;
        let iter = [].into_iter();
        Ok(iter)
    }

    pub fn set_up_endpoints(
        &mut self,
        arguments: &[u8],
    ) -> GreatResult<impl Iterator<Item = u8> + 'a> {
        #[repr(C)]
        #[derive(FromBytes, Unaligned)]
        struct ArgEndpoint {
            address: u8,
            max_packet_size: U16<LittleEndian>,
            transfer_type: u8,
        }

        // while we have endpoint triplets to handle
        let mut byte_slice = arguments;
        while let Some((endpoint, next)) =
            zerocopy::LayoutVerified::<_, ArgEndpoint>::new_from_prefix(byte_slice)
        {
            byte_slice = next;

            // endpoint zero is always the control endpoint, and can't be configured
            if endpoint.address & 0x7f == 0x00 {
                warn!("greatdancer: ignoring request to reconfigure control endpoint");
                continue;
            }

            // ignore endpoint configurations we won't be able to handle
            if endpoint.max_packet_size.get() as usize > self.packet_buffer.len() {
                error!(
                    "greatdancer: failed to setup endpoint with max packet size {} > {}",
                    endpoint.max_packet_size,
                    self.packet_buffer.len()
                );
                return Err(GreatError::InvalidArgument);
            }

            // TODO configure endpoint
        }

        let iter = [].into_iter();
        Ok(iter)
    }
}

// - verb implementations: status & control -----------------------------------

impl<'a> Greatdancer<'a> {
    /// Query the GreatDancer for any events that need to be processed.
    /// FIXME: should this actually use an interrupt pipe?
    ///
    /// The index value is used to select which status section we're looking for:
    ///
    ///	0 = pending interrupts (USBSTS register)
    ///	1 = setup status for all endpoints (ENDPTSETUPSTAT)
    ///	2 = endpoint completion status (ENDPTCOMPLETE)
    ///	3 = endpoint primed status (ENDPTSTATUS)
    ///
    ///	Always transmits a 4-byte word back to the host.
    pub fn get_status(&self, arguments: &[u8]) -> GreatResult<impl Iterator<Item = u8> + 'a> {
        #[repr(C)]
        #[derive(FromBytes, Unaligned)]
        struct Args {
            register_type: u8,
        }
        let _args = Args::read_from(arguments).ok_or(GreatError::BadMessage)?;
        let iter = [].into_iter();
        Ok(iter)
    }

    /// Read a setup packet from the GreatDancer port and relays it to the host.
    ///
    /// The endpoint_number parameter specifies which endpoint we should be reading from.
    ///
    /// Always transmits an 8-byte setup packet back to the host. If no setup packet
    /// is waiting, the results of this vendor request are unspecified.
    pub fn read_setup(&self, arguments: &[u8]) -> GreatResult<impl Iterator<Item = u8> + 'a> {
        #[repr(C)]
        #[derive(FromBytes, Unaligned)]
        struct Args {
            endpoint_number: u8,
        }
        let _args = Args::read_from(arguments).ok_or(GreatError::BadMessage)?;
        let iter = [].into_iter();
        Ok(iter)
    }

    /// Temporarily stalls the given USB endpoint.
    pub fn stall_endpoint(&self, arguments: &[u8]) -> GreatResult<impl Iterator<Item = u8> + 'a> {
        #[repr(C)]
        #[derive(FromBytes, Unaligned)]
        struct Args {
            endpoint_number: u8,
        }
        let _args = Args::read_from(arguments).ok_or(GreatError::BadMessage)?;
        let iter = [].into_iter();
        Ok(iter)
    }
}

// - verb implementations: data transfer --------------------------------------

impl<'a> Greatdancer<'a> {
    /// Read data from the GreatFET host and sends on the provided GreatDancer endpoint.
    ///
    /// The OUT request should contain a data stage containing all data to be sent.
    pub fn send_on_endpoint(&self, arguments: &[u8]) -> GreatResult<impl Iterator<Item = u8> + 'a> {
        struct Args<B: zerocopy::ByteSlice> {
            endpoint_number: zerocopy::LayoutVerified<B, u8>,
            data_to_send: B,
        }
        let (endpoint_number, data_to_send) =
            zerocopy::LayoutVerified::new_unaligned_from_prefix(arguments)
                .ok_or(GreatError::BadMessage)?;
        let _args = Args {
            endpoint_number,
            data_to_send,
        };
        let iter = [].into_iter();
        Ok(iter)
    }

    /// Should be called whenever a transfer is complete; cleans up any transfer
    /// descriptors associated with that transfer.
    pub fn clean_up_transfer(
        &self,
        arguments: &[u8],
    ) -> GreatResult<impl Iterator<Item = u8> + 'a> {
        #[repr(C)]
        #[derive(FromBytes, Unaligned)]
        struct Args {
            endpoint_address: u8,
        }
        let args = Args::read_from(arguments).ok_or(GreatError::BadMessage)?;
        let endpoint_number = args.endpoint_address & 0x7f;

        let iter = [].into_iter();
        Ok(iter)
    }

    /// Prime the USB controller to recieve data on a particular endpoint.
    ///
    /// Does not wait for a transfer to complete. The transfer's
    /// status can be checked with `get_transfer_status` and then read
    /// with `finish_nonblocking_read`.
    pub fn start_nonblocking_read(
        &self,
        arguments: &[u8],
    ) -> GreatResult<impl Iterator<Item = u8> + 'a> {
        #[repr(C)]
        #[derive(FromBytes, Unaligned)]
        struct Args {
            endpoint_number: u8,
        }
        let _args = Args::read_from(arguments).ok_or(GreatError::BadMessage)?;
        let iter = [].into_iter();
        Ok(iter)
    }

    /// Finish a non-blocking read by returning the read data back to the host.
    ///
    /// This should only be used after determining that a transfer is
    /// complete with the `get_transfer_status` request and reading
    /// the relevant length with `get_nonblocking_data_length`.
    pub fn finish_nonblocking_read(
        &self,
        arguments: &[u8],
    ) -> GreatResult<impl Iterator<Item = u8> + 'a> {
        #[repr(C)]
        #[derive(FromBytes, Unaligned)]
        struct Args {
            endpoint_number: u8,
        }
        let _args = Args::read_from(arguments).ok_or(GreatError::BadMessage)?;
        let iter = [].into_iter();
        Ok(iter)
    }

    /// Query an endpoint to determine how much data is available.
    ///
    /// This should only be used after a nonblocking read was primed
    /// with `start_nonblocking_read` and completed by the USB
    /// hardware.
    ///
    /// Response is invalid unless a transfer has been initiated with
    /// `start_nonblocking_read` and completed.
    pub fn get_nonblocking_data_length(
        &self,
        arguments: &[u8],
    ) -> GreatResult<impl Iterator<Item = u8> + 'a> {
        #[repr(C)]
        #[derive(FromBytes, Unaligned)]
        struct Args {
            endpoint_number: u8,
        }
        let _args = Args::read_from(arguments).ok_or(GreatError::BadMessage)?;
        let iter = [].into_iter();
        Ok(iter)
    }
}

// - dispatch -----------------------------------------------------------------

use libgreat::gcp::{iter_to_response, GcpResponse, GCP_MAX_RESPONSE_LENGTH};

use core::{array, iter};

impl<'a> Greatdancer<'a> {
    pub fn dispatch(
        &mut self,
        verb_number: u32,
        arguments: &[u8],
        response_buffer: [u8; GCP_MAX_RESPONSE_LENGTH],
    ) -> GreatResult<GcpResponse> {
        match verb_number {
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

            verb_number => Err(GreatError::GcpVerbNotFound(
                gcp::class::ClassId::greatdancer,
                verb_number,
            )),
        }
    }
}
