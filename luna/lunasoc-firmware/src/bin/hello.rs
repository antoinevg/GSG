#![no_std]
#![no_main]

use panic_halt as _;
use riscv_rt::entry;

use core::fmt::Write;

use lunasoc_firmware as firmware;

use firmware::pac;
use firmware::hal;
use hal::prelude::*;

use firmware::{Timer, Uart};
use firmware::SYSTEM_CLOCK_FREQUENCY;


#[entry]
fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();

    let leds = &peripherals.LEDS;
    let mut timer = Timer::new(peripherals.TIMER, SYSTEM_CLOCK_FREQUENCY);
    let mut uart = Uart::new(peripherals.UART);

    writeln!(uart, "Peripherals initialized, entering main loop.").unwrap();

    let mut counter = 0;
    let mut direction = true;
    let mut led_state = 0b11000000;

    loop {
        timer.delay_ms(100_u32);

        if direction {
            led_state >>= 1;
            if led_state == 0b00000011 {
                direction = false;
                writeln!(uart, "left: {}", counter).unwrap();
            }
        } else {
            led_state <<= 1;
            if led_state == 0b11000000 {
                direction = true;
                writeln!(uart, "right: {}", counter).unwrap();
            }
        }

        leds.output.write(|w| unsafe { w.output().bits(led_state) });
        counter += 1;
    }
}
