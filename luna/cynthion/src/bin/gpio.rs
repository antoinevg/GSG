#![no_std]
#![no_main]

use cynthion::pac;

use cynthion::hal;
use hal::hal::delay::DelayUs;
use hal::Serial;
use hal::Timer;

use log::{error, info};

use riscv_rt::entry;

// - interrupt handler --------------------------------------------------------

#[allow(non_snake_case)]
#[no_mangle]
fn MachineExternal() {
    let peripherals = unsafe { pac::Peripherals::steal() };
    let gpioa = &peripherals.GPIOA;

    if pac::csr::interrupt::pending(pac::Interrupt::GPIOA) {
        let pending = gpioa.ev_pending.read().pending().bit();
        gpioa.ev_pending.write(|w| w.pending().bit(pending));

        let bits_all: u32 = gpioa.idr.read().bits();
        let bits_in: u32 = gpioa.idr.read().bits() & 0b0000_1111;
        info!("gpioa bits: {bits_all:#010b} {bits_in:#010b}");
    } else {
        error!("MachineExternal - unknown interrupt");
    }
}

// - main entry point ---------------------------------------------------------

#[entry]
fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();
    let leds = &peripherals.LEDS;

    // initialize logging
    let serial = Serial::new(peripherals.UART);
    cynthion::log::init(serial);

    // configure gpioa pins 7-4:output, 3-0:input
    let gpioa = &peripherals.GPIOA;
    gpioa
        .moder
        .write(|w| unsafe { w.moder().bits(0b0000_1111) }); // 0=output, 1=input

    // enable gpioa events
    gpioa.ev_enable.write(|w| w.enable().bit(true));

    // configure and enable timer
    let mut timer = Timer::new(peripherals.TIMER, pac::clock::sysclk());
    timer.set_timeout_ticks(pac::clock::sysclk() / 2);
    timer.enable();

    // enable interrupts
    unsafe {
        // set mstatus register: interrupt enable
        riscv::interrupt::enable();

        // set mie register: machine external interrupts enable
        riscv::register::mie::set_mext();

        // write csr: enable timer interrupt
        pac::csr::interrupt::enable(pac::Interrupt::GPIOA)
    }

    info!("Peripherals initialized, entering main loop.");

    let mut counter = 0;

    loop {
        gpioa
            .odr
            .write(|w| unsafe { w.odr().bits(counter & 0b1111_0000) });
        leds.output.write(|w| unsafe { w.output().bits(counter) });

        timer.delay_ms(1_000).unwrap();
        counter += 1;
    }
}
