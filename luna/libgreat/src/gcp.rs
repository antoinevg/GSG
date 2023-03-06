#![allow(dead_code, unused_imports, unused_variables)] // TODO

use zerocopy::{
    AsBytes, BigEndian, ByteSlice, ByteSliceMut, FromBytes, LayoutVerified, LittleEndian,
    Unaligned, U32,
};

///! Great Communications Protocol
pub mod class;
pub mod class_core;
//pub mod old_core;

pub use class::*;

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

    use core::slice;


    // - fixtures -------------------------------------------------------------

    const COMMAND_NO_ARGS: [u8; 8] = [
        // class = 1
        0x01, 0x00, 0x00, 0x00, // verb = 2
        0x02, 0x00, 0x00, 0x00,
    ];
    const COMMAND_READ_BOARD_ID: [u8; 8] = [
        0x00, 0x00, 0x00, 0x00, // class = 0 (core)
        0x00, 0x00, 0x00, 0x00, // verb = 0 (read_board_id)
    ];
    const COMMAND_GET_CLASS_NAME: [u8; 12] = [
        0x00, 0x00, 0x00, 0x00, // class = 0 (core)
        0x08, 0x00, 0x00, 0x00, // verb = 8 (get_class_name)
        0x01, 0x00, 0x00, 0x00, // arg0: class_number = 1
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
/*
    #[test]
    fn test_enum_class() {
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
*/
    // - test_parse_* --

    #[test]
    fn test_parse_as_bytes() {
        let prelude: CommandPrelude = CommandPrelude {
            class: 1.into(),
            verb: 2.into(),
        };
        let bytes: &[u8] = prelude.as_bytes();
        println!("test_as_bytes: {:?}", bytes);

        assert_eq!(bytes, [0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_parse_from_bytes_no_args() {
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

    // - test_olddispatch_* --

    /*#[test]
    fn test_olddispatch_get_verb_descriptor() {
        let command =
            Command::parse(&COMMAND_GET_VERB_DESCRIPTOR[..]).expect("failed parsing command");
        println!("\ntest_dispatch_get_verb_descriptor: {:?}", command);

        let dispatch = class::OldDispatch::new();
        let response = dispatch.dispatch(command);
        println!("  -> {:?}\n", response);
    }*/

    // - test_dispatch_* --

    #[test]
    fn test_dispatch_read_board_id() {
        let command =
            Command::parse(&COMMAND_READ_BOARD_ID[..]).expect("failed parsing command");
        println!("\ntest_dispatch_read_board_id: {:?}", command);

        let mut context = 0;
        let dispatch = class::Dispatch::new();
        let response = dispatch.dispatch(command, &mut context);
        println!("  -> {:?}", response);

        assert_eq!(response.as_slice(), [0, 0, 0, 0]);
    }

    #[test]
    fn test_dispatch_get_verb_descriptor() {
        let command =
            Command::parse(&COMMAND_GET_VERB_DESCRIPTOR[..]).expect("failed parsing command");
        println!("\ntest_dispatch_get_verb_descriptor: {:?}", command);

        let mut context: (u32, u32, u32) = (23, 42, 12);
        let dispatch = class::Dispatch::new();
        let response = dispatch.dispatch(command, &mut context);
        println!("  -> {:?}", response);
        println!("  -> {:?}", context);

        let command =
            Command::parse(&COMMAND_GET_VERB_DESCRIPTOR[..]).expect("failed parsing command");
        let dispatch = class::Dispatch::new();
        let response = dispatch.dispatch(command, &mut context);
        println!("  -> {:?}", response);
        println!("  -> {:?}", context);
    }

    // - figure out introspection --

    struct ClassRegistry {
    }

    fn get_available_classes<'a>() -> impl Iterator<Item = u8>  {
        static CLASSES: [u32; 3] = [
            Class::core.into_u32(),
            Class::firmware.into_u32(),
            Class::gpio.into_u32(),
        ];
        CLASSES.iter().flat_map(|class| class.to_le_bytes())
    }

    fn get_available_verbs_core<'a>(verbs: &'a class_core::Verbs<'a>) -> impl Iterator<Item = u8> + 'a {
        let iter: slice::Iter<'a, VerbRecord> = verbs.iter();
        let iter = iter.map(|verb| verb.verb_number);
        let iter = iter.flat_map(|verb_number| verb_number.to_le_bytes());
        iter
    }

    #[test]
    fn test_introspection() {
        //println!("\ntest_iter: {:?}\n", classes);

        let classes = get_available_classes();
        for class in classes {
            println!("class: {}", class);
        }

        let verbs = class_core::Verbs::new();
        let verbs = get_available_verbs_core(&verbs);
        for verb in verbs {
            println!("verb: {}", verb);
        }
    }

    // - test_any --

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
