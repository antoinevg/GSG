use super::{Class, Command};

use log::{debug, error};

use zerocopy::{AsBytes, BigEndian, ByteSlice, FromBytes, LittleEndian, Unaligned, U32};

/// Core API used to query information about the device, and perform a
/// few standard functions.
#[repr(u32)]
#[derive(Debug, PartialEq, Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum Core {
    // - board information --
    /// Return the board id
    /// .in_signature    = "",
    /// .out_signature   = TODO
    /// .out_param_names = TODO
    read_board_id = 0x0,

    /// Return the board version string
    /// .in_signature    = "",
    /// .out_signature   = TODO
    /// .out_param_names = TODO
    read_version_string = 0x1,

    /// Return the board's processor id ?
    /// .in_signature    = "",
    /// .out_signature   = TODO
    /// .out_param_names = TODO
    read_part_id = 0x2,

    /// Return the board's serial number ?
    /// .in_signature    = "",
    /// .out_signature   = TODO
    /// .out_param_names = TODO
    read_serial_number = 0x3,

    // - api introspection --
    /// Return the list of classes supported by the board.
    /// .in_signature    = "",
    /// .out_signature   = TODO
    /// .out_param_names = TODO
    get_available_classes = 0x4,

    /// Return the list of verbs supported by the given class.
    /// .in_signature   = "<I",
    /// .in_param_names = "class_number",
    /// .out_signature  = TODO,
    get_available_verbs = 0x5,

    /// Return the name of the given verb.
    /// .in_signature  = "<II",
    /// .out_signature = TODO
    get_verb_name = 0x6,

    /// Return the descriptor for the given verb and descriptor id.
    ///
    get_verb_descriptor = 0x7,

    /// Return the name of the given class.
    ///
    get_class_name = 0x8,

    /// Return the documentation for the given class.
    ///
    get_class_docs = 0x9,

    /// Unsupported verb
    reserved(u32),
}

impl core::convert::From<u32> for Core {
    fn from(verb: u32) -> Self {
        match verb {
            0x0 => Core::read_board_id,
            0x1 => Core::read_version_string,
            0x2 => Core::read_part_id,
            0x3 => Core::read_serial_number,
            0x4 => Core::get_available_classes,
            0x5 => Core::get_available_verbs,
            0x6 => Core::get_verb_name,
            0x7 => Core::get_verb_descriptor,
            0x8 => Core::get_class_name,
            0x9 => Core::get_class_docs,
            _ => Core::reserved(verb),
        }
    }
}

impl core::convert::From<U32<LittleEndian>> for Core {
    fn from(value: U32<LittleEndian>) -> Self {
        Core::from(value.get())
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
    pub fn dispatch<'a, B>(&'a self, command: Command<B>) -> &[u8]
    where
        B: ByteSlice,
    {
        let verb = Core::from(command.prelude.verb);
        println!("dispatching: {:?}", verb);
        match verb {
            Core::read_board_id => read_board_id(),
            Core::read_version_string => read_version_string(),
            Core::read_part_id => read_part_id(),
            Core::read_serial_number => read_serial_number(),

            //Core::get_available_classes => get_available_classes(command.arguments),
            Core::get_available_verbs => get_available_verbs(command.arguments),
            Core::get_verb_name => get_verb_name(command.arguments),
            Core::get_verb_descriptor => get_verb_descriptor(command.arguments),
            Core::get_class_name => get_class_name(command.arguments),
            Core::get_class_docs => get_class_docs(command.arguments),
            _ => {
                error!("unknown verb: {:?}.{:?}", command.class(), verb);
                &[]
            }
        }
    }

    pub fn handle(&self, class: super::Class, verb: Core) -> &[u8] {
        match verb {
            Core::read_board_id => read_board_id(),
            Core::read_version_string => read_version_string(),
            Core::read_part_id => read_part_id(),
            Core::read_serial_number => read_serial_number(),
            //Core::get_available_classes => get_available_classes(),
            //Core::get_available_verbs => get_available_verbs(),
            //Core::get_verb_name => get_verb_name(),
            //Core::get_verb_descriptor => get_verb_descriptor(),
            //Core::get_class_name => get_class_name(),
            //Core::get_class_docs => get_class_docs(),
            //class_core::Core::reserved(_id) => {
            _ => {
                error!("unknown verb: {:?}.{:?}", class, verb);
                &[]
            }
        }
    }
}

// - verb implementations: api ------------------------------------------------

pub fn get_available_classes<B>(arguments: B) -> &'static [u8]
//pub fn get_available_classes<B>(arguments: B) -> [u8; 8]
where
    B: ByteSlice,
{
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

    &[]
}

pub fn get_class_name<B>(arguments: B) -> &'static [u8]
where
    B: ByteSlice,
{
    &[]
}

pub fn get_class_docs<B>(arguments: B) -> &'static [u8]
where
    B: ByteSlice,
{
    &[]
}

pub fn get_available_verbs<B>(arguments: B) -> &'static [u8]
where
    B: ByteSlice,
{
    &[]
}

pub fn get_verb_name<B>(arguments: B) -> &'static [u8]
where
    B: ByteSlice,
{
    &[]
}

pub fn get_verb_descriptor<'a, B>(arguments: B) -> &'a [u8]
where
    B: ByteSlice,
{
    #[repr(C)]
    #[derive(Debug, FromBytes, Unaligned)]
    struct Args {
        class: U32<LittleEndian>,
        verb: U32<LittleEndian>,
        descriptor: u8,
    }
    match Args::read_from(arguments) {
        Some(arguments) => {
            println!("get_verb_descriptor -> {:?}", arguments);
            &[]
        }
        None => {
            error!("get_verb_descriptor received invalid arguments");
            &[]
        }
    }
}

// - verb implementations: board ----------------------------------------------

pub fn read_board_id() -> &'static [u8] {
    static BOARD_ID: [u8; 4] = [0x00, 0x00, 0x00, 0x00];
    debug!("  sending board id: {:?}", BOARD_ID);
    &BOARD_ID
}

pub fn read_version_string() -> &'static [u8] {
    static VERSION_STRING: &[u8] = "v2021.2.1\0".as_bytes();
    debug!("  sending version string: {:?}", VERSION_STRING);
    VERSION_STRING
}

pub fn read_part_id() -> &'static [u8] {
    // TODO this should probably come from the SoC
    static PART_ID: [u8; 8] = [0x30, 0xa, 0x00, 0xa0, 0x5e, 0x4f, 0x60, 0x00];
    debug!("  sending part id: {:?}", PART_ID);
    &PART_ID
}

pub fn read_serial_number() -> &'static [u8] {
    // TODO this should probably come from the SoC
    static SERIAL_NUMBER: [u8; 16] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe6, 0x67, 0xcc, 0x57, 0x57, 0x53, 0x6f,
        0x30,
    ];
    debug!("  sending part id: {:?}", SERIAL_NUMBER);
    &SERIAL_NUMBER
}
