#![allow(dead_code, non_snake_case, unused_imports, unused_mut, unused_variables)]
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
    let leds = IO_LEDS as *mut _;
    unsafe { core::ptr::write_volatile(leds, 0b11_0000) };
    loop {
    }
}

use cynthion::rt_minimal::TrapFrame;

#[no_mangle]
pub fn ExceptionHandler(trap_frame: &TrapFrame) -> ! {
    let leds = IO_LEDS as *mut _;
    unsafe { core::ptr::write_volatile(leds, 0b11_1100) };
    uart_tx("trap\n");

    let mut s = heapless::String::<32>::new();
    let _ = write!(s, "ra: 0x{:08x}", trap_frame.ra).unwrap();
    uart_tx(s.as_str());
    uart_tx("trap\n");

    loop { }
}

#[no_mangle]
pub fn DefaultHandler() {
    let leds = IO_LEDS as *mut _;
    unsafe { core::ptr::write_volatile(leds, 0b11_1111) };
}

// - main ---------------------------------------------------------------------

const IO_BASE: usize = 0x8000_0000;
const IO_LEDS: usize = IO_BASE + 0x0080;
const IO_UART_TX_DATA: usize = IO_BASE + 0x0010;
const IO_UART_TX_RDY: usize = IO_BASE + 0x0014;

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn main() -> ! {
    //pac::cpu::vexriscv::flush_icache();
    //pac::cpu::vexriscv::flush_dcache();

    let peripherals = pac::Peripherals::steal();
    let leds = IO_LEDS as *mut _;
    let mut serial = Writer { uart: peripherals.UART };

    writeln!(serial, "0x{:08x}", IO_LEDS).unwrap();
    //writeln!(serial, "oh hai, here we go already!\n").unwrap();
    //uart_tx("oh hai, here we go already!\n");

    let mut counter: usize = 0;
    loop {
        core::ptr::write_volatile(leds, counter & 0b0011_1111);
        unsafe { riscv::asm::delay(1_000_000) };
        counter += 1;
    }
}



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

struct Writer {
    uart: pac::UART,
}

impl core::fmt::Write for Writer {
    #[inline(never)]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        uart_tx(s);
        Ok(())
    }
}
