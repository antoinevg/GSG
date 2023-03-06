use super::{Class, Command, VerbRecord, VerbRecordCollection};

use log::{debug, error};
use zerocopy::{AsBytes, BigEndian, FromBytes, LittleEndian, Unaligned, U32};

use core::any::Any;
use core::slice;

static CLASS_DOCS: &str
    = "Common API for updating firmware on a libgreat device.";

pub struct Verbs<'a>([VerbRecord<'a>; 10]);

impl<'a> Verbs<'a> {
    pub fn new() -> Self {
        Self([
        ])
    }
}

/// Verbs for class: Firmware
#[repr(u32)]
#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Firmware {
    /// Sets up the board to have its firmware programmed.
    /// .in_signature = "", .out_signature = "<II", .out_param_names = "page_size, total_size",
    initialize = 0x0,
    /// Erases the entire firmware flash chip.
    /// .in_signature = "", .out_signature	= ""
    full_erase = 0x1,
    /// Erases the page with the provided address on the fw flash.
    /// .in_signature = "<I", .out_signature = "", .in_param_names = "address",
    page_erase = 0x2,
    /// Writes the provided data to a single firmware flash page.
    /// .in_signature = "<I*X", .out_signature = "", .in_param_names = "address, data",
    write_page = 0x3,
    /// Returns the contents of the flash page at the given address.
    /// .in_signature = "<I", .out_signature = "<*X", .in_param_names = "address", .out_param_names = "data",
    read_page = 0x4,

    /// Unsupported verb
    unsupported(u32),
}

impl core::convert::From<u32> for Firmware {
    fn from(verb: u32) -> Self {
        match verb {
            0x0 => Firmware::initialize,
            0x1 => Firmware::full_erase,
            0x2 => Firmware::page_erase,
            0x3 => Firmware::write_page,
            0x4 => Firmware::read_page,
            _ => Firmware::unsupported(verb),
        }
    }
}

impl core::convert::From<U32<LittleEndian>> for Firmware {
    fn from(value: U32<LittleEndian>) -> Self {
        Firmware::from(value.get())
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
    pub fn handle(&self, class: super::Class, verb: Firmware) -> &[u8] {
        match verb {
            Firmware::initialize => initialize(),
            _ => {
                error!("unsupported verb: {:?}.{:?}", class, verb);
                &[]
            }
        }
    }
}

// - verb implementations -----------------------------------------------------

pub fn initialize() -> &'static [u8] {
    &[]
}
