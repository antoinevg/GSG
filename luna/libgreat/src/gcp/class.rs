///! Great Communications Protocol Class Registry
///!
use crate::error::{GreatError, GreatResult};

use super::class_core;
use super::{Command, CommandPrelude};

use log::{debug, error};
use zerocopy::{AsBytes, BigEndian, FromBytes, LittleEndian, Unaligned, U32};

use core::any::Any;
use core::slice;

// - Classes ------------------------------------------------------------------

/// Classes
#[derive(Copy, Clone)]
pub struct Classes(pub &'static [Class]);

impl Classes {
    pub fn class(&self, id: ClassId) -> Option<&Class> {
        self.0.iter().find(|&class| class.id == id)
    }

    pub fn new() -> Self {
        Self(&[])
    }
}

impl core::ops::Deref for Classes {
    type Target = &'static [Class];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// - Class --------------------------------------------------------------------

#[derive(Copy, Clone)]
pub struct Class {
    pub id: ClassId,
    pub name: &'static str,
    pub docs: &'static str,
    pub verbs: &'static [Verb],
}

impl Class {
    pub fn verb(&self, id: u32) -> Option<&Verb> {
        self.verbs.iter().find(|&verb| verb.id == id)
    }
}

// - TODO CommandHandler ------------------------------------------------------

/*pub trait CommandHandler {
    type Context;
    fn dispatch(&mut self, context: Self::Context) -> Self::Context;
}

struct SomeCommand;

impl CommandHandler for SomeCommand {
    type Context = u32;
    fn dispatch(&mut self, mut context: Self::Context) -> Self::Context {
        context *= 23;
        context
    }
}*/

// - Verb ---------------------------------------------------------------------

/// Verb
#[derive(Copy, Clone)]
pub struct Verb {
    pub id: u32,
    pub name: &'static str,
    pub in_signature: &'static str,
    pub in_param_names: &'static str,
    pub out_signature: &'static str,
    pub out_param_names: &'static str,
    pub doc: &'static str,
    //pub command_handler: fn(arguments: &[u8], context: &'a dyn Any) -> slice::Iter<'a, u8>,
    //pub command_handler: fn(arguments: &[u8], _context: &'a dyn Any) -> impl Iterator<Item = u8>,
}

/// Verb Descriptor
#[repr(u8)]
pub enum VerbDescriptor {
    OutSignature = 0,
    InSignature = 1,
    Doc = 2,
    OutParamNames = 3,
    InParamNames = 4,
    Unknown(u8),
}

impl core::convert::From<u8> for VerbDescriptor {
    fn from(value: u8) -> Self {
        use VerbDescriptor::*;
        match value {
            0 => OutSignature,
            1 => InSignature,
            2 => Doc,
            3 => OutParamNames,
            4 => InParamNames,
            _ => Unknown(value),
        }
    }
}

// - ClassId ------------------------------------------------------------------

/// ClassId
#[repr(u32)]
#[derive(Debug, PartialEq, Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum ClassId {
    core = 0x0000,
    firmware = 0x0001,
    gpio = 0x0103,
    greatdancer = 0x0104,
    moondancer = 0x0120,
    unsupported(u32),
}

impl core::convert::From<u32> for ClassId {
    fn from(value: u32) -> Self {
        match value {
            0x0000 => ClassId::core,
            0x0001 => ClassId::firmware,
            0x0103 => ClassId::gpio,
            0x0104 => ClassId::greatdancer,
            0x0120 => ClassId::moondancer,
            _ => ClassId::unsupported(value),
        }
    }
}

impl ClassId {
    pub const fn into_u32(&self) -> u32 {
        match self {
            ClassId::core => 0x0000,
            ClassId::firmware => 0x0001,
            ClassId::gpio => 0x0103,
            ClassId::greatdancer => 0x0104,
            ClassId::moondancer => 0x0120,
            ClassId::unsupported(value) => *value,
        }
    }
}

/*impl From<Class> for u32 {
    fn from(class: Class) -> Self {
        class.into_u32()
    }
}*/

impl core::convert::From<U32<LittleEndian>> for ClassId {
    fn from(value: U32<LittleEndian>) -> Self {
        ClassId::from(value.get())
    }
}
