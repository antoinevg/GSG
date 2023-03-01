use log::{debug, error};
use zerocopy::{AsBytes, BigEndian, FromBytes, LittleEndian, Unaligned, U32};

/// Verbs for class: Core
#[repr(u32)]
#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Core {
    // - board information --

    ///
    ///
    read_board_id = 0x0,
    ///
    ///
    read_version_string = 0x1,
    ///
    ///
    read_part_id = 0x2,
    ///
    ///
    read_serial_number = 0x3,

    // - api introspection --

    ///
    ///
    get_available_classes = 0x4,
    ///
    ///
    get_available_verbs = 0x5,
    ///
    ///
    get_verb_name = 0x6,
    ///
    ///
    get_verb_descriptor = 0x7,
    ///
    ///
    get_class_name = 0x8,
    ///
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
    pub fn handle(&self, class: super::Class, verb: Core) -> &[u8] {
        match verb {
            Core::read_board_id => read_board_id(),
            Core::read_version_string => read_version_string(),
            Core::read_part_id => read_part_id(),
            Core::read_serial_number => read_serial_number(),
            //class_core::Core::reserved(_id) => {
            _ => {
                error!("unknown verb: {:?}.{:?}", class, verb);
                &[]
            }
        }
    }
}

// - verb implementations -----------------------------------------------------

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
