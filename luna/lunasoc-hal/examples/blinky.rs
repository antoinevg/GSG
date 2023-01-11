#![no_std]
#![no_main]

use panic_halt as _;
use riscv_rt::entry;

use lunasoc_hal as hal;
use lunasoc_pac as pac;

use hal::hal::delay::DelayUs;
use hal::Timer;

const SYSTEM_CLOCK_FREQUENCY: u32 = 10_000_000;

#[entry]
fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();

    let mut timer = Timer::new(peripherals.TIMER, SYSTEM_CLOCK_FREQUENCY);
    let leds = &peripherals.LEDS;

    let mut direction = true;
    let mut led_state = 0b11000000;

    loop {
        timer.delay_ms(100_u32).unwrap();

        if direction {
            led_state >>= 1;
            if led_state == 0b00000011 {
                direction = false;
            }
        } else {
            led_state <<= 1;
            if led_state == 0b11000000 {
                direction = true;
            }
        }
        unsafe {
            leds.output.write(|w| w.output().bits(led_state));
        }
    }
}
