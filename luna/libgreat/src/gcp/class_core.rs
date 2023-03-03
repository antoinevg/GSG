use super::{Class, Command, VerbRecord, VerbRecordCollection};

use log::{debug, error};
use zerocopy::{AsBytes, BigEndian, ByteSlice, FromBytes, LittleEndian, Unaligned, U32};

use core::any::Any;
use core::slice;

static CLASS_DOCS: &str
    = "Core API used to query information about the device, and perform a few standard functions.";

pub struct Verbs<'a>([VerbRecord<'a>; 10]);

impl<'a> VerbRecordCollection<'a> for Verbs<'a> {
    fn iter(&'a self) -> slice::Iter<VerbRecord> {
        self.0.iter()
    }

    fn verb(&'a self, verb_number: u32) -> &'a VerbRecord {
        self.0.iter().find(|&record| record.verb_number == verb_number).expect("could not find verb")
    }
}

impl<'a> Verbs<'a> {
    pub fn new() -> Self {
        Self([
            VerbRecord {
                verb_number: 0x0,
                name: "read_board_id",
                doc: "Return the board id.",
                in_signature: "",
                in_param_names: "",
                out_signature: "",
                out_param_names: "",
                command_handler: read_board_id,
            },
            VerbRecord {
                verb_number: 0x1,
                name: "read_version_string",
                doc: "Return the board version string.",
                in_signature: "",
                in_param_names: "",
                out_signature: "",
                out_param_names: "",
                command_handler: read_version_string,
            },
            VerbRecord {
                verb_number: 0x2,
                name: "read_part_id",
                doc: "Return the board part id.",
                in_signature: "",
                in_param_names: "",
                out_signature: "",
                out_param_names: "",
                command_handler: read_part_id,
            },
            VerbRecord {
                verb_number: 0x3,
                name: "read_serial_number",
                doc: "Return the board serial number.",
                in_signature: "",
                in_param_names: "",
                out_signature: "",
                out_param_names: "",
                command_handler: read_serial_number,
            },

            // - api introspection --

            VerbRecord {
                verb_number: 0x4,
                name: "get_available_classes",
                doc: "Return the classes supported by the board.",
                in_signature: "",
                in_param_names: "",
                out_signature: "",
                out_param_names: "",
                command_handler: get_available_classes,
            },
            VerbRecord {
                verb_number: 0x5,
                name: "get_available_verbs",
                doc: "Return the verbs supported by the given class.",
                in_signature: "<I",
                in_param_names: "class_number",
                out_signature: "",
                out_param_names: "",
                command_handler: get_available_verbs,
            },
            VerbRecord {
                verb_number: 0x6,
                name: "get_verb_name",
                doc: "Return the name of the given class and verb.",
                in_signature: "<II",
                in_param_names: "class_number, verb_number",
                out_signature: "",
                out_param_names: "",
                command_handler: get_verb_name,
            },
            VerbRecord {
                verb_number: 0x7,
                name: "get_verb_descriptor",
                doc: "Returns the descriptor of the given class, verb and descriptor.",
                in_signature: "<III",
                in_param_names: "class_number, verb_number, descriptor_number",
                out_signature: "",
                out_param_names: "",
                command_handler: get_verb_descriptor,
            },
            VerbRecord {
                verb_number: 0x8,
                name: "get_class_name",
                doc: "Return the name of the given class.",
                in_signature: "<I",
                in_param_names: "class_number",
                out_signature: "",
                out_param_names: "",
                command_handler: get_class_name,
            },
            VerbRecord {
                verb_number: 0x9,
                name: "get_class_docs",
                doc: "Return the documentation for the given class.",
                in_signature: "<I",
                in_param_names: "class_number",
                out_signature: "",
                out_param_names: "",
                command_handler: get_class_docs,
            },
        ])
    }
}


// - verb implementations: board ----------------------------------------------

fn read_board_id<'a>(arguments: &[u8], _context: &'a mut dyn Any) -> slice::Iter<'a, u8> {
    static BOARD_ID: [u8; 4] = [0x00, 0x00, 0x00, 0x00];
    debug!("  sending board id: {:?}", BOARD_ID);
    BOARD_ID.iter()
}

fn read_version_string<'a>(arguments: &[u8], _context: &'a mut dyn Any) -> slice::Iter<'a, u8> {
    static VERSION_STRING: &[u8] = "v2021.2.1\0".as_bytes();
    debug!("  sending version string: {:?}", VERSION_STRING);
    VERSION_STRING.iter()
}

fn read_part_id<'a>(arguments: &[u8], _context: &'a mut dyn Any) -> slice::Iter<'a, u8> {
    // TODO this should probably come from the SoC
    static PART_ID: [u8; 8] = [0x30, 0xa, 0x00, 0xa0, 0x5e, 0x4f, 0x60, 0x00];
    debug!("  sending part id: {:?}", PART_ID);
    PART_ID.iter()
}

fn read_serial_number<'a>(arguments: &[u8], _context: &'a mut dyn Any) -> slice::Iter<'a, u8> {
    // TODO this should probably come from the SoC
    static SERIAL_NUMBER: [u8; 16] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe6, 0x67, 0xcc, 0x57, 0x57, 0x53, 0x6f,
        0x30,
    ];
    debug!("  sending part id: {:?}", SERIAL_NUMBER);
    SERIAL_NUMBER.iter()
}


// - verb implementations: api ------------------------------------------------

fn get_available_classes<'a>(arguments: &[u8], _context: &'a mut dyn Any) -> slice::Iter<'a, u8> {
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
    response*/

    // TODO iter ?

    [].iter()
}

fn get_class_name<'a>(arguments: &[u8], _context: &'a mut dyn Any) -> slice::Iter<'a, u8> {
    [].iter()
}

fn get_class_docs<'a>(arguments: &[u8], _context: &'a mut dyn Any) -> slice::Iter<'a, u8> {
    [].iter()
}

fn get_available_verbs<'a>(arguments: &[u8], _context: &'a mut dyn Any) -> slice::Iter<'a, u8> {
    [].iter()
}

fn get_verb_name<'a>(arguments: &[u8], _context: &'a mut dyn Any) -> slice::Iter<'a, u8> {
    [].iter()
}


fn get_verb_descriptor<'a>(arguments: &[u8], _context: &'a mut dyn Any) -> slice::Iter<'a, u8> {
    #[repr(C)]
    #[derive(Debug, FromBytes, Unaligned)]
    struct Args {
        class: U32<LittleEndian>,
        verb: U32<LittleEndian>,
        descriptor: u8,
    }
    match Args::read_from(arguments) {
        Some(arguments) => {
            let context = _context.downcast_mut::<(u32, u32, u32)>().expect("argh");
            println!("get_verb_descriptor -> {:?} -> {:?}",
                     arguments, context);
            context.0 *= 2;
            context.1 *= 3;
            context.2 *= 4;
            [].iter()
        }
        None => {
            error!("get_verb_descriptor received invalid arguments");
           [].iter()
        }
    }
}
