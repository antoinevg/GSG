#![allow(dead_code, unused_imports)]
#![no_std]
#![no_main]

///! So the problem is that RV32I does not support atomic
///! instructions:
///!
///!  - heapless relies on atomic-polyfill which relies on
///!    critical-section on platforms without atomics
///!  - log relies on critical-section
///!  - Mutex implementation also comes from critical-section
///!
///! So given the lack of support for atomics critical-section on rv32i
///! basically just shouts yolo and disables interrupts globally.
///!
///! Which unfortunately has some performance implications ...
use moondancer::pac;
use pac::csr::interrupt;

use moondancer::hal;

use log::{debug, error, info, trace, warn};

use riscv_rt::entry;

// - global static state ------------------------------------------------------

use moondancer::Message;
use heapless::mpmc::MpMcQueue as Queue;
static MESSAGE_QUEUE: Queue<Message, 256> = Queue::new();

// - MachineExternal interrupt handler ----------------------------------------

#[allow(non_snake_case)]
#[no_mangle]
fn MachineExternal() {
    if pac::csr::interrupt::pending(pac::Interrupt::TIMER) {
        let timer = unsafe { hal::Timer::summon() };
        timer.clear_pending();

        // enqueue a message
        MESSAGE_QUEUE
            .enqueue(Message::TimerEvent(0))
            .expect("MachineExternal - message queue overflow");
    } else {
        error!("MachineExternal - unknown interrupt");
    }
}

// - program state ------------------------------------------------------------

#[derive(Debug)]
struct State {
    counter: usize,
    interrupts: u32,
    secondary: u32,
    while_max: u32,
    while_last: u32,
}

impl State {
    const fn new() -> Self {
        Self {
            counter: 1,
            interrupts: 0,
            secondary: 0,
            while_max: 0,
            while_last: 0,
        }
    }
}

// - main entry point ---------------------------------------------------------

#[entry]
fn entry() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();
    let leds = &peripherals.LEDS;
    leds.output.write(|w| unsafe { w.output().bits(0x00) });

    // initialize logging
    let serial = hal::Serial::new(peripherals.UART);
    moondancer::log::init(serial);
    info!("logging initialized");
    unsafe { riscv::asm::delay(pac::clock::sysclk()) };

    // configure and enable timer
    let one_second = pac::clock::sysclk();
    let mut timer = hal::Timer::new(peripherals.TIMER, one_second);
    timer.set_timeout_ticks(one_second / 38_000); // > 35_000 starts saturating very quickly
    timer.enable();
    timer.listen(hal::timer::Event::TimeOut);

    // enable interrupts
    unsafe {
        riscv::interrupt::enable();
        riscv::register::mie::set_mext();
        pac::csr::interrupt::enable(pac::Interrupt::TIMER)
    }

    // main loop
    let mut state = State::new();
    loop {
        state = match main_loop(state, leds) {
            Ok(state) => state,
            Err(e) => {
                leds.output.write(|w| unsafe { w.output().bits(0xff) });
                panic!("Fatal: {:?}", e);
            }
        };
    }
}

// - main loop ----------------------------------------------------------------

#[inline(always)]
fn main_loop(mut state: State, leds: &pac::LEDS) -> moondancer::GreatResult<State> {
    leds.output.write(|w| unsafe { w.output().bits(1 << 0) });

    let mut while_counter = 0;
    while let Some(message) = MESSAGE_QUEUE.dequeue() {
        match message {
            Message::TimerEvent(0) => {
                state.interrupts += 1;
                leds.output.write(|w| unsafe { w.output().bits(1 << 1) });
            }
            Message::TimerEvent(_value) => {
                state.secondary += 1;
            }
            _ => {}
        }
        while_counter += 1;
        if while_counter > state.while_max {
            state.while_max = while_counter;
        }
    }
    state.while_last = while_counter;

    leds.output.write(|w| unsafe { w.output().bits(1 << 2) });

    if state.counter % 50_000 == 0 {
        leds.output.write(|w| unsafe { w.output().bits(1 << 3) });
        info!("main_loop() => {:?}", state);
        MESSAGE_QUEUE
            .enqueue(Message::TimerEvent(state.counter))
            .expect("main_loop() message queue overflow");
    }

    state.counter += 1;

    Ok(state)
}
