#![allow(unused_imports, unused_mut, unused_variables)]
#![no_std]
#![no_main]

use core::fmt::Write;
use cynthion::pac;

// - asm.S --------------------------------------------------------------------

//core::arch::global_asm!(include_str!("../../asm.S"));

// - panic_handler ------------------------------------------------------------

use core::panic::PanicInfo;

#[panic_handler]
#[no_mangle]
#[inline(never)]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// - __pre_init________________________________________________________________

/*#[no_mangle]
#[inline(never)]
pub unsafe fn __pre_init() {
    pac::cpu::vexriscv::flush_icache();
    pac::cpu::vexriscv::flush_dcache();
}*/

// - main ---------------------------------------------------------------------

const BUF8_SIZE: usize = 512;
static mut BUF8: [u8; BUF8_SIZE] = [0; BUF8_SIZE];

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn main() -> ! {
    pac::cpu::vexriscv::flush_icache();
    pac::cpu::vexriscv::flush_dcache();

    let peripherals = pac::Peripherals::steal();
    let leds = &peripherals.LEDS;
    let mut serial = Writer { uart: peripherals.UART };

    let buf8_ptr = BUF8.as_ptr() as usize;

    //writeln!(serial, "0x{:08x}", buf8_ptr).unwrap();

    //unsafe { riscv::asm::nop() };

    let mut counter: usize = 0;
    loop {
        leds.output.write(|w| unsafe { w.output().bits((counter % 63) as u8) });
        unsafe { riscv::asm::delay(1_000_000) };
        counter += 1;
    }
}

// - core::fmt::Write ---------------------------------------------------------

struct Writer {
    uart: pac::UART,
}

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for &byte in s.as_bytes() {
            self.uart.tx_data.write(|w| unsafe {
                w.tx_data().bits(byte.into())
            });
            let mut timeout = 0;
            while self.uart.tx_rdy.read().tx_rdy().bit() == false {
                unsafe { riscv::asm::delay(1) };
                if timeout > 1_000 {
                    break;
                }
                timeout += 1;
            }
        }
        Ok(())
    }
}
