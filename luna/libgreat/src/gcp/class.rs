///! Great Communications Protocol Class Registry
///!
use crate::error::{GreatError, Result};

use super::class_core;
use super::{Command, CommandPrelude};

use log::{debug, error};
use zerocopy::{AsBytes, BigEndian, FromBytes, LittleEndian, Unaligned, U32};

use core::any::Any;
use core::slice;

// - Classes ------------------------------------------------------------------

/// Classes
pub struct Classes<'a>(pub &'a [Class<'a>]);

impl<'a> Classes<'a> {
    pub fn dispatch<B>(
        &'a self,
        command: Command<B>,
        context: &'a dyn Any,
    ) -> Result<slice::Iter<'a, u8>>
    where
        B: zerocopy::ByteSlice,
    {
        let class = self
            .class(command.class())
            .ok_or(&GreatError::NotFound("class not found"))?;
        let verb = class
            .verb(command.verb())
            .ok_or(&GreatError::NotFound("verb not found"))?;
        let handler = verb.command_handler;
        let arguments = command.arguments.as_bytes();
        let response = handler(arguments, context);

        Ok(response)
    }

    pub fn class(&'a self, id: ClassId) -> Option<&Class> {
        self.0.iter().find(|&class| class.id == id)
    }

    pub fn new() -> Self {
        Self(&[])
    }
}

// - Class --------------------------------------------------------------------

pub struct Class<'a> {
    pub id: ClassId,
    pub verbs: &'a [Verb<'a>],
}

impl<'a> Class<'a> {
    pub fn verb(&'a self, id: u32) -> Option<&Verb> {
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
pub struct Verb<'a> {
    pub id: u32,
    pub name: &'a str,
    pub in_signature: &'a str,
    pub in_param_names: &'a str,
    pub out_signature: &'a str,
    pub out_param_names: &'a str,
    pub doc: &'a str,
    pub command_handler: fn(arguments: &[u8], context: &'a dyn Any) -> slice::Iter<'a, u8>,
}

// - ClassId ------------------------------------------------------------------

/// Class
#[repr(u32)]
#[derive(Debug, PartialEq)]
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
