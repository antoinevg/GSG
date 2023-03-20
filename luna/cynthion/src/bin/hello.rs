#![no_std]
#![no_main]

use cynthion::pac;

use cynthion::hal;
use hal::hal::delay::DelayUs;
use hal::Serial;
use hal::Timer;

use log::{debug, info};

use riscv_rt::entry;

#[entry]
fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();
    let leds = &peripherals.LEDS;

    // initialize logging
    let serial = Serial::new(peripherals.UART);
    cynthion::log::init(serial);

    let mut timer = Timer::new(peripherals.TIMER, cynthion::SYSTEM_CLOCK_FREQUENCY);
    let mut counter = 0;
    let mut direction = true;
    let mut led_state = 0b110000;

    info!("Peripherals initialized, entering main loop.");

    loop {
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
                debug!("right: {}", counter);
            }
        }

        leds.output.write(|w| unsafe { w.output().bits(led_state) });
        counter += 1;
    }
}
