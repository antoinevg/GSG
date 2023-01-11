#![allow(dead_code, unused_mut, unused_variables)]
#![no_std]
#![no_main]

use panic_halt as _;
use riscv_rt::entry;

use lunasoc_hal as hal;
use lunasoc_pac as pac;

use hal::Serial;
use hal::Timer;

use core::fmt::Write;

const SYSTEM_CLOCK_FREQUENCY: u32 = 60_000_000;

#[entry]
fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();

    let leds = &peripherals.LEDS;
    let mut timer = Timer::new(peripherals.TIMER, SYSTEM_CLOCK_FREQUENCY);
    let mut uart = Serial::new(peripherals.UART);

    // configure and enable timer
    timer.set_timeout_ticks(SYSTEM_CLOCK_FREQUENCY / 2);
    timer.enable();

    // enable timer events
    timer.listen(hal::timer::Event::TimeOut);

    // enable interrupts
    unsafe {
        // set mstatus register: interrupt enable
        riscv::interrupt::enable();

        // set mie register: machine external interrupts enable
        riscv::register::mie::set_mext();

        // write csr: enable timer interrupt
        pac::csr::interrupt::enable(pac::Interrupt::TIMER)
    }

    loop {
        unsafe {
            riscv::asm::delay(SYSTEM_CLOCK_FREQUENCY);
        }
        writeln!(uart, "Ping").unwrap();
    }
}

// interrupt handler
#[allow(non_snake_case)]
#[no_mangle]
fn MachineExternal() {
    static mut TOGGLE: bool = true;

    if unsafe { pac::csr::interrupt::pending(pac::Interrupt::TIMER) } {
        let mut timer = unsafe { Timer::summon() };
        timer.clear_irq();

        // blinkenlights
        let peripherals = unsafe { pac::Peripherals::steal() };
        let leds = &peripherals.LEDS;

        if unsafe { TOGGLE } {
            leds.output.write(|w| unsafe { w.output().bits(255) });
        } else {
            leds.output.write(|w| unsafe { w.output().bits(0) });
        }
        unsafe { TOGGLE = !TOGGLE };
    } else {
        let mut uart = unsafe { Serial::summon() };
        writeln!(uart, "MachineExternal - unknown interrupt").unwrap();
    }
}
