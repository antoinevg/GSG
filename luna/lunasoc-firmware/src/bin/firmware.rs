#![allow(unused_imports)] // TODO
#![no_std]
#![no_main]

use lunasoc_firmware as firmware;

use firmware::hal;
use firmware::pac;

use hal::smolusb::Device;
use pac::csr::interrupt;

use log::{debug, error, info, warn};

use panic_halt as _;
use riscv_rt::entry;

// - global static state ------------------------------------------------------

use firmware::Message;
use heapless::mpmc::MpMcQueue as Queue;
static MESSAGE_QUEUE: Queue<Message, 128> = Queue::new();

// - MachineExternal interrupt handler ----------------------------------------

#[allow(non_snake_case)]
#[no_mangle]
fn MachineExternal() {
    static mut COUNTER_IRQ: u32 = 0;

    let mut timer = unsafe { hal::Timer::summon() };
    let mut usb0 = unsafe { hal::UsbInterface0::summon() };

    let message = if usb0.is_pending(pac::Interrupt::USB0) {
        usb0.clear_pending(pac::Interrupt::USB0);
        Message::Interrupt(pac::Interrupt::USB0)
    } else if usb0.is_pending(pac::Interrupt::USB0_EP_CONTROL) {
        usb0.clear_pending(pac::Interrupt::USB0_EP_CONTROL);
        Message::Interrupt(pac::Interrupt::USB0_EP_CONTROL)

    } else if usb0.is_pending(pac::Interrupt::USB0_EP_IN) {
        usb0.clear_pending(pac::Interrupt::USB0_EP_IN);
        Message::Interrupt(pac::Interrupt::USB0_EP_IN)

    } else if usb0.is_pending(pac::Interrupt::USB0_EP_OUT) {
        usb0.clear_pending(pac::Interrupt::USB0_EP_OUT);
        Message::Interrupt(pac::Interrupt::USB0_EP_OUT)

    } else if timer.is_pending() {
        timer.clear_pending();
        Message::TimerEvent(unsafe { COUNTER_IRQ })

    } else {
        let pending = interrupt::reg_pending();
        Message::UnknownInterrupt(pending)
    };

    match MESSAGE_QUEUE.enqueue(message) {
        Ok(_) => (),
        Err(e) => {
            panic!("MachineExternal - message queue overflow: {:?}", e);
        }
    }

    unsafe {
        COUNTER_IRQ += 1;
    }
}

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

    // initialize usb
    let mut usb0 = Device::new(hal::UsbInterface0::new(
        peripherals.USB0,
        peripherals.USB0_EP_CONTROL,
        peripherals.USB0_EP_IN,
        peripherals.USB0_EP_OUT,
    ));
    info!("Connecting USB device...");
    let speed = usb0.connect();
    info!("Connected: {:?}", speed);

    // enable interrupt events
    timer.listen(hal::timer::Event::TimeOut);
    usb0.enable_interrupts();

    unsafe {
        // set mstatus register: interrupt enable
        riscv::interrupt::enable();
        // set mie register: machine external interrupts enable
        riscv::register::mie::set_mext();
        // write csr: enable interrupts
        interrupt::enable(pac::Interrupt::TIMER);
        interrupt::enable(pac::Interrupt::USB0);
    }

    info!("Peripherals initialized, entering main loop.");

    loop {
        if let Some(message) = MESSAGE_QUEUE.dequeue() {
            match message {
                Message::TimerEvent(_tick) => (),
                Message::Interrupt(pac::Interrupt::USB0) => {
                    usb0.reset();
                }
                Message::Interrupt(pac::Interrupt::USB0_EP_CONTROL) => {
                }
                Message::Interrupt(pac::Interrupt::USB0_EP_IN) => {
                }
                Message::Interrupt(pac::Interrupt::USB0_EP_OUT) => {
                }
                Message::Interrupt(interrupt) => {
                    warn!("Unhandled interrupt: {:?}", interrupt);
                }
                Message::UnknownInterrupt(pending) => {
                    error!("Unknown interrupt pending: {:#035b}", pending);
                }
            }
        }
    }
}
