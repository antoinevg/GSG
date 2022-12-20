#![no_std]
#![no_main]

// - libnotquitegreatyet ------------------------------------------------------

const IO_BASE: usize = 0x00005000;
const IO_LEDS: usize = IO_BASE + 0x7000;

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
}


// - panic_handler ------------------------------------------------------------

use core::panic::PanicInfo;

#[panic_handler]
#[no_mangle]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}


// - entry point --------------------------------------------------------------

#[export_name = "_start"]
#[no_mangle]
pub unsafe extern "C" fn main() -> ! {
    let io_leds = IO_LEDS as *mut _;

    loop {
        for n in 0..63 {
            *io_leds = n;
            asm::delay(1_000_000);
        }
    }
}
