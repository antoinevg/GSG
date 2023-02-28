use super::class_core;
use super::CommandPrelude;

use log::error;
use zerocopy::{AsBytes, BigEndian, FromBytes, LittleEndian, Unaligned, U32};

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
            _ => Class::reserved(class),
        }
    }
}

impl core::convert::From<U32<LittleEndian>> for Class {
    fn from(value: U32<LittleEndian>) -> Self {
        Class::from(value.get())
    }
}

/// Dispatch
pub struct Dispatch {
    class_core: class_core::Dispatch,
}

impl Dispatch {
    pub const fn new() -> Self {
        Self {
            class_core: class_core::Dispatch::new(),
        }
    }
}

impl Dispatch {
    pub fn handle(&self, command_prelude: CommandPrelude) -> &[u8] {
        let class = Class::from(command_prelude.class);
        match class {
            Class::core => {
                let verb = class_core::Core::from(command_prelude.verb);
                self.class_core.handle(class, verb)
            }
            Class::reserved(_id) => {
                error!("unknown class: {:?}", class);
                &[]
            }
        }
    }
}
