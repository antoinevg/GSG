#![allow(dead_code, unused_imports, unused_variables)] // TODO

use zerocopy::{
    AsBytes, BigEndian, ByteSlice, ByteSliceMut, FromBytes, LayoutVerified, LittleEndian,
    Unaligned, U32,
};

///! Great Communications Protocol
pub mod class;

pub use class::Class;

/// CommandPrelude
#[repr(C)]
#[derive(Debug, FromBytes, AsBytes, Unaligned)]
pub struct CommandPrelude {
    pub class: U32<LittleEndian>,
    pub verb: U32<LittleEndian>,
}

#[derive(Debug)]
pub struct Command<B: ByteSlice> {
    pub prelude: LayoutVerified<B, CommandPrelude>,
    pub arguments: B,
}

impl<B> Command<B>
where
    B: ByteSlice,
{
    pub fn parse(byte_slice: B) -> Option<Command<B>> {
        let (prelude, arguments) = LayoutVerified::new_unaligned_from_prefix(byte_slice)?;
        Some(Command { prelude, arguments })
    }

    pub fn class(&self) -> Class {
        Class::from(self.prelude.class)
    }
}

impl<B> Command<B>
where
    B: ByteSliceMut,
{
    fn set_class(&mut self, class: u32) {
        self.prelude.class = class.into();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // - fixtures -------------------------------------------------------------

    const COMMAND_NO_ARGS: [u8; 8] = [
        // class = 1
        0x01, 0x00, 0x00, 0x00, // verb = 2
        0x02, 0x00, 0x00, 0x00,
    ];
    const COMMAND_GET_CLASS_NAME: [u8; 12] = [
        // class = 0 (core)
        0x00, 0x00, 0x00, 0x00, // verb = 8 (get_class_name)
        0x08, 0x00, 0x00, 0x00, // arg0: class_number = 1
        0x01, 0x00, 0x00, 0x00,
    ];
    const COMMAND_GET_VERB_DESCRIPTOR: [u8; 17] = [
        // class = 0 (core)
        0x00, 0x00, 0x00, 0x00, // verb = 7 (get_verb_descriptor)
        0x07, 0x00, 0x00, 0x00, // arg0: class_number = 1
        0x01, 0x00, 0x00, 0x00, // arg1: verb_number = 0
        0x00, 0x00, 0x00, 0x00, // arg2: descriptor = 1 (in_signature)
        0x01,
    ];

    // - tests ----------------------------------------------------------------

    #[test]
    fn test_as_bytes() {
        let prelude: CommandPrelude = CommandPrelude {
            class: 1.into(),
            verb: 2.into(),
        };
        let bytes: &[u8] = prelude.as_bytes();
        println!("test_as_bytes: {:?}", bytes);

        assert_eq!(bytes, [0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_from_bytes_no_args() {
        let prelude: CommandPrelude =
            CommandPrelude::read_from(&COMMAND_NO_ARGS[..]).expect("failed parsing command");
        println!("test_from_bytes: {:?}", prelude);

        assert_eq!(prelude.class.get(), 1);
        assert_eq!(prelude.verb.get(), 2);
    }

    #[test]
    fn test_parse_no_args() {
        let command = Command::parse(&COMMAND_NO_ARGS[..]).expect("failed parsing command");
        println!("test_parse_no_args: {:?}", command);

        assert_eq!(command.prelude.class.get(), 1);
        assert_eq!(command.prelude.verb.get(), 2);
    }

    #[test]
    fn test_parse_get_class_name() {
        let command = Command::parse(&COMMAND_GET_CLASS_NAME[..]).expect("failed parsing command");
        println!("test_parse_get_class_name: {:?}", command);

        assert_eq!(command.prelude.class.get(), 0);
        assert_eq!(command.prelude.verb.get(), 8);
    }

    #[test]
    fn test_parse_get_verb_descriptor() {
        let command =
            Command::parse(&COMMAND_GET_VERB_DESCRIPTOR[..]).expect("failed parsing command");
        println!("test_parse_get_verb_descriptor: {:?}", command);

        assert_eq!(command.prelude.class.get(), 0);
        assert_eq!(command.prelude.verb.get(), 7);
    }

    #[test]
    fn test_dispatch_get_verb_descriptor() {
        let command =
            Command::parse(&COMMAND_GET_VERB_DESCRIPTOR[..]).expect("failed parsing command");
        println!("test_dispatch_get_verb_descriptor: {:?}", command);

        let dispatch = class::Dispatch::new();
        let response = dispatch.dispatch(command);
        println!("  -> {:?}", response);
    }

    #[test]
    fn test_enums() {
        let class_core: Class = Class::from(0);
        let class_reserved: Class = Class::from(1);
        let core_read_version_string: class::Core = class::Core::from(1);
        let core_reserved: class::Core = class::Core::from(0x20);
        println!(
            "test_enums: {:?}, {:?}, {:?}, {:?}",
            class_core, class_reserved, core_read_version_string, core_reserved,
        );

        assert_eq!(class_core, class::Class::core);
        //assert_eq!(class_reserved, class::Class::unsupported(1));
        assert_eq!(core_read_version_string, class::Core::read_version_string);
        assert_eq!(core_reserved, class::Core::reserved(0x20));
    }

    // -

    use core::any::Any;

    #[derive(Debug, Clone, Copy)]
    struct State {
        value: u32,
    }

    struct Device {}

    impl Device {
        fn new() -> Self {
            Self {}
        }

        fn handle_setup<'a>(&self, some_state: &'a mut dyn Any) -> Option<&'a mut dyn Any> {
            if let Some(state) = some_state.downcast_mut::<State>() {
                println!("handle_setup() state: {:?}", state);
                state.value = 42;
                return Some(some_state);
            }
            Some(some_state)
        }
    }

    #[test]
    fn test_any() {
        let device = Device::new();
        let mut my_state = State { value: 23 };
        println!("my_state: {:?}", my_state);

        let any_state: Option<&mut dyn Any> = device.handle_setup(&mut my_state);
        let any_state = any_state.unwrap();
        println!("any_state: {:?}", any_state);

        if let Some(my_state) = any_state.downcast_mut::<State>() {
            println!("&mut my_state: {:?}", my_state);
        }

        println!("my_state: {:?}", my_state);

        assert_eq!(true, true);
    }
}
