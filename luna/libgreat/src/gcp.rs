#![allow(dead_code, unused_imports, unused_variables)] // TODO

use zerocopy::{AsBytes, BigEndian, FromBytes, LittleEndian, Unaligned, U32};

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

#[cfg(test)]
mod tests {
    use super::*;

    // - fixtures -------------------------------------------------------------

    // - tests ----------------------------------------------------------------

    #[test]
    fn test_from_bytes() {
        let bytes: [u8; 8] = [0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00];
        let prelude: CommandPrelude = CommandPrelude::read_from(&bytes[..]).expect("failed");
        println!("test_from_bytes: {:?}", prelude);

        assert_eq!(prelude.class.get(), 1);
        assert_eq!(prelude.verb.get(), 2);
    }

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
    fn test_enums() {
        let class_core: Class = Class::from(0);
        let class_reserved: Class = Class::from(1);
        let core_read_version_string: Core = Core::from(1);
        let core_reserved: Core = Core::from(0x20);
        println!(
            "test_enums: {:?}, {:?}, {:?}, {:?}",
            class_core, class_reserved, core_read_version_string, core_reserved,
        );

        assert_eq!(class_core, Class::core);
        assert_eq!(class_reserved, Class::unsupported(1));
        assert_eq!(core_read_version_string, Core::read_version_string);
        assert_eq!(core_reserved, Core::reserved(0x20));
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
