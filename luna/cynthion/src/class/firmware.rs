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

pub const fn verbs<'a>() -> [Verb<'a>; 1] {
    [
        Verb {
            id: 0x0,
            name: "initialize\0",
            doc: "Prepare the board to have its firmware programmed.\0",
            in_signature: "\0",
            in_param_names: "\0",
            out_signature: "<II\0",
            out_param_names: "page_size, total_size\0",
            command_handler: dummy_handler, // initialize,
        },
        /*Verb {
            id: 0x1,
            name: "full_erase\0",
            doc: "Erase the entire firmware flash chip.\0",
            in_signature: "\0",
            in_param_names: "\0",
            out_signature: "\0",
            out_param_names: "\0",
            command_handler: dummy_handler, // full_erase,
        },
        Verb {
            id: 0x2,
            name: "page_erase\0",
            doc: "Erase the page with the given address on the firmware flash chip.\0",
            in_signature: "<I\0",
            in_param_names: "address\0",
            out_signature: "\0",
            out_param_names: "\0",
            command_handler: dummy_handler, // page_erase,
        },
        Verb {
            id: 0x3,
            name: "write_page\0",
            doc: "Write the provided data to a single firmware flash page.\0",
            in_signature: "<I*X\0",
            in_param_names: "address, data\0",
            out_signature: "\0",
            out_param_names: "\0",
            command_handler: dummy_handler, // write_page,
        },
        Verb {
            id: 0x4,
            name: "read_page\0",
            doc: "Return the content of the flash page at the given address.\0",
            in_signature: "<I\0",
            in_param_names: "address\0",
            out_signature: "<*X\0",
            out_param_names: "data\0",
            command_handler: dummy_handler, // read_page,
        },*/
    ]
}

// - verb implementations -----------------------------------------------------

fn old_initialize<'a>(
    arguments: &[u8],
    _context: &'a dyn Any,
) -> Result<impl Iterator<Item = u8> + 'a> {
    Ok([].into_iter())
}

pub fn initialize<'a>(
    arguments: &[u8],
    _context: &'a dyn Any,
) -> Result<impl Iterator<Item = u8> + 'a> {
    let page_size: u32 = 8;
    let total_size: u32 = 1;
    let response = page_size
        .to_le_bytes()
        .into_iter()
        .chain(total_size.to_le_bytes().into_iter());
    Ok(response)
}

pub fn full_erase<'a>(
    arguments: &[u8],
    _context: &'a dyn Any,
) -> Result<impl Iterator<Item = u8> + 'a> {
    Ok([].into_iter())
}

pub fn page_erase<'a>(
    arguments: &[u8],
    _context: &'a dyn Any,
) -> Result<impl Iterator<Item = u8> + 'a> {
    #[repr(C)]
    #[derive(FromBytes, Unaligned)]
    struct Args {
        address: U32<LittleEndian>,
    }
    let _args = Args::read_from(arguments).ok_or(&GreatError::GcpInvalidArguments)?;
    Ok([].into_iter())
}

pub fn write_page<'a>(
    arguments: &[u8],
    _context: &'a dyn Any,
) -> Result<impl Iterator<Item = u8> + 'a> {
    struct Args<B: zerocopy::ByteSlice> {
        address: zerocopy::LayoutVerified<B, U32<LittleEndian>>,
        data: B,
    }
    let (address, data) = zerocopy::LayoutVerified::new_unaligned_from_prefix(
        arguments
    ).ok_or(
        &GreatError::GcpInvalidArguments
    )?;
    let _args = Args { address, data };
    Ok([].into_iter())
}

pub fn read_page<'a>(
    arguments: &[u8],
    _context: &'a dyn Any,
) -> Result<impl Iterator<Item = u8> + 'a> {
    #[repr(C)]
    #[derive(FromBytes, Unaligned)]
    struct Args {
        address: U32<LittleEndian>,
    }
    let _args = Args::read_from(arguments).ok_or(&GreatError::GcpInvalidArguments)?;
    let data: [u8; 8] = [0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0];
    Ok(data.into_iter())
}
