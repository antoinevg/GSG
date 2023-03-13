use super::{ClassId, Command, Verb, VerbDescriptor};

use crate::error::{GreatError, Result};

use log::{error, trace};
use zerocopy::{AsBytes, BigEndian, ByteSlice, FromBytes, LittleEndian, Unaligned, U32};

use core::any::Any;
use core::slice;

pub static CLASS_DOCS: &str =
    "Core API\0"; // used to query information about the device, and perform a few standard functions.\0";

fn dummy_handler<'a>(_arguments: &[u8], _context: &'a dyn Any) -> slice::Iter<'a, u8> {
    [].iter()
}

pub const fn verbs<'a>() -> [Verb<'a>; 10] {
    [
        Verb {
            id: 0x0,
            name: "read_board_id\0",
            doc: "Return the board id.\0",
            in_signature: "\0",
            in_param_names: "\0",
            out_signature: "\0",
            out_param_names: "\0",
            command_handler: old_read_board_id,
        },
        Verb {
            id: 0x1,
            name: "read_version_string\0",
            doc: "Return the board version string.\0",
            in_signature: "\0",
            in_param_names: "\0",
            out_signature: "\0",
            out_param_names: "\0",
            command_handler: dummy_handler, //read_version_string,
        },
        Verb {
            id: 0x2,
            name: "read_part_id\0",
            doc: "Return the board part id.\0",
            in_signature: "\0",
            in_param_names: "\0",
            out_signature: "\0",
            out_param_names: "\0",
            command_handler: dummy_handler, //read_part_id,
        },
        Verb {
            id: 0x3,
            name: "read_serial_number\0",
            doc: "Return the board serial number.\0",
            in_signature: "\0",
            in_param_names: "\0",
            out_signature: "\0",
            out_param_names: "\0",
            command_handler: dummy_handler, //read_serial_number,
        },
        // - api introspection --
        Verb {
            id: 0x4,
            name: "get_available_classes\0",
            doc: "Return the classes supported by the board.\0",
            in_signature: "\0",
            in_param_names: "\0",
            out_signature: "\0",
            out_param_names: "\0",
            command_handler: dummy_handler, //get_available_classes,
        },
        Verb {
            id: 0x5,
            name: "get_available_verbs\0",
            doc: "Return the verbs supported by the given class.\0",
            in_signature: "<I\0",
            in_param_names: "class_number\0",
            out_signature: "\0",
            out_param_names: "\0",
            command_handler: dummy_handler, //get_available_verbs,
        },
        Verb {
            id: 0x6,
            name: "get_verb_name\0",
            doc: "Return the name of the given class and verb.\0",
            in_signature: "<II\0",
            in_param_names: "class_number, verb_number\0",
            out_signature: "\0",
            out_param_names: "\0",
            command_handler: dummy_handler, //get_verb_name,
        },
        Verb {
            id: 0x7,
            name: "get_verb_descriptor\0",
            doc: "Returns the descriptor of the given class, verb and descriptor.\0",
            in_signature: "<III\0",
            in_param_names: "class_number, verb_number, descriptor_number\0",
            out_signature: "\0",
            out_param_names: "\0",
            command_handler: dummy_handler, //get_verb_descriptor,
        },
        Verb {
            id: 0x8,
            name: "get_class_name\0",
            doc: "Return the name of the given class.\0",
            in_signature: "<I\0",
            in_param_names: "class_number\0",
            out_signature: "\0",
            out_param_names: "\0",
            command_handler: dummy_handler, //get_class_name,
        },
        Verb {
            id: 0x9,
            name: "get_class_docs\0",
            doc: "Return the documentation for the given class.\0",
            in_signature: "<I\0",
            in_param_names: "class_number\0",
            out_signature: "\0",
            out_param_names: "\0",
            command_handler: dummy_handler, //get_class_docs,
        },
    ]
}

// - verb implementations: board ----------------------------------------------

fn old_read_board_id<'a>(arguments: &[u8], _context: &'a dyn Any) -> slice::Iter<'a, u8> {
    static BOARD_ID: [u8; 4] = [0x00, 0x00, 0x00, 0x00];
    trace!("  sending board id: {:?}", BOARD_ID);
    BOARD_ID.iter()
}

pub fn read_board_id<'a>(
    _arguments: &[u8],
    board_information: &'a crate::firmware::BoardInformation,
) -> Result<impl Iterator<Item = u8> + 'a> {
    let board_id = board_information.board_id;
    trace!("  sending board id: {:?}", board_information.board_id);
    Ok(board_information.board_id.into_iter())
}

pub fn read_version_string<'a>(
    _arguments: &[u8],
    board_information: &'a crate::firmware::BoardInformation,
) -> Result<impl Iterator<Item = u8> + 'a> {
    let version_string = board_information.version_string;
    trace!("  sending version string: {:?}", version_string);
    Ok(version_string.as_bytes().into_iter().copied())
}

pub fn read_part_id<'a>(
    _arguments: &[u8],
    board_information: &'a crate::firmware::BoardInformation,
) -> Result<impl Iterator<Item = u8> + 'a> {
    let part_id = board_information.part_id;
    trace!("  sending part id: {:?}", part_id);
    Ok(part_id.into_iter())
}

