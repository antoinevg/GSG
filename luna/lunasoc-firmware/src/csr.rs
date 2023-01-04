#![allow(dead_code)]

pub mod interrupt {
    //! Support for Minerva SoC's vendor specific CSR selectors.

    use crate::pac;
    use crate::register;

    use pac::Interrupt;

    pub unsafe fn enable(interrupt: Interrupt) {
        let mask = register::minerva::mim::read();
        register::minerva::mim::write(mask | (1 << interrupt as usize))
    }

    pub unsafe fn disable(interrupt: Interrupt) {
        let mask = register::minerva::mim::read();
        register::minerva::mim::write(mask & !(1 << interrupt as usize))
    }

    pub unsafe fn pending(interrupt: Interrupt) -> bool {
        let pending = register::minerva::mip::read();
        (pending & (1 << interrupt as usize)) != 0
    }
}
