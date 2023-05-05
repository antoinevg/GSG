#![no_std]
#![no_main]

// - entry point --------------------------------------------------------------

#[export_name = "_start"]
#[no_mangle]
pub unsafe extern "C" fn main() -> ! {
    let io_leds = IO_LEDS as *mut _;

    loop {
        for n in 0..64 {
            core::ptr::write_volatile(io_leds, n);
            timer_delay(500_000);
        }
        uart_tx("boink\n");
    }
}

// - panic_handler ------------------------------------------------------------

use core::panic::PanicInfo;

#[panic_handler]
#[no_mangle]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// - libnotquitegreatyet ------------------------------------------------------

const IO_BASE: usize = 0x8000_0000;

const IO_UART_TX_DATA: usize = IO_BASE + 0x0010;
const IO_UART_TX_RDY: usize = IO_BASE + 0x0014;

const IO_TIMER_RELOAD: usize = IO_BASE + 0x1000;
const IO_TIMER_EN: usize = IO_BASE + 0x1004;
const IO_TIMER_CTR: usize = IO_BASE + 0x1008;

const IO_LEDS: usize = IO_BASE + 0x0080;

pub unsafe fn timer_delay(cycles: u32) {
    let reload = IO_TIMER_RELOAD as *mut u32;
    let enable = IO_TIMER_EN as *mut u32;
    let counter = IO_TIMER_CTR as *mut u32;

    core::ptr::write_volatile(enable, 1);
    core::ptr::write_volatile(reload, cycles);

    while core::ptr::read_volatile(counter) > 0 {
        asm::nop();
    }

    core::ptr::write_volatile(enable, 0);
    core::ptr::write_volatile(reload, 0);
}

pub unsafe fn uart_tx(string: &str) {
    let tx_data = IO_UART_TX_DATA as *mut u32;
    let tx_ready = IO_UART_TX_RDY as *mut u32;

    for c in string.chars() {
        while core::ptr::read_volatile(tx_ready) == 0 {
            asm::nop();
        }
        core::ptr::write_volatile(tx_data, c as u32);
    }
}

mod asm {
    #[inline]
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

    #[inline]
    pub unsafe fn nop() {
        core::arch::asm!("nop", options(nomem, nostack),)
    }
}
