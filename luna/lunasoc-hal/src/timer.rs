#[macro_export]
macro_rules! timer {
    ($(
        $TIMERX:ident: $PACTIMERX:ty,
    )+) => {
        $(
            #[derive(Debug)]
            pub struct $TIMERX {
                registers: $PACTIMERX,
                pub sys_clk: u32,
            }

            impl $TIMERX {
                pub fn new(registers: $PACTIMERX, sys_clk: u32) -> Self {
                    Self { registers, sys_clk }
                }

                pub fn free(self) -> $PACTIMERX {
                    self.registers
                }
            }

            impl<UXX: core::convert::Into<u32>> $crate::hal::blocking::delay::DelayMs<UXX> for $TIMERX {
                fn delay_ms(&mut self, ms: UXX) -> () {
                    let value: u32 = self.sys_clk / 1_000 * ms.into();
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
