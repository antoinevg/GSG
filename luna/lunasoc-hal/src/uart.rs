#[macro_export]
macro_rules! uart {
    ($(
        $UARTX:ident: $PACUARTX:ty,
    )+) => {
        $(
            #[derive(Debug)]
            pub struct $UARTX {
                registers: $PACUARTX,
            }

            // lifecycle
            impl $UARTX {
                /// Create a new `Uart` from the [`UART`](pac::UART) peripheral.
                pub fn new(registers: $PACUARTX) -> Self {
                    Self { registers }
                }

                /// Release the [`Uart`](pac::UART) peripheral and consume self.
                pub fn free(self) -> $PACUARTX {
                    self.registers
                }

                /// Obtain a static `Uart` instance for use in e.g. interrupt handlers
                ///
                /// # Safety
                ///
                /// 'Tis thine responsibility, that which thou doth summon.
                pub unsafe fn summon() -> Self {
                    Self {
                        registers: crate::pac::Peripherals::steal().UART,
                    }
                }
            }

            // trait: hal::serial::Write
            impl $crate::hal::serial::Write<u8> for $UARTX {
                type Error = core::convert::Infallible;

                fn write(&mut self, word: u8) -> $crate::nb::Result<(), Self::Error> {
                    if self.registers.tx_rdy.read().tx_rdy().bit() == false {
                        Err($crate::nb::Error::WouldBlock)
                    } else {
                        //unsafe {
                        //    self.registers.rxtx.write(|w| w.rxtx().bits(word.into()));
                        //}
                        self.registers.tx_data.write(|w| unsafe { w.tx_data().bits(word.into()) });
                        Ok(())
                    }
                }
                fn flush(&mut self) -> $crate::nb::Result<(), Self::Error> {
                    if self.registers.tx_rdy.read().tx_rdy().bit() == true {
                        Ok(())
                    } else {
                        Err($crate::nb::Error::WouldBlock)
                    }
                }
            }

            // trait: hal::serial::write::Default
            impl $crate::hal::blocking::serial::write::Default<u8> for $UARTX {}

            // trait: core::fmt::Write
            impl core::fmt::Write for $UARTX {
                fn write_str(&mut self, s: &str) -> core::fmt::Result {
                    use $crate::hal::prelude::*;
                    self.bwrite_all(s.as_bytes()).ok();
                    Ok(())
                }
            }

            // trait: From
            impl From<$PACUARTX> for $UARTX {
                fn from(registers: $PACUARTX) -> $UARTX {
                    $UARTX::new(registers)
                }
            }
        )+
    }
}

crate::uart! { Uart: crate::pac::UART, }
