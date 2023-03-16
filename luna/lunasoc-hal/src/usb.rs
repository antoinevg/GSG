//! Simple USB implementation

mod error;
pub use error::ErrorKind;

use crate::smolusb::control::*;
use crate::smolusb::traits::{
    ControlRead, EndpointRead, EndpointWrite, EndpointWriteRef, UsbDriver, UsbDriverOperations,
};

use crate::pac;
use pac::interrupt::Interrupt;

use log::{trace, warn};

/// Macro to generate hal wrappers for pac::USBx peripherals
///
/// For example:
///
///     impl_usb! {
///         Usb0: USB0, USB0_EP_CONTROL, USB0_EP_IN, USB0_EP_OUT,
///         Usb1: USB1, USB1_EP_CONTROL, USB1_EP_IN, USB1_EP_OUT,
///     }
///
macro_rules! impl_usb {
    ($(
        $USBX:ident: $USBX_CONTROLLER:ident, $USBX_EP_CONTROL:ident, $USBX_EP_IN:ident, $USBX_EP_OUT:ident,
    )+) => {
        $(
            pub struct $USBX {
                pub controller: pac::$USBX_CONTROLLER,
                pub ep_control: pac::$USBX_EP_CONTROL,
                pub ep_in: pac::$USBX_EP_IN,
                pub ep_out: pac::$USBX_EP_OUT,
            }

            impl $USBX {
                /// Create a new `Usb` from the [`USB`](pac::USB) peripheral.
                pub fn new(
                    controller: pac::$USBX_CONTROLLER,
                    ep_control: pac::$USBX_EP_CONTROL,
                    ep_in: pac::$USBX_EP_IN,
                    ep_out: pac::$USBX_EP_OUT,
                ) -> Self {
                    Self {
                        controller,
                        ep_control,
                        ep_in,
                        ep_out,
                    }
                }

                /// Release the [`USB`](pac::USB) peripheral and consume self.
                pub fn free(
                    self,
                ) -> (
                    pac::$USBX_CONTROLLER,
                    pac::$USBX_EP_CONTROL,
                    pac::$USBX_EP_IN,
                    pac::$USBX_EP_OUT,
                ) {
                    (self.controller, self.ep_control, self.ep_in, self.ep_out)
                }

                /// Obtain a static `Usb` instance for use in e.g. interrupt handlers
                ///
                /// # Safety
                ///
                /// 'Tis thine responsibility, that which thou doth summon.
                pub unsafe fn summon() -> Self {
                    Self {
                        controller: pac::Peripherals::steal().$USBX_CONTROLLER,
                        ep_control: pac::Peripherals::steal().$USBX_EP_CONTROL,
                        ep_in: pac::Peripherals::steal().$USBX_EP_IN,
                        ep_out: pac::Peripherals::steal().$USBX_EP_OUT,
                    }
                }
            }

            impl $USBX {
                // TODO &mut self
                // TODO pass event to listen for
                pub fn enable_interrupts(&self) {
                    // clear all event handlers
                    self.clear_pending(Interrupt::$USBX_CONTROLLER);
                    self.clear_pending(Interrupt::$USBX_EP_CONTROL);
                    self.clear_pending(Interrupt::$USBX_EP_IN);
                    self.clear_pending(Interrupt::$USBX_EP_OUT);

                    // enable device controller events for bus reset signal
                    self.enable_interrupt(Interrupt::$USBX_CONTROLLER);
                    self.enable_interrupt(Interrupt::$USBX_EP_CONTROL);
                    self.enable_interrupt(Interrupt::$USBX_EP_IN);
                    self.enable_interrupt(Interrupt::$USBX_EP_OUT);
                }

                pub fn is_pending(&self, interrupt: Interrupt) -> bool {
                    pac::csr::interrupt::pending(interrupt)
                }

                // TODO &mut self
                pub fn clear_pending(&self, interrupt: Interrupt) {
                    match interrupt {
                        Interrupt::$USBX_CONTROLLER => self
                            .controller
                            .ev_pending
                            .modify(|r, w| w.pending().bit(r.pending().bit())),
                        Interrupt::$USBX_EP_CONTROL => self
                            .ep_control
                            .ev_pending
                            .modify(|r, w| w.pending().bit(r.pending().bit())),
                        Interrupt::$USBX_EP_IN => self
                            .ep_in
                            .ev_pending
                            .modify(|r, w| w.pending().bit(r.pending().bit())),
                        Interrupt::$USBX_EP_OUT => self
                            .ep_out
                            .ev_pending
                            .modify(|r, w| w.pending().bit(r.pending().bit())),
                        _ => {
                            warn!("Ignoring invalid interrupt clear pending: {:?}", interrupt);
                        }
                    }
                }

                // TODO &mut self
                pub fn enable_interrupt(&self, interrupt: Interrupt) {
                    match interrupt {
                        Interrupt::$USBX_CONTROLLER => self
                            .controller
                            .ev_enable
                            .write(|w| w.enable().bit(true)),
                        Interrupt::$USBX_EP_CONTROL => self
                            .ep_control
                            .ev_enable
                            .write(|w| w.enable().bit(true)),
                        Interrupt::$USBX_EP_IN => self
                            .ep_in
                            .ev_enable
                            .write(|w| w.enable().bit(true)),
                        Interrupt::$USBX_EP_OUT => self
                            .ep_out
                            .ev_enable
                            .write(|w| w.enable().bit(true)),
                        _ => {
                            warn!("Ignoring invalid interrupt enable: {:?}", interrupt);
                        }
                    }
                }

                // TODO &mut self
                pub fn disable_interrupt(&self, interrupt: Interrupt) {
                    match interrupt {
                        Interrupt::$USBX_CONTROLLER => self
                            .controller
                            .ev_enable
                            .write(|w| w.enable().bit(false)),
                        Interrupt::$USBX_EP_CONTROL => self
                            .ep_control
                            .ev_enable
                            .write(|w| w.enable().bit(false)),
                        Interrupt::$USBX_EP_IN => self
                            .ep_in
                            .ev_enable
                            .write(|w| w.enable().bit(false)),
                        Interrupt::$USBX_EP_OUT => self
                            .ep_out
                            .ev_enable
                            .write(|w| w.enable().bit(false)),
                        _ => {
                            warn!("Ignoring invalid interrupt enable: {:?}", interrupt);
                        }
                    }
                }

                /// Prepare endpoint to receive a single OUT packet.
                pub fn ep_out_prime_receive(&self, endpoint: u8) {
                    // clear receive buffer
                    self.ep_out.reset.write(|w| w.reset().bit(true));

                    // select endpoint
                    self.ep_out
                        .epno
                        .write(|w| unsafe { w.epno().bits(endpoint) });

                    // prime endpoint
                    self.ep_out.prime.write(|w| w.prime().bit(true));

                    // enable it
                    self.ep_out.enable.write(|w| w.enable().bit(true));
                }

                pub fn ep_control_address(&self) -> u8 {
                    self.ep_control.address.read().address().bits()
                }
            }

            // - trait: UsbDriverOperations -----------------------------------------------

            impl UsbDriverOperations for $USBX {
                /// Set the interface up for new connections
                fn connect(&self) -> u8 {
                    // disconnect device controller
                    self.controller.connect.write(|w| w.connect().bit(false));

                    // disable endpoint events
                    self.disable_interrupt(Interrupt::$USBX_CONTROLLER);
                    self.disable_interrupt(Interrupt::$USBX_EP_CONTROL);
                    self.disable_interrupt(Interrupt::$USBX_EP_IN);
                    self.disable_interrupt(Interrupt::$USBX_EP_OUT);

                    // reset FIFOs
                    self.ep_control.reset.write(|w| w.reset().bit(true));
                    self.ep_in.reset.write(|w| w.reset().bit(true));
                    self.ep_out.reset.write(|w| w.reset().bit(true));

                    // connect device controller
                    self.controller.connect.write(|w| w.connect().bit(true));

                    // 0: High, 1: Full, 2: Low, 3:SuperSpeed (incl SuperSpeed+)
                    self.controller.speed.read().speed().bits()
                }

                fn reset(&self) -> u8 {
                    // disable endpoint events
                    self.disable_interrupt(Interrupt::$USBX_EP_CONTROL);
                    self.disable_interrupt(Interrupt::$USBX_EP_IN);
                    self.disable_interrupt(Interrupt::$USBX_EP_OUT);

                    // reset device address to 0
                    self.ep_control
                        .address
                        .write(|w| unsafe { w.address().bits(0) });

                    // reset FIFOs
                    self.ep_control.reset.write(|w| w.reset().bit(true));
                    self.ep_in.reset.write(|w| w.reset().bit(true));
                    self.ep_out.reset.write(|w| w.reset().bit(true));

                    self.enable_interrupts();

                    // TODO handle speed
                    // 0: High, 1: Full, 2: Low, 3:SuperSpeed (incl SuperSpeed+)
                    let speed = self.controller.speed.read().speed().bits();
                    trace!("UsbInterface0::reset() -> {}", speed);
                    speed
                }

                /// Acknowledge the status stage of an incoming control request.
                fn ack_status_stage(&self, packet: &SetupPacket) {
                    match Direction::from(packet.request_type) {
                        // If this is an IN request, read a zero-length packet (ZLP) from the host..
                        Direction::DeviceToHost => self.ep_out_prime_receive(0),
                        // ... otherwise, send a ZLP.
                        Direction::HostToDevice => self.write(0, [].into_iter()),
                    }
                }

                fn ack(&self, endpoint: u8, packet: &SetupPacket) {
                    match Direction::from(packet.request_type) {
                        // If this is an IN request, read a zero-length packet (ZLP) from the host..
                        Direction::DeviceToHost => self.ep_out_prime_receive(endpoint),
                        // ... otherwise, send a ZLP.
                        Direction::HostToDevice => self.write(endpoint, [].into_iter()),
                    }
                }

                fn set_address(&self, address: u8) {
                    self.ep_control
                        .address
                        .write(|w| unsafe { w.address().bits(address & 0x7f) });
                    self.ep_out
                        .address
                        .write(|w| unsafe { w.address().bits(address & 0x7f) });
                }

                /// Stalls the current control request.
                fn stall_request(&self) {
                    self.ep_in.epno.write(|w| unsafe { w.epno().bits(0) });
                    self.ep_in.stall.write(|w| w.stall().bit(true));
                    self.ep_out.epno.write(|w| unsafe { w.epno().bits(0) });
                    self.ep_out.stall.write(|w| w.stall().bit(true));
                }

                /// Sets the stall state for the given endpoint address
                ///
                /// TODO endpoint_address is a USB address i.e. masked with 0x80
                /// for direction. It may be more consistent to actually pass
                /// in the direction and peripheral address separately
                fn stall_endpoint(&self, endpoint_address: u8, state: bool) {
                    match Direction::from(endpoint_address) {
                        Direction::HostToDevice => {
                            self.ep_out
                                .epno
                                .write(|w| unsafe { w.epno().bits(endpoint_address & 0xf) });
                            self.ep_out.stall.write(|w| w.stall().bit(state));
                            trace!("  STALL EP_OUT: {} -> {}", endpoint_address, state);
                        }
                        Direction::DeviceToHost => {
                            let endpoint_address = endpoint_address - 0x80; // TODO - see above
                            self.ep_in
                                .epno
                                .write(|w| unsafe { w.epno().bits(endpoint_address & 0xf) });
                            self.ep_in.stall.write(|w| w.stall().bit(state));
                            trace!("  STALL EP_IN: {} -> {}", endpoint_address, state);
                        }
                    }
                }
            }

            // - trait: Read/Write traits -------------------------------------------------

            impl ControlRead for $USBX {
                fn read_control(&self, buffer: &mut [u8]) -> usize {
                    // drain fifo
                    let mut bytes_read = 0;
                    let mut overflow = 0;
                    while self.ep_control.have.read().have().bit() {
                        if bytes_read >= buffer.len() {
                            let _drain = self.ep_control.data.read().data().bits();
                            overflow += 1;
                        } else {
                            buffer[bytes_read] = self.ep_control.data.read().data().bits();
                            bytes_read += 1;
                        }
                    }

                    trace!("  RX CONTROL {} bytes + {} overflow: {:?}", bytes_read, overflow, &buffer[0..bytes_read]);

                    bytes_read
                }
            }

            impl EndpointRead for $USBX {
                fn read(&self, endpoint: u8, buffer: &mut [u8]) -> usize {
                    // drain fifo
                    let mut bytes_read = 0;
                    let mut overflow = 0;
                    while self.ep_out.have.read().have().bit() {
                        if bytes_read >= buffer.len() {
                            let _drain = self.ep_out.data.read().data().bits();
                            overflow += 1;
                        } else {
                            buffer[bytes_read] = self.ep_out.data.read().data().bits();
                            bytes_read += 1;
                        }
                    }

                    // re-enable endpoint after consuming all data
                    self.ep_out.enable.write(|w| w.enable().bit(true));

                    trace!("  RX OUT{} {} bytes + {} overflow: {:?}", endpoint, bytes_read, overflow, &buffer[0..bytes_read]);

                    // TODO prime endpoints - this is dodgy af
                    for ep in (0..=4).rev() {
                        self.ep_out.epno.write(|w| unsafe { w.epno().bits(ep) });
                        self.ep_out.prime.write(|w| w.prime().bit(true));
                        self.ep_out.enable.write(|w| w.enable().bit(true));
                    }

                    bytes_read
                }
            }

            impl EndpointWrite for $USBX {
                fn write<I>(&self, endpoint: u8, iter: I)
                where
                    I: Iterator<Item = u8>,
                {
                    // reset output fifo if needed
                    if self.ep_in.have.read().have().bit() {
                        trace!("  clear tx");
                        self.ep_in.reset.write(|w| w.reset().bit(true));
                    }

                    // write data
                    let mut bytes_written = 0;
                    for byte in iter {
                        self.ep_in.data.write(|w| unsafe { w.data().bits(byte) });
                        bytes_written += 1;
                    }

                    // finally, prime IN endpoint
                    self.ep_in
                        .epno
                        .write(|w| unsafe { w.epno().bits(endpoint & 0xf) });

                    trace!("  TX {} bytes", bytes_written);
                }
            }

            impl EndpointWriteRef for $USBX {
                fn write_ref<'a, I>(&self, endpoint: u8, iter: I)
                where
                    I: Iterator<Item = &'a u8>,
                {
                    // reset output fifo if needed
                    if self.ep_in.have.read().have().bit() {
                        trace!("  clear tx");
                        self.ep_in.reset.write(|w| w.reset().bit(true));
                    }

                    // write data
                    let mut bytes_written = 0;
                    for &byte in iter {
                        self.ep_in.data.write(|w| unsafe { w.data().bits(byte) });
                        bytes_written += 1;
                    }

                    // finally, prime IN endpoint
                    self.ep_in
                        .epno
                        .write(|w| unsafe { w.epno().bits(endpoint & 0xf) });

                    trace!("  TX {} bytes", bytes_written);
                }
            }

            // mark implementation as complete
            impl UsbDriver for $USBX {}
        )+
    }
}

impl_usb! {
    Usb0: USB0, USB0_EP_CONTROL, USB0_EP_IN, USB0_EP_OUT,
    Usb1: USB1, USB1_EP_CONTROL, USB1_EP_IN, USB1_EP_OUT,
    Usb2: USB2, USB2_EP_CONTROL, USB2_EP_IN, USB2_EP_OUT,
}
