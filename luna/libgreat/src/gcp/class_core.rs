use super::{ClassId, Command, Verb};

use log::{trace, error};
use zerocopy::{AsBytes, BigEndian, ByteSlice, FromBytes, LittleEndian, Unaligned, U32};

use core::any::Any;
use core::slice;

pub static CLASS_DOCS: &str =
    "Core API used to query information about the device, and perform a few standard functions.";

fn dummy_handler<'a>(_arguments: &[u8], _context: &'a dyn Any) -> slice::Iter<'a, u8> {
    [].iter()
}

pub const fn verbs<'a>() -> [Verb<'a>; 1] {
    [
        Verb {
            id: 0x0,
            name: "read_board_id",
            doc: "Return the board id.",
            in_signature: "",
            in_param_names: "",
            out_signature: "",
            out_param_names: "",
            command_handler: dummy_handler, //read_board_id,
        },
        /*Verb {
            id: 0x1,
            name: "read_version_string",
            doc: "Return the board version string.",
            in_signature: "",
            in_param_names: "",
            out_signature: "",
            out_param_names: "",
            command_handler: read_version_string,
        },
        Verb {
            id: 0x2,
            name: "read_part_id",
            doc: "Return the board part id.",
            in_signature: "",
            in_param_names: "",
            out_signature: "",
            out_param_names: "",
            command_handler: read_part_id,
        },
        Verb {
            id: 0x3,
            name: "read_serial_number",
            doc: "Return the board serial number.",
            in_signature: "",
            in_param_names: "",
            out_signature: "",
            out_param_names: "",
            command_handler: read_serial_number,
        },
        // - api introspection --
        Verb {
            id: 0x4,
            name: "get_available_classes",
            doc: "Return the classes supported by the board.",
            in_signature: "",
            in_param_names: "",
            out_signature: "",
            out_param_names: "",
            command_handler: get_available_classes,
        },
        Verb {
            id: 0x5,
            name: "get_available_verbs",
            doc: "Return the verbs supported by the given class.",
            in_signature: "<I",
            in_param_names: "class_number",
            out_signature: "",
            out_param_names: "",
            command_handler: get_available_verbs,
        },
        Verb {
            id: 0x6,
            name: "get_verb_name",
            doc: "Return the name of the given class and verb.",
            in_signature: "<II",
            in_param_names: "class_number, verb_number",
            out_signature: "",
            out_param_names: "",
            command_handler: get_verb_name,
        },
        Verb {
            id: 0x7,
            name: "get_verb_descriptor",
            doc: "Returns the descriptor of the given class, verb and descriptor.",
            in_signature: "<III",
            in_param_names: "class_number, verb_number, descriptor_number",
            out_signature: "",
            out_param_names: "",
            command_handler: get_verb_descriptor,
        },
        Verb {
            id: 0x8,
            name: "get_class_name",
            doc: "Return the name of the given class.",
            in_signature: "<I",
            in_param_names: "class_number",
            out_signature: "",
            out_param_names: "",
            command_handler: get_class_name,
        },
        Verb {
            id: 0x9,
            name: "get_class_docs",
            doc: "Return the documentation for the given class.",
            in_signature: "<I",
            in_param_names: "class_number",
            out_signature: "",
            out_param_names: "",
            command_handler: get_class_docs,
        },*/
    ]
}

// - verb implementations: board ----------------------------------------------

pub fn man_read_board_id<'a>(board_information: &'a crate::firmware::BoardInformation) -> impl Iterator<Item = u8> + 'a {
    let board_id = board_information.board_id;
    trace!("  sending board id: {:?}", board_information.board_id);
    board_information.board_id.into_iter()
}

fn read_board_id<'a>(arguments: &[u8], _context: &'a dyn Any) -> impl Iterator<Item = u8> + 'a {
    static BOARD_ID: [u8; 4] = [0x00, 0x00, 0x00, 0x00];
    trace!("  sending board id: {:?}", BOARD_ID);
    BOARD_ID.into_iter()
}

fn read_version_string<'a>(arguments: &[u8], _context: &'a dyn Any) -> impl Iterator<Item = u8> + 'a {
    static VERSION_STRING: &[u8] = "v2021.2.1\0".as_bytes();
    trace!("  sending version string: {:?}", VERSION_STRING);
    VERSION_STRING.into_iter().copied()
}

fn read_part_id<'a>(arguments: &[u8], _context: &'a dyn Any) -> impl Iterator<Item = u8> + 'a {
    // TODO this should probably come from the SoC
    static PART_ID: [u8; 8] = [0x30, 0xa, 0x00, 0xa0, 0x5e, 0x4f, 0x60, 0x00];
    trace!("  sending part id: {:?}", PART_ID);
    PART_ID.into_iter()
}

fn read_serial_number<'a>(arguments: &[u8], _context: &'a dyn Any) -> impl Iterator<Item = u8> + 'a {
    // TODO this should probably come from the SoC
    static SERIAL_NUMBER: [u8; 16] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe6, 0x67, 0xcc, 0x57, 0x57, 0x53, 0x6f,
        0x30,
    ];
    trace!("  sending part id: {:?}", SERIAL_NUMBER);
    SERIAL_NUMBER.into_iter()
}

// - verb implementations: api ------------------------------------------------

pub fn man_get_available_classes<'a>(classes: &'a crate::gcp::Classes) -> impl Iterator<Item = u8> + 'a {
    [].into_iter()
}

fn get_available_classes<'a>(arguments: &[u8], _context: &'a dyn Any) -> impl Iterator<Item = u8> + 'a {
    // can't set alignment
    /*#[repr(C)]
    #[derive(Debug, AsBytes)]
    struct Classes {
        classes: [u32; 3] // not aligned
    }
    static CLASSES: Classes = Classes {
        classes: [0x0, 0x1, 0x2]
    };
    CLASSES.as_bytes()*/

    // can't return as ref
    /*static CLASSES: [u32; 2] = [ 0x0, 0x1 ];
    let mut response = [0; 8];
    for (dest, source) in response.chunks_exact_mut(4).zip(CLASSES.iter()) {
        dest.copy_from_slice(&source.to_le_bytes())
    }
    response.iter()*/

    // TODO iter ?

    [].into_iter()
}

fn get_class_name<'a>(arguments: &[u8], _context: &'a dyn Any) -> impl Iterator<Item = u8> + 'a {
    [].into_iter()
}

fn get_class_docs<'a>(arguments: &[u8], _context: &'a dyn Any) -> impl Iterator<Item = u8> + 'a {
    [].into_iter()
}

fn get_available_verbs<'a>(arguments: &[u8], _context: &'a dyn Any) -> impl Iterator<Item = u8> + 'a {
    [].into_iter()
}

fn get_verb_name<'a>(arguments: &[u8], _context: &'a dyn Any) -> impl Iterator<Item = u8> + 'a {
    [].into_iter()
}

fn get_verb_descriptor<'a>(arguments: &[u8], _context: &'a dyn Any) -> impl Iterator<Item = u8> + 'a {
    #[repr(C)]
    #[derive(Debug, FromBytes, Unaligned)]
    struct Args {
        class: U32<LittleEndian>,
        verb: U32<LittleEndian>,
        descriptor: u8,
    }
    match Args::read_from(arguments) {
        Some(arguments) => {
            //let context = _context.downcast_ref::<(u32, u32, u32)>().expect("argh");
            //context.0 *= 2;
            //context.1 *= 3;
            //context.2 *= 4;
            [].into_iter()
        }
        None => {
            error!("get_verb_descriptor received invalid arguments");
            [].into_iter()
        }
    }
}