pub fn read_serial_number<'a>(
    _arguments: &[u8],
    board_information: &'a crate::firmware::BoardInformation,
) -> Result<impl Iterator<Item = u8> + 'a> {
    let serial_number = board_information.serial_number;
    trace!("  sending serial number: {:?}", serial_number);
    Ok(serial_number.into_iter())
}

// - verb implementations: api ------------------------------------------------

fn old_get_available_classes<'a>(arguments: &[u8], _context: &'a dyn Any) -> slice::Iter<'a, u8> {
    static CLASSES: [u8; 8] = [
        0x0, 0x0, 0x0, 0x0,
        0x1, 0x0, 0x0, 0x0,
    ];
    CLASSES.iter()
}

pub fn get_available_classes<'a, 'b>(
    _arguments: &'a [u8],
    classes: &'b crate::gcp::Classes<'b>,
) -> Result<impl Iterator<Item = u8> + 'b> {
    let classes = classes
        .iter()
        .flat_map(|class| class.id.into_u32().to_le_bytes());
    Ok(classes)
}

pub fn get_available_verbs<'a, 'b>(
    arguments: &[u8],
    classes: &'b crate::gcp::Classes<'b>,
) -> Result<impl Iterator<Item = u8> + 'b> {
    #[repr(C)]
    #[derive(FromBytes, Unaligned)]
    struct Args {
        class_number: U32<LittleEndian>,
    }
    let args = Args::read_from(arguments).ok_or(&GreatError::GcpInvalidArguments)?;
    let class = classes
        .class(args.class_number.into())
        .ok_or(&GreatError::GcpClassNotFound)?;
    let verbs = class.verbs.iter().flat_map(|verb| verb.id.to_le_bytes());
    Ok(verbs)
}

pub fn get_verb_name<'a, 'b>(
    arguments: &[u8],
    classes: &'b crate::gcp::Classes<'b>,
) -> Result<impl Iterator<Item = u8> + 'b> {
    #[repr(C)]
    #[derive(FromBytes, Unaligned)]
    struct Args {
        class_number: U32<LittleEndian>,
        verb_number: U32<LittleEndian>,
    }
    let args = Args::read_from(arguments).ok_or(&GreatError::GcpInvalidArguments)?;
    let class = classes
        .class(args.class_number.into())
        .ok_or(&GreatError::GcpClassNotFound)?;
    let verb = class
        .verb(args.verb_number.into())
        .ok_or(&GreatError::GcpVerbNotFound)?;
    Ok(verb.name.as_bytes().into_iter().copied())
}

pub fn get_verb_descriptor<'a, 'b>(
    arguments: &[u8],
    classes: &'b crate::gcp::Classes<'b>,
) -> Result<impl Iterator<Item = u8> + 'b> {
    #[repr(C)]
    #[derive(Debug, FromBytes, Unaligned)]
    struct Args {
        class_number: U32<LittleEndian>,
        verb_number: U32<LittleEndian>,
        descriptor: u8,
    }
    let args = Args::read_from(arguments).ok_or(&GreatError::GcpInvalidArguments)?;
    let class = classes
        .class(args.class_number.into())
        .ok_or(&GreatError::GcpClassNotFound)?;
    let verb = class
        .verb(args.verb_number.into())
        .ok_or(&GreatError::GcpVerbNotFound)?;
    match args.descriptor.into() {
        VerbDescriptor::InSignature => Ok(verb.in_signature.as_bytes().into_iter().copied()),
        VerbDescriptor::InParamNames => Ok(verb.in_param_names.as_bytes().into_iter().copied()),
        VerbDescriptor::OutSignature => Ok(verb.out_signature.as_bytes().into_iter().copied()),
        VerbDescriptor::OutParamNames => Ok(verb.out_param_names.as_bytes().into_iter().copied()),
        VerbDescriptor::Doc => Ok(verb.doc.as_bytes().into_iter().copied()),
        VerbDescriptor::Unknown(_value) => Err(&GreatError::GcpUnknownVerbDescriptor),
    }
}

pub fn get_class_name<'a, 'b>(
    arguments: &[u8],
    classes: &'b crate::gcp::Classes<'b>,
) -> Result<impl Iterator<Item = u8> + 'b> {
    trace!("  get_class_name: {:?}", arguments);
    #[repr(C)]
    #[derive(FromBytes, Unaligned)]
    struct Args {
        class_number: U32<LittleEndian>,
    }
    let args = Args::read_from(arguments).ok_or(&GreatError::GcpInvalidArguments)?;
    let class = classes
        .class(args.class_number.into())
        .ok_or(&GreatError::GcpClassNotFound)?;
    Ok(class.name.as_bytes().iter().copied())
}

pub fn get_class_docs<'a, 'b>(
    arguments: &[u8],
    classes: &'b crate::gcp::Classes<'b>,
) -> Result<impl Iterator<Item = u8> + 'b> {
    #[repr(C)]
    #[derive(FromBytes, Unaligned)]
    struct Args {
        class_number: U32<LittleEndian>,
    }
    let args = Args::read_from(arguments).ok_or(&GreatError::GcpInvalidArguments)?;
    let class = classes
        .class(args.class_number.into())
        .ok_or(&GreatError::GcpClassNotFound)?;
    Ok(class.docs.as_bytes().into_iter().copied())
}
