use log::{debug, error};
use zerocopy::{AsBytes, BigEndian, FromBytes, LittleEndian, Unaligned, U32};

/// Verbs for class: Greatdancer
#[repr(u32)]
#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Greatdancer {

    // - connection / disconnection --

    /// Sets up the target port to connect to a host.
    /// Enables the target port's USB pull-ups.
    /// .in_signature = "<HH", .out_signature = "", .in_param_names = "ep0_max_packet_size, quirk_flags",
    connect = 0x0,
    /// Disconnects the target port from the host.
    /// .in_signature = "", .out_signature = ""
    disconnect = 0x1,
    /// Causes the target device to handle a bus reset.
    /// Usually issued when the host requests a bus reset.
    /// .in_signature = "", .out_signature = ""
    bus_reset = 0x2,

    // - enumeration / setup --

    /// Sets the address of the target device.
    /// If deferred is set, this action won't complete until setup phase ends
    /// .in_signature = "<BB", .out_signature = "", .in_param_names = "address, deferred",
    set_address = 0x3,
    /// Sets up all of the non-control endpoints for the device.
    /// Accepts endpoint triplets of (address, max_packet_size, transfer_type).
    /// in_signature = "<*(BHB)", .out_signature = "", .in_param_names = "endpoint_descriptors"
    set_up_endpoints = 0x4,

    // - status & control --

    /// Reads one of the device's USB status registers.
    /// .in_signature = "<B", .out_signature = "<I", .in_param_names = "register_type", .out_param_names = "register_value"
    get_status = 0x5,
    /// Reads any pending setup packets recieved on the given endpoint.
    /// .in_signature = "<B", .out_signature = "<8X", .in_param_names = "endpoint_number", .out_param_names = "raw_setup_packet"
    read_setup = 0x6,
    /// Stalls the endpoint with the provided address.
    /// .in_signature = "<B", .out_signature = "", .in_param_names = "endpoint_address",
    stall_endpoint = 0x7,

    // - data transfers --

    /// Sends the provided data on the given IN endpoint.
    /// .in_signature = "<B*X", .out_signature = "", .in_param_names = "endpoint_number, data_to_send"
    send_on_endpoint = 0x8,
    /// Cleans up any complete transfers on the given endpoint.
    /// .handler = greatdancer_verb_clean_up_transfer, .in_signature = "<B", .out_signature = "", .in_param_names = "endpoint_address",
    clean_up_transfer = 0x9,
    /// Begins listening for data on the given OUT endpoint.
    /// .in_signature = "<B", .out_signature = "", .in_param_names = "endpoint_number",
    start_nonblocking_read = 0x10,
    /// Returns the data read after a given non-blocking read.
    /// .in_signature = "<B", .out_signature = "<*X", .in_param_names = "endpoint_number", .out_param_names = "read_data",
    finish_nonblocking_read = 0x11,
    /// Returns the amount of data read after a given non-blocking read.
    /// .in_signature = "<B", .out_signature = "<I", .in_param_names = "endpoint_number", .out_param_names = "length",
    get_nonblocking_data_length = 0x12,

    /// Unsupported verb
    unsupported(u32),
}

impl core::convert::From<u32> for Greatdancer {
    fn from(verb: u32) -> Self {
        match verb {
            0x0 => Greatdancer::connect,
            0x1 => Greatdancer::disconnect,
            0x2 => Greatdancer::bus_reset,
            0x3 => Greatdancer::set_address,
            0x4 => Greatdancer::set_up_endpoints,
            0x5 => Greatdancer::get_status,
            0x6 => Greatdancer::read_setup,
            0x7 => Greatdancer::stall_endpoint,
            0x8 => Greatdancer::send_on_endpoint,
            0x9 => Greatdancer::clean_up_transfer,
            0x10 => Greatdancer::start_nonblocking_read,
            0x11 => Greatdancer::finish_nonblocking_read,
            0x12 => Greatdancer::get_nonblocking_data_length,
            _ => Greatdancer::unsupported(verb),
        }
    }
}

impl core::convert::From<U32<LittleEndian>> for Greatdancer {
    fn from(value: U32<LittleEndian>) -> Self {
        Greatdancer::from(value.get())
    }
}

/// Dispatch
pub struct Dispatch {}

impl Dispatch {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Dispatch {
    pub fn handle(&self, class: super::Class, verb: Greatdancer) -> &[u8] {
        match verb {
            Greatdancer::connect => connect(),
            _ => {
                error!("unknown verb: {:?}.{:?}", class, verb);
                &[]
            }
        }
    }
}

// - verb implementations -----------------------------------------------------

pub fn connect() -> &'static [u8] {
    &[]
}
