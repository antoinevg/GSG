///! Great Communications Protocol Class Registry
pub mod class_core;
pub mod firmware;
pub mod gpio;
pub mod greatdancer;
pub mod moondancer;

pub use class_core::Core;
pub use firmware::Firmware;
pub use gpio::Gpio;
pub use greatdancer::Greatdancer;
pub use moondancer::Moondancer;

use super::CommandPrelude;

use log::error;
use zerocopy::{AsBytes, BigEndian, FromBytes, LittleEndian, Unaligned, U32};

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

impl core::convert::From<U32<LittleEndian>> for Class {
    fn from(value: U32<LittleEndian>) -> Self {
        Class::from(value.get())
    }
}

/// Dispatch
pub struct Dispatch {
    core: class_core::Dispatch,
    firmware: firmware::Dispatch,
    gpio: gpio::Dispatch,
    greatdancer: greatdancer::Dispatch,
    moondancer: moondancer::Dispatch,
}

impl Dispatch {
    pub const fn new() -> Self {
        Self {
            core: class_core::Dispatch::new(),
            firmware: firmware::Dispatch::new(),
            gpio: gpio::Dispatch::new(),
            greatdancer: greatdancer::Dispatch::new(),
            moondancer: moondancer::Dispatch::new(),
        }
    }
}

impl Dispatch {
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
