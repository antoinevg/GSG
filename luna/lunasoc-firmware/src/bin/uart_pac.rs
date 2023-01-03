#![no_std]
#![no_main]

use lunasoc_pac as pac;
use panic_halt as _;
use riscv_rt::entry;

const SYSTEM_CLOCK_FREQUENCY: u32 = 10_000_000;

#[entry]
fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap() ;
    let leds = &peripherals.LEDS;

    let mut direction = true;
    let mut led_state = 0b11000000;

    loop {
        delay_ms(SYSTEM_CLOCK_FREQUENCY, 100);

        if direction {
            led_state >>= 1;
            if led_state == 0b00000011 {
                direction = false;
                uart_tx("left\n");
            }
        } else {
            led_state <<= 1;
            if led_state == 0b11000000 {
                direction = true;
                uart_tx("right\n");
            }
        }

        leds.output.write(|w| unsafe { w.output().bits(led_state) });
    }
}

fn delay_ms(sys_clk: u32, ms: u32) {
    let cycles: u32 = sys_clk / 1_000 * ms;

    let peripherals = unsafe { pac::Peripherals::steal() };
    let timer = &peripherals.TIMER;

    timer.en.write(|w| w.en().bit(true));
    timer.reload.write(|w| unsafe { w.reload().bits(cycles) });

    while timer.ctr.read().ctr().bits() > 0 {
        unsafe { riscv::asm::nop(); }
    }

    timer.en.write(|w| w.en().bit(false));
    timer.reload.write(|w| unsafe { w.reload().bits(0) });
}

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
