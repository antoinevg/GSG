//! Support for various vendor defined softcore extensions.

pub mod minerva {

    //! Micro-architecture specific CSR extensions for the Minerva RISC-V
    //! soft processor.
    //!
    //! See: [ISA definition](https://github.com/minerva-cpu/minerva/blob/master/minerva/isa.py)
    //!
    //! These are somewhat weird because peripheral irq enable (0x330)
    //! overlaps with the Machine Counter Setup `mhpmevent16`
    //! performance-monitoring event selector.
    //!
    //! See: [Chapter 2 - Control and Status Registers](https://riscv.org/wp-content/uploads/2017/05/riscv-privileged-v1.10.pdf)

    /// Machine IRQ Mask
    pub mod mim {
        crate::macros::read_csr_as_usize!(0x330);
        crate::macros::write_csr_as_usize!(0x330);
    }

    // Machine IRQ Pending
    pub mod mip {
        crate::macros::read_csr_as_usize!(0x360);
    }
}
