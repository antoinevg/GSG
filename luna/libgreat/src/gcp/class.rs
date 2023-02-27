use zerocopy::{AsBytes, BigEndian, LittleEndian, FromBytes, Unaligned, U32};

///! Great Communications Protocol Class Registry

/// Class
#[repr(u32)]
#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Class {
    core = 0x0,
    reserved(u32),
}

impl core::convert::From<u32> for Class {
    fn from(class: u32) -> Self {
        match class {
            0x0 => Class::core,
            _ => Class::reserved(class)
        }
    }
}

impl core::convert::From<U32<LittleEndian>> for Class {
    fn from(value: U32<LittleEndian>) -> Self {
        Class::from(value.get())
    }
}

/// Verbs for class: Core
#[repr(u32)]
#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Core {
    read_board_id = 0x0,
    read_version_string = 0x1,
    read_part_id = 0x2,
    read_serial_number = 0x3,
    get_available_classes = 0x4,
    get_available_verbs = 0x5,
    get_verb_name = 0x6,
    get_verb_descriptor = 0x7,
    get_class_name = 0x8,
    get_class_docs = 0x9,

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
            _ => Core::reserved(verb)
        }
    }
}

impl core::convert::From<U32<LittleEndian>> for Core {
    fn from(value: U32<LittleEndian>) -> Self {
        Core::from(value.get())
    }
}
