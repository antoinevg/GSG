/// Timer Events
///
/// Each event is a possible interrupt source, if enabled
pub enum Event {
    /// Timer timed out / count down ended
    TimeOut,
}

#[macro_export]
macro_rules! timer {
    ($(
        $TIMERX:ident: $PACTIMERX:ty,
    )+) => {
        $(
            #[derive(Debug)]
            pub struct $TIMERX {
                registers: $PACTIMERX,
                pub clk: u32,
            }

            // lifecycle
            impl $TIMERX {
                /// Create a new `Timer` from the [`TIMER`](pac::TIMER) peripheral.
                pub fn new(registers: $PACTIMERX, clk: u32) -> Self {
                    Self { registers, clk }
                }

                /// Release the [`TIMER`](pac::TIMER) peripheral and consume self.
                pub fn free(self) -> $PACTIMERX {
                    self.registers
                }

                /// Obtain a static `Timer` instance for use in e.g. interrupt handlers
                ///
                /// # Safety
                ///
                /// 'Tis thine responsibility, that which thou doth summon.
                pub unsafe fn summon() -> Self {
                    Self {
                        registers: crate::pac::Peripherals::steal().TIMER,
                        clk: 0, // TODO
                    }
                }
            }

            // configuration
            impl $TIMERX {
                pub fn counter(&self) -> u32 {
                    self.registers.ctr.read().ctr().bits()
                }

                pub fn disable(&self) {
                    self.registers.en.write(|w| w.en().bit(false));
                }

                pub fn enable(&self) {
                    self.registers.en.write(|w| w.en().bit(true));
                }

                pub fn set_timeout<T>(&mut self, timeout: T)
                where
                    T: Into<core::time::Duration>
                {
                    const NANOS_PER_SECOND: u64 = 1_000_000_000;
                    let timeout = timeout.into();

                    let clk = self.clk as u64;
                    let ticks = u32::try_from(
                        clk * timeout.as_secs() +
                        clk * u64::from(timeout.subsec_nanos()) / NANOS_PER_SECOND,
                    ).unwrap_or(u32::max_value());

                    self.set_timeout_ticks(ticks.max(1));
                }

                // TODO private
                pub fn set_timeout_ticks(&mut self, ticks: u32) {
                    self.registers.reload.write(|w| unsafe {
                        w.reload().bits(ticks)
                    });
                }
            }

            // interrupts
            impl $TIMERX {
                /// Start listening for `event`
                pub fn listen(&mut self, event: Event) {
                    match event {
                        Event::TimeOut => {
                            self.registers.ev_enable.write(|w| w.enable().bit(true));
                        }
                    }
                }

                /// Stop listening for `event`
                pub fn unlisten(&mut self, event: Event) {
                    match event {
                        Event::TimeOut => {
                            self.registers.ev_enable.write(|w| w.enable().bit(false));
                        }
                    }
                }

                /// Check if the interrupt flag is pending
                pub fn is_irq_pending(&mut self) -> bool {
                    self.registers.ev_pending.read().pending().bit_is_set()
                }

                /// Check if the interrupt flag is cleared
                pub fn is_irq_clear(&mut self) -> bool {
                    self.registers.ev_pending.read().pending().bit_is_clear()
                }

                /// Clear the interrupt flag
                pub fn clear_irq(&mut self) {
                    let pending = self.registers.ev_pending.read().pending().bit();
                    self.registers.ev_pending.write(|w| w.pending().bit(pending));
                }
            }

            // original implementation
            impl<UXX: core::convert::Into<u32>> $crate::hal::blocking::delay::DelayMs<UXX> for $TIMERX {
                fn delay_ms(&mut self, ms: UXX) -> () {
                    let value: u32 = self.clk / 1_000 * ms.into();
                    unsafe {
                        // start timer
                        self.registers.en.write(|w| w.en().bit(true));
                        self.registers.reload.write(|w| w.reload().bits(value));
                        while self.registers.ctr.read().ctr().bits() > 0 {
                            riscv::asm::nop();
                        }

                        // reset timer
                        self.registers.en.write(|w| w.en().bit(false));
                        self.registers.reload.write(|w| w.reload().bits(0));
                    }
                }
            }
        )+
    }
}

crate::timer! { Timer: crate::pac::TIMER, }
