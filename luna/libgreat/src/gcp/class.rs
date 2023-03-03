///! Great Communications Protocol Class Registry

// - Class --------------------------------------------------------------------

use super::{Command, CommandPrelude};
use super::class_core;

use log::{debug, error};
use zerocopy::{AsBytes, BigEndian, FromBytes, LittleEndian, Unaligned, U32};

use core::any::Any;
use core::slice;

/// Class
#[repr(u32)]
#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Class {
    core = 0x0000,
    firmware = 0x0001,
    gpio = 0x0103,
    greatdancer = 0x0104,
    moondancer = 0x0120,
    unsupported(u32),
}

impl core::convert::From<u32> for Class {
    fn from(class: u32) -> Self {
        match class {
            0x0000 => Class::core,
            0x0001 => Class::firmware,
            0x0103 => Class::gpio,
            0x0104 => Class::greatdancer,
            0x0120 => Class::moondancer,
            _ => Class::unsupported(class),
        }
    }
}

impl From<Class> for u32 {
    fn from(class: Class) -> Self {
        match class {
            Class::core => 0x0000,
            Class::firmware => 0x0001,
            Class::gpio => 0x0103,
            Class::greatdancer => 0x0104,
            Class::moondancer => 0x0120,
            Class::unsupported(value) => value,
        }
    }
}

impl core::convert::From<U32<LittleEndian>> for Class {
    fn from(value: U32<LittleEndian>) -> Self {
        Class::from(value.get())
    }
}


// - VerbRecord ---------------------------------------------------------------

pub trait VerbRecordCollection<'a> {
    fn iter(&'a self) -> slice::Iter<VerbRecord>;
    fn verb(&'a self, verb_number: u32) -> &'a VerbRecord;
}

/// VerbRecord
pub struct VerbRecord<'a> {
    pub verb_number: u32,
    pub name: &'a str,
    pub in_signature: &'a str,
    pub in_param_names: &'a str,
    pub out_signature: &'a str,
    pub out_param_names: &'a str,
    pub doc: &'a str,
    pub command_handler: fn(arguments: &[u8], context: &'a mut dyn Any) -> slice::Iter<'a, u8>,
}

/*
impl<'a, 'b> VerbRecord<'a, 'b> {
    pub fn new() -> Self {
        fn empty_command_handler<'a, 'b>(_arguments: &[u8], _context: &'b mut dyn Any) -> slice::Iter<'a, u8> {
            [].iter()
        }
        Self {
            verb_number: 0,
            name: "",
            in_signature: "",
            in_param_names: "",
            out_signature: "",
            out_param_names: "",
            doc: "",
            command_handler: empty_command_handler,
        }
    }
}
*/

// - Class AltDispatch -----------------------------------------------------------

/// Dispatch
pub struct AltDispatch<'a> {
    core: class_core::Verbs<'a>,
    //firmware: firmware::Dispatch,
}

impl<'a> AltDispatch<'a> {
    pub fn new() -> Self {
        Self {
            core: class_core::Verbs::new(),
            //firmware: firmware::Dispatch::new(),
        }
    }
}

impl<'a> AltDispatch<'a> {
    pub fn dispatch<B>(&'a self, command: Command<B>, context: &'a mut dyn Any) -> slice::Iter<'a, u8>
    where
        B: zerocopy::ByteSlice,
    {
        let class = command.class();
        let verb_number = command.prelude.verb.get();
        match class {
            Class::core => {
                //self.core.dispatch(command)
                //let record = &alt_core::VERBS[verb_number];
                let record = &self.core.verb(verb_number);
                let handler = record.command_handler;
                let arguments = command.arguments.as_bytes();
                let response = handler(arguments, context);
                response
            },
            _ => {
                unimplemented!()
            }
        }
    }
}


// - Class Dispatch -----------------------------------------------------------
/*

/// Dispatch
pub struct OldDispatch {
    core: old_core::Dispatch,
    firmware: firmware::Dispatch,
    gpio: gpio::Dispatch,
    greatdancer: greatdancer::Dispatch,
    moondancer: moondancer::Dispatch,
}

impl OldDispatch {
    pub const fn new() -> Self {
        Self {
            core: old_core::Dispatch::new(),
            firmware: firmware::Dispatch::new(),
            gpio: gpio::Dispatch::new(),
            greatdancer: greatdancer::Dispatch::new(),
            moondancer: moondancer::Dispatch::new(),
        }
    }
}

impl OldDispatch {
    pub fn dispatch<'a, B>(&'a self, command: Command<B>) -> &[u8]
    where
        B: zerocopy::ByteSlice,
    {
        let class = command.class();
        match class {
            Class::core => self.core.dispatch(command),
            _ => {
                unimplemented!()
            }
        }
    }

    pub fn handle(&self, command_prelude: CommandPrelude) -> &[u8] {
        let class = Class::from(command_prelude.class);
        match class {
            Class::core => {
                let verb = Core::from(command_prelude.verb);
                self.core.handle(class, verb)
            }
            Class::firmware => {
                let verb = Firmware::from(command_prelude.verb);
                self.firmware.handle(class, verb)
            }
            Class::gpio => {
                let verb = Gpio::from(command_prelude.verb);
                self.gpio.handle(class, verb)
            }
            Class::greatdancer => {
                let verb = Greatdancer::from(command_prelude.verb);
                self.greatdancer.handle(class, verb)
            }
            Class::moondancer => {
                let verb = Moondancer::from(command_prelude.verb);
                self.moondancer.handle(class, verb)
            }
            Class::unsupported(_id) => {
                error!("unsupported class: {:?}", class);
                &[]
            }
        }
    }
}
*/
