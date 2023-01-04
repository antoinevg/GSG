#![allow(dead_code, unused_mut, unused_variables)]

#![no_std]
#![no_main]

use lunasoc_pac as pac;
use panic_halt as _;
use riscv_rt::entry;

use lunasoc_firmware::csr;

const SYSTEM_CLOCK_FREQUENCY: u32 = 60_000_000;

#[entry]
fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap() ;
    let leds = &peripherals.LEDS;
    let timer = &peripherals.TIMER;
    let uart = &peripherals.UART;

    // configure and enable timer
    timer.reload.write(|w| unsafe { w.reload().bits(SYSTEM_CLOCK_FREQUENCY / 2) });
    timer.en.write(|w| w.en().bit(true));

    // enable timer events
    timer.ev_enable.write(|w| w.enable().bit(true));

    // enable interrupts
    unsafe {
        // set mstatus register: interrupt enable
        riscv::interrupt::enable();

        // set mie register: machine external interrupts enable
        riscv::register::mie::set_mext();

        // write csr: enable timer interrupt
        csr::interrupt::enable(pac::Interrupt::TIMER)
    }

    loop {
        unsafe { riscv::asm::nop(); }
    }
}


// - interrupt handler --------------------------------------------------------

#[allow(non_snake_case)]
#[no_mangle]
unsafe fn MachineExternal() {
    static mut TOGGLE: bool = true;

    let peripherals = unsafe { pac::Peripherals::steal() };
    let leds = &peripherals.LEDS;
    let timer = &peripherals.TIMER;

    if csr::interrupt::pending(pac::Interrupt::TIMER) {
        // clear interrupt
        let pending = timer.ev_pending.read().pending().bit();
        timer.ev_pending.write(|w| w.pending().bit(pending));

        // blinkenlights
        if TOGGLE {
            leds.output.write(|w| unsafe { w.output().bits(255) });
        } else {
            leds.output.write(|w| unsafe { w.output().bits(0) });
        }
        TOGGLE = !TOGGLE;

    } else {
        uart_tx("MachineExternal - unknown interrupt\n");
    }
}


// - exception handler --------------------------------------------------------

#[allow(non_snake_case)]
#[no_mangle]
unsafe fn ExceptionHandler(trap_frame: &riscv_rt::TrapFrame) -> ! {
    uart_tx("ExceptionHandler\n");
    loop {}
}


// - helpers ------------------------------------------------------------------

fn uart_tx(string: &str) {
    let peripherals = unsafe { pac::Peripherals::steal() };
    let uart = &peripherals.UART;

    for c in string.chars() {
        while uart.tx_rdy.read().tx_rdy().bit() == false {
            unsafe { riscv::asm::nop(); }
        }
        uart.tx_data.write(|w| unsafe { w.tx_data().bits(c as u8) })
    }
}
