#![no_std]
#![no_main]

use panic_halt as _;
use riscv_rt::entry;

// - main entry point ---------------------------------------------------------

#[entry]
fn main() -> ! {
    loop {}
}
