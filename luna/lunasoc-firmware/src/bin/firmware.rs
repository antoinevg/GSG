#![allow(unused_imports)] // TODO
#![no_std]
#![no_main]

use lunasoc_firmware as firmware;

use firmware::hal;
use firmware::pac;

use hal::smolusb::Device;

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
                }
                Message::UnknownInterrupt(pending) => {
                    error!("Unknown interrupt pending: {:#035b}", pending);
                }
            }
        }
    }
}

// - MachineExternal interrupt handler ----------------------------------------

#[allow(non_snake_case)]
#[no_mangle]
fn MachineExternal() {
    static mut COUNTER_IRQ: u32 = 0;

    let mut timer = unsafe { hal::Timer::summon() };
    let mut usb0 = unsafe { hal::UsbInterface0::summon() };

    let message = if usb0.is_pending(pac::Interrupt::USB0) {
        usb0.clear_pending(pac::Interrupt::USB0);
        Message::UsbReset
    } else if usb0.is_pending(pac::Interrupt::USB0_EP_CONTROL) {
        usb0.clear_pending(pac::Interrupt::USB0_EP_CONTROL);
        panic!("MachineExternal - usb0.ep_control interrupt");
    } else if usb0.is_pending(pac::Interrupt::USB0_EP_IN) {
        usb0.clear_pending(pac::Interrupt::USB0_EP_IN);
        panic!("MachineExternal - usb0.ep_in interrupt");
    } else if usb0.is_pending(pac::Interrupt::USB0_EP_OUT) {
        usb0.clear_pending(pac::Interrupt::USB0_EP_OUT);
        panic!("MachineExternal - usb0.ep_out interrupt");
    } else if timer.is_pending() {
        timer.clear_pending();
        Message::Timer(unsafe { COUNTER_IRQ })
    } else {
        let pending = unsafe { pac::csr::interrupt::reg_pending() };
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
