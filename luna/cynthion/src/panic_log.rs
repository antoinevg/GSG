use core::panic::PanicInfo;
use core::sync::atomic::{self, Ordering};

use log::error;

// - panic handler ------------------------------------------------------------

#[no_mangle]
#[inline(never)]
#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    // panic stations
    let peripherals = unsafe { crate::pac::Peripherals::steal() };
    let leds = &peripherals.LEDS;
    leds.output
        .write(|w| unsafe { w.output().bits(0b101010) });

    if let Some(message) = panic_info.message() {
        error!("Panic: {}", message);
    } else {
        error!("Panic: Unknown");
    }

    if let Some(location) = panic_info.location() {
        error!("'{}' : {}", location.file(), location.line(),);
    }

    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}
