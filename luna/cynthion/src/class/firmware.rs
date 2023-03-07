#![allow(dead_code, unused_imports)] // TODO

use libgreat::gcp::Verb;

use log::{debug, error};
use zerocopy::{AsBytes, BigEndian, FromBytes, LittleEndian, Unaligned, U32};

use core::any::Any;
use core::slice;

pub static CLASS_DOCS: &str = "Common API for updating firmware on a libgreat device.";

pub fn verbs<'a>() -> [Verb<'a>; 5] {
    [
        Verb {
            id: 0x0,
            name: "initialize",
            doc: "Prepares the board up to have its firmware programmed.",
            in_signature: "",
            in_param_names: "",
            out_signature: "<II",
            out_param_names: "page_size, total_size",
            command_handler: initialize,
        },
        Verb {
            id: 0x1,
            name: "full_erase",
            doc: "Erase the entire firmware flash chip.",
            in_signature: "",
            in_param_names: "",
            out_signature: "",
            out_param_names: "",
            command_handler: full_erase,
        },
        Verb {
            id: 0x2,
            name: "page_erase",
            doc: "Erase the page with the given address on the firmware flash chip.",
            in_signature: "<I",
            in_param_names: "address",
            out_signature: "",
            out_param_names: "",
            command_handler: page_erase,
        },
        Verb {
            id: 0x3,
            name: "write_page",
            doc: "Write the provided data to a single firmware flash page.",
            in_signature: "<I*X",
            in_param_names: "address, data",
            out_signature: "",
            out_param_names: "",
            command_handler: write_page,
        },
        Verb {
            id: 0x4,
            name: "read_page",
            doc: "Return the content of the flash page at the given address.",
            in_signature: "<I",
            in_param_names: "address",
            out_signature: "<*X",
            out_param_names: "data",
            command_handler: read_page,
        },
    ]
}

// - verb implementations -----------------------------------------------------

fn initialize<'a>(_arguments: &[u8], _context: &'a dyn Any) -> slice::Iter<'a, u8> {
    [].iter()
}

fn full_erase<'a>(_arguments: &[u8], _context: &'a dyn Any) -> slice::Iter<'a, u8> {
    [].iter()
}

fn page_erase<'a>(_arguments: &[u8], _context: &'a dyn Any) -> slice::Iter<'a, u8> {
    [].iter()
}

fn write_page<'a>(_arguments: &[u8], _context: &'a dyn Any) -> slice::Iter<'a, u8> {
    [].iter()
}

fn read_page<'a>(_arguments: &[u8], _context: &'a dyn Any) -> slice::Iter<'a, u8> {
    [].iter()
}
