#![allow(dead_code, unused_imports, unused_mut, unused_variables)]

#![no_std]
#![no_main]

// - panic_handler ------------------------------------------------------------

use core::panic::PanicInfo;

#[no_mangle]
#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::ptr::write_volatile(IO_LEDS as *mut u32, 0b11_1100) };
    loop { }
}

#[export_name = "ExceptionHandler"]
fn custom_exception_handler(panic_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::ptr::write_volatile(IO_LEDS as *mut u32, 0b11_1110) };
    loop { }
}

// - start of day -------------------------------------------------------------

core::arch::global_asm!(r#"
.section .init
_start:
    // flush icache
    .word(0x100f)
    nop
    nop
    nop
    nop
    nop

    // flush dcache
    .word(0x500f)

    // global pointer
    .option push
    .option norelax
    la gp, __global_pointer$
    .option pop

    // stack pointer
    la sp, __stack_top
    add s0, sp, zero

    // jump to main
    jal zero, main

    // either here or below
    nop
"#);


// - main ---------------------------------------------------------------------
#[link_section = ".text"]
#[no_mangle]
pub unsafe extern "C" fn main() -> ! {
    const MSG: &'static str = "Entering main loop.";
    uart_tx(MSG);

    let mut counter = 0;
    loop {
        unsafe { asm::delay(1_000_000) };
        unsafe { core::ptr::write_volatile(IO_LEDS as *mut u32, counter & 0b11_1111) };
        counter += 1;
    }
}

// - helpers ------------------------------------------------------------------

mod asm {
    pub unsafe fn delay(cycles: u32) {
        let real_cyc = 1 + cycles / 2;
        core::arch::asm!(
            "1:",
            "addi {0}, {0}, -1",
            "bne {0}, zero, 1b",
            inout(reg) real_cyc => _,
            options(nomem, nostack),
        )
    }
}

// - peripherals --------------------------------------------------------------

const IO_BASE: usize = 0xf000_0000;
const IO_LEDS: usize = IO_BASE + 0x1000;
const IO_UART_TX_DAT: usize = IO_BASE + 0x2000 + 0x0010;
const IO_UART_TX_RDY: usize = IO_BASE + 0x2000 + 0x0014;

fn uart_tx(s: &str) {
    for b in s.bytes() {
        while unsafe { core::ptr::read_volatile(IO_UART_TX_RDY as *mut u32) } == 0 { }
        unsafe { core::ptr::write_volatile(IO_UART_TX_DAT as *mut u32, b as u32 & 0b1111_1111) };
    }
}
