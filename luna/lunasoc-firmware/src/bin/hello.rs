#![no_std]
#![no_main]

use panic_halt as _;
use riscv_rt::entry;

use firmware::hal;
use firmware::pac;
use lunasoc_firmware as firmware;

use core::fmt::Write;
use hal::Serial;

use hal::hal::delay::DelayUs;
use hal::Timer;

use firmware::SYSTEM_CLOCK_FREQUENCY;

#[entry]
fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();

    let leds = &peripherals.LEDS;
    let mut timer = Timer::new(peripherals.TIMER, SYSTEM_CLOCK_FREQUENCY);
    let mut serial = Serial::new(peripherals.UART);

    writeln!(serial, "Peripherals initialized, entering main loop.").unwrap();

    let mut counter = 0;
    let mut direction = true;
    let mut led_state = 0b11000000;

    loop {
        timer.delay_ms(100_u32).unwrap();

        if direction {
            led_state >>= 1;
            if led_state == 0b00000011 {
                direction = false;
                writeln!(serial, "left: {}", counter).unwrap();
            }
        } else {
            led_state <<= 1;
            if led_state == 0b11000000 {
                direction = true;
                writeln!(serial, "right: {}", counter).unwrap();
            }
        }

        leds.output.write(|w| unsafe { w.output().bits(led_state) });
        counter += 1;
    }
}
