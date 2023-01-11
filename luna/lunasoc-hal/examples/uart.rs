#![no_std]
#![no_main]

use panic_halt as _;
use riscv_rt::entry;

use lunasoc_hal as hal;
use lunasoc_pac as pac;

use hal::hal::delay::DelayUs;
use hal::Timer;

use core::fmt::Write;
use hal::Serial;

const SYSTEM_CLOCK_FREQUENCY: u32 = 50_000_000;

#[entry]
fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();

    let mut serial = Serial::new(peripherals.UART);
    let mut timer = Timer::new(peripherals.TIMER, SYSTEM_CLOCK_FREQUENCY);
    let mut uptime = 0;

    writeln!(serial, "Peripherals initialized, entering main loop.").unwrap();

    loop {
        timer.delay_ms(1000_u32).unwrap();
        uptime += 1;
        writeln!(serial, "Uptime: {} seconds", uptime).unwrap();
    }
}
