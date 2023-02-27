#![allow(unused_imports)]

use zerocopy::{AsBytes, BigEndian, LittleEndian, FromBytes, Unaligned, U32};

///! Great Communications Protocol

mod class;
pub use class::*;

// setup_packet.value
pub const LIBGREAT_REQUEST_VALUE: u32        = 0x0000;
pub const LIBGREAT_REQUEST_CANCEL_VALUE: u32 = 0xDEAD;

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
        let bytes: [u8; 8] = [
            0x01, 0x00, 0x00, 0x00,
            0x02, 0x00, 0x00, 0x00,
        ];
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
            class_core,
            class_reserved,
            core_read_version_string,
            core_reserved,
        );

        assert_eq!(class_core, Class::core);
        assert_eq!(class_reserved, Class::reserved(1));
        assert_eq!(core_read_version_string, Core::read_version_string);
        assert_eq!(core_reserved, Core::reserved(0x20));
    }
}
