#![no_std]
#![no_main]

use panic_halt as _;
use riscv_rt::entry;

use firmware::{hal, pac};
use lunasoc_firmware as firmware;

use log::{debug, error, info};

#[entry]
fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();

    // initialize logging
    let serial = hal::Serial::new(peripherals.UART);
    firmware::log::init(serial);

    // configure and enable timer
    let one_second = firmware::SYSTEM_CLOCK_FREQUENCY;
    let mut timer = hal::Timer::new(peripherals.TIMER, one_second);
    timer.set_timeout_ticks(one_second / 2);
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

    info!("Peripherals initialized, entering main loop.");

    let mut uptime = 1;
    loop {
        unsafe {
            riscv::asm::delay(firmware::SYSTEM_CLOCK_FREQUENCY);
            uptime += 1;
        }
        info!("Uptime: {} seconds", uptime);
    }
}

// interrupt handler
#[allow(non_snake_case)]
#[no_mangle]
fn MachineExternal() {
    static mut TOGGLE: bool = true;

    if unsafe { pac::csr::interrupt::pending(pac::Interrupt::TIMER) } {
        let mut timer = unsafe { hal::Timer::summon() };
        timer.clear_irq();

        debug!("MachineExternal - timer interrupt");

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
        error!("MachineExternal - unknown interrupt");
    }
}
