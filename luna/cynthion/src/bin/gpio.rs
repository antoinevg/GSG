#![no_std]
#![no_main]

use cynthion::pac;

use cynthion::hal;
use hal::hal::delay::DelayUs;
use hal::Serial;
use hal::Timer;

use log::info;

use riscv_rt::entry;

#[entry]
fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();

    // initialize logging
    let serial = Serial::new(peripherals.UART);
    cynthion::log::init(serial);

    // configure gpioa pins 7-4: output, 3-0: input
    let gpioa = &peripherals.GPIOA;
    gpioa.mode.write(|w| unsafe { w.mode().bits(0b0000_1111) }); // 0=output, 1=input

    let leds = &peripherals.LEDS;
    let mut timer = Timer::new(peripherals.TIMER, cynthion::SYSTEM_CLOCK_FREQUENCY);

    info!("Peripherals initialized, entering main loop.");

    let mut counter = 0;
    let mut direction = true;
    let mut led_state = 0b110000;

    loop {
        timer.delay_ms(1_000).unwrap();

        if direction {
            led_state >>= 1;
            if led_state == 0b000011 {
                direction = false;
            }
        } else {
            led_state <<= 1;
            if led_state == 0b110000 {
                direction = true;
            }
        }
        leds.output.write(|w| unsafe { w.output().bits(led_state) });

        // gpioa - read input pins
        let bits: u32 = gpioa.idr.read().bits() & 0b0000_1111;
        info!("gpioa bits: {bits:#010b}");

        // gpioa - toggle output pins
        if counter % 2 == 0 {
            gpioa.odr.write(|w| unsafe { w.odr().bits(0b0001_0000) });
        } else {
            gpioa.odr.write(|w| unsafe { w.odr().bits(0b0000_0000) });
        }

        counter += 1;
    }
}
