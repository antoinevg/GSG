pub struct CSR;

pub mod interrupt {
    //! CSR access methods.

    use crate::register;
    use crate::Interrupt;

    pub unsafe fn enable(interrupt: Interrupt) {
        let mask = crate::register::minerva::mim::read();
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

    pub unsafe fn irqno() -> usize {
        register::minerva::mip::read()
    }
}
