#![allow(unused_imports, unused_mut, unused_variables)]
#![no_std]
#![no_main]

use core::fmt::Write;
use cynthion::{hal, pac};
use hal::hal::delay::DelayUs;
use hal::Serial;
use hal::Timer;

// - asm.S --------------------------------------------------------------------

//core::arch::global_asm!(include_str!("../../asm.S"));

// - panic_handler ------------------------------------------------------------

use core::panic::PanicInfo;

#[panic_handler]
#[no_mangle]
//#[inline(never)]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// - main ---------------------------------------------------------------------

#[no_mangle]
//#[inline(never)]
pub unsafe fn __pre_init() {
    pac::cpu::vexriscv::flush_icache();
    pac::cpu::vexriscv::flush_dcache();
}

#[no_mangle]
//#[inline(never)]
pub unsafe extern "C" fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();
    let leds = &peripherals.LEDS;
    let mut serial = Serial::new(peripherals.UART);
    //let mut timer = Timer::new(peripherals.TIMER, pac::clock::sysclk());

    log("Peripherals initialized, entering main loop.");
    //writeln!(serial, "Peripherals initialized, entering main loop.").unwrap();

    let mut direction = true;
    let mut led_state = 0b110000;
    let mut counter = 0;

    loop {
        pac::cpu::vexriscv::flush_dcache();

        //timer.delay_ms(1000).unwrap();
        unsafe { riscv::asm::delay(1_000_000) };

        if direction {
            led_state >>= 1;
            if led_state == 0b000011 {
                direction = false;
                log("left:");
                //writeln!(serial, "left: {}", counter).unwrap();
            }
        } else {
            led_state <<= 1;
            if led_state == 0b110000 {
                direction = true;
                log("right:");
                //writeln!(serial, "right: {}", counter).unwrap();
            }
        }

        //set_leds(leds, led_state);
        leds.output.write(|w| unsafe { w.output().bits(led_state) });
        counter += 1;
    }
}

//#[inline(never)]
pub fn set_leds(leds: &pac::LEDS, led_state: u8) {
    leds.output.write(|w| unsafe { w.output().bits(led_state) });
}

//#[inline(never)]
pub fn log(message: &str) {
    use core::fmt::Write;
    use hal::Serial;

    let peripherals = unsafe { pac::Peripherals::steal() };
    let mut serial = Serial::new(peripherals.UART);
    writeln!(serial, "{}", message).unwrap();
}
