#![allow(unused_imports)] // TODO

#![no_std]
#![no_main]

use lunasoc_firmware as firmware;

use firmware::hal;
use firmware::pac;

use log::{debug, error, info};

use panic_halt as _;
use riscv_rt::entry;

// - global static state ------------------------------------------------------

use firmware::Message;
use heapless::mpmc::MpMcQueue as Queue;
static MESSAGE_QUEUE: Queue<Message, 128> = Queue::new();

// - main entry point ---------------------------------------------------------

#[entry]
fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();
    let _leds = &peripherals.LEDS;

    // initialize logging
    let serial = hal::Serial::new(peripherals.UART);
    firmware::log::init(serial);

    // initialize timer
    let sysclk = firmware::SYSTEM_CLOCK_FREQUENCY;
    let mut timer = hal::Timer::new(peripherals.TIMER, sysclk);
    timer.set_timeout_ticks(sysclk * 1);
    timer.enable();
    timer.listen(hal::timer::Event::TimeOut);

    // initialize usb
    let mut usb0 = hal::UsbInterface0::new(
        peripherals.USB0,
        peripherals.USB0_EP_CONTROL,
        peripherals.USB0_EP_IN,
        peripherals.USB0_EP_OUT,
    );
    info!("Connecting USB device...");
    let speed = usb0.connect();
    //usb0.listen();
    info!("Connected: {}", speed);

    // enable interrupts
    unsafe {
        // set mstatus register: interrupt enable
        riscv::interrupt::enable();
        // set mie register: machine external interrupts enable
        riscv::register::mie::set_mext();
        // write csr: enable interrupts
        pac::csr::interrupt::enable(pac::Interrupt::TIMER);
        pac::csr::interrupt::enable(pac::Interrupt::USB0);
    }

    info!("Peripherals initialized, entering main loop.");

    loop {
        if let Some(message) = MESSAGE_QUEUE.dequeue() {
            match message {
                Message::Timer(_tick) => (),
                Message::UsbReset => {
                    usb0.reset();
                },
                Message::UnknownInterrupt(pending) => {
                    error!("Unknown interrupt pending: {:#035b}", pending);
                },
            }
        }


    }
}

// - MachineExternal interrupt handler ----------------------------------------

#[allow(non_snake_case)]
#[no_mangle]
fn MachineExternal() {
    static mut COUNTER_IRQ: u32 = 0;

    let peripherals = unsafe { pac::Peripherals::steal() };
    let timer = &peripherals.TIMER;
    let usb0 = &peripherals.USB0;

    let message = if usb0.ev_pending.read().pending().bit() {
        usb0.ev_pending.modify(|r, w| w.pending().bit(r.pending().bit()));
        Message::UsbReset

    } else if timer.ev_pending.read().pending().bit() {
        timer.ev_pending.modify(|r, w| w.pending().bit(r.pending().bit()));
        Message::Timer(unsafe { COUNTER_IRQ })

    } else {
        let pending = unsafe { pac::csr::interrupt::reg_pending() };
        Message::UnknownInterrupt(pending)
    };

    // TODO requeue if needed
    match MESSAGE_QUEUE.enqueue(message) {
        Ok(_) => (),
        Err(e) => {
            error!("overflow: {:?}", e);
        }
    }

    unsafe {
        COUNTER_IRQ += 1;
    }
}
