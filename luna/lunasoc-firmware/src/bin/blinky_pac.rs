#![no_std]
#![no_main]

use lunasoc_pac as pac;
use panic_halt as _;
use riscv_rt::entry;

const SYSTEM_CLOCK_FREQUENCY: u32 = 10_000_000;

#[entry]
fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();
    let leds = &peripherals.LEDS;

    let mut direction = true;
    let mut led_state = 0b11000000;

    loop {
        delay_ms(SYSTEM_CLOCK_FREQUENCY, 100);

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

fn delay_ms(sys_clk: u32, ms: u32) {
    let value: u32 = sys_clk / 1_000 * ms;

    let peripherals = unsafe { pac::Peripherals::steal() };
    let timer = &peripherals.TIMER;

    unsafe {
        timer.en.write(|w| w.en().bit(true));
        timer.reload.write(|w| w.reload().bits(value));
        while timer.ctr.read().ctr().bits() > 0 {
            riscv::asm::nop();
        }
        timer.en.write(|w| w.en().bit(false));
        timer.reload.write(|w| w.reload().bits(0));
    }
}
