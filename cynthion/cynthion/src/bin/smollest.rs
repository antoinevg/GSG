#![allow(dead_code)]

#![no_std]
#![no_main]

use core::fmt::Write;
use cynthion::pac;

#[cfg(feature = "vexriscv")]
#[riscv_rt::pre_init]
unsafe fn pre_main() {
    pac::cpu::vexriscv::flush_icache();
    #[cfg(feature = "vexriscv_dcache")]
    pac::cpu::vexriscv::flush_dcache();
}

#[riscv_rt::entry]
fn main() -> ! {
    let mut serial = Writer;

    writeln!(serial, "Formatted output: 0x{:08x} addr", IO_LEDS).unwrap();
    writeln!(serial, "Entering main loop.").unwrap();

    let mut counter = 0;
    loop {
        unsafe { riscv::asm::delay(1_000_000) };
        unsafe { core::ptr::write_volatile(IO_LEDS as *mut _, counter & 0b11_1111) };
        counter += 1;
    }
}

// - peripheral registers -----------------------------------------------------

const IO_BASE: usize = 0x8000_0000;
const IO_LEDS: usize = IO_BASE + 0x0080;
const IO_UART_TX_DATA: usize = IO_BASE + 0x0010;
const IO_UART_TX_RDY: usize = IO_BASE + 0x0014;

// - core::fmt::Write ---------------------------------------------------------

#[inline(never)]
fn uart_tx(s: &str) {
    let tx_data = IO_UART_TX_DATA as *mut u32;
    let tx_ready = IO_UART_TX_RDY as *mut u32;
    for &c in s.as_bytes() {
        while unsafe { core::ptr::read_volatile(tx_ready) } == 0 { }
        unsafe { core::ptr::write_volatile(tx_data, c as u32 & 0b1111_1111) };
    }
}

struct Writer;

impl core::fmt::Write for Writer {
    #[inline(never)]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        uart_tx(s);
        Ok(())
    }
}
