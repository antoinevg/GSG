#![allow(dead_code)]

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


// - macros -------------------------------------------------------------------

macro_rules! read_csr {
    ($csr_number:literal) => {
        #[inline]
        unsafe fn _read() -> usize {
            match () {
                () => {
                    let r: usize;
                    core::arch::asm!(concat!("csrrs {0}, ", stringify!($csr_number), ", x0"), out(reg) r);
                    r
                }
            }
        }
    };
}

macro_rules! write_csr {
    ($csr_number:literal) => {
        #[inline]
        #[allow(unused_variables)]
        unsafe fn _write(bits: usize) {
            match () {
                () => core::arch::asm!(concat!("csrrw x0, ", stringify!($csr_number), ", {0}"), in(reg) bits),
            }
        }
    };
}

macro_rules! read_csr_as_usize {
    ($csr_number:literal) => {
        read_csr!($csr_number);

        #[inline]
        pub fn read() -> usize {
            unsafe { _read() }
        }
    };
}

macro_rules! write_csr_as_usize {
    ($csr_number:literal) => {
        write_csr!($csr_number);

        #[inline]
        pub fn write(bits: usize) {
            unsafe { _write(bits) }
        }
    };
}


// - mim - machine irq mask ---------------------------------------------------

pub mod mim {
    read_csr_as_usize!(0x330);
    write_csr_as_usize!(0x330);
}


// - mip - machine irq pending ------------------------------------------------

pub mod mip {
    read_csr_as_usize!(0x360);
}
