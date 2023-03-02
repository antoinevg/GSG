#![no_std]
#![no_main]

use riscv_rt::entry;

use cynthion::pac;

use cynthion::hal;
use hal::hal::delay::DelayUs;
use hal::Serial;
use hal::Timer;

use log::{debug, info};

#[entry]
fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();

    // initialize logging
    let serial = Serial::new(peripherals.UART);
    cynthion::log::init(serial);

    let leds = &peripherals.LEDS;
    let mut timer = Timer::new(peripherals.TIMER, cynthion::SYSTEM_CLOCK_FREQUENCY);

    info!("Peripherals initialized, entering main loop.");

    let mut counter = 0;
    let mut direction = true;
    let mut led_state = 0b11000000;

    loop {
        timer.delay_ms(100).unwrap();

        if direction {
            led_state >>= 1;
            if led_state == 0b00000011 {
                direction = false;
                info!("left: {}", counter);
            }
        } else {
            led_state <<= 1;
            if led_state == 0b11000000 {
                direction = true;
                debug!("right: {}", counter);
            }
        }

        leds.output.write(|w| unsafe { w.output().bits(led_state) });
        counter += 1;
    }
}
