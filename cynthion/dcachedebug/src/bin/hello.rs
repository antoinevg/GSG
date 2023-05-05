#![allow(unused_imports, unused_mut, unused_variables)]

#![no_std]
#![no_main]

use cynthion::pac;
use cynthion::rt;

use cynthion::hal;
use hal::hal::delay::DelayUs;
use hal::Serial;
use hal::Timer;

use log::info;

use core::fmt::Write;

// - asm.S --------------------------------------------------------------------

//core::arch::global_asm!(include_str!("../../asm.S"));


// - panic_handler ------------------------------------------------------------

use core::panic::PanicInfo;

#[panic_handler]
#[no_mangle]
fn panic(_info: &PanicInfo) -> ! {
    loop {
    }
}

// - main ---------------------------------------------------------------------

#[no_mangle]
pub unsafe fn __pre_init() {
    pac::cpu::vexriscv::flush_icache();
    pac::cpu::vexriscv::flush_dcache();
}

#[no_mangle]
pub unsafe extern "C" fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();
    let leds = &peripherals.LEDS;

    // initialize logging
    let mut serial = Serial::new(peripherals.UART);
    cynthion::log::init(serial);

    let mut timer = Timer::new(peripherals.TIMER, pac::clock::sysclk());
    let mut counter = 0;
    let mut direction = true;
    let mut led_state = 0b110000;

    info!("Peripherals initialized, entering main loop.");

    loop {
        //pac::cpu::vexriscv::flush_dcache();
        //unsafe { riscv::asm::nop() };

        timer.delay_ms(1000).unwrap();

        if direction {
            led_state >>= 1;
            if led_state == 0b000011 {
                direction = false;
                info!("left: {}", counter);
            }
        } else {
            led_state <<= 1;
            if led_state == 0b110000 {
                direction = true;
                info!("right: {}", counter);
            }
        }

        leds.output.write(|w| unsafe { w.output().bits(led_state) });
        counter += 1;
    }
}
