#![no_std]
#![no_main]

use core::fmt::Write;
use lunasoc_hal::prelude::*;
use lunasoc_pac::{Peripherals};
use riscv_rt::entry;
use panic_halt as _;

lunasoc_hal::uart! {
    Uart: lunasoc_pac::UART,
}

lunasoc_hal::timer! {
    Timer: lunasoc_pac::TIMER,
}

const SYSTEM_CLOCK_FREQUENCY: u32 = 50_000_000;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    let mut uart = Uart::new(peripherals.UART);
    let mut timer = Timer::new(peripherals.TIMER, SYSTEM_CLOCK_FREQUENCY);
    let mut uptime = 0;

    writeln!(uart, "Peripherals initialized, entering main loop.").unwrap();

    loop {
        timer.delay_ms(1000_u32);
        uptime += 1;
        writeln!(uart, "Uptime: {} seconds", uptime).unwrap();
    }
}
