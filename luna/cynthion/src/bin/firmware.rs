#![allow(dead_code, unused_imports, unused_variables)] // TODO
#![no_std]
#![no_main]

use cynthion::pac;
use pac::csr::interrupt;

use cynthion::hal;
use hal::smolusb;
use smolusb::class;
use smolusb::class::cynthion::vendor::{VendorRequest, VendorRequestValue};
use smolusb::control::{Direction, RequestType, SetupPacket};
use smolusb::device::{Speed, UsbDevice};
use smolusb::traits::{
    ControlRead, EndpointRead, EndpointWrite, EndpointWriteRef, UsbDriverOperations,
};

use libgreat::GreatError;
use libgreat::gcp::{self, iter_to_response, GcpResponse, GCP_MAX_RESPONSE_LENGTH};

use log::{debug, error, info, trace, warn};
use riscv_rt::entry;

use core::any::Any;
use core::array;
use core::iter;
use core::slice;

// - global constants ---------------------------------------------------------

// - global static state ------------------------------------------------------

use cynthion::Message;
use heapless::mpmc::MpMcQueue as Queue;
static MESSAGE_QUEUE: Queue<Message, 32> = Queue::new();

// - MachineExternal interrupt handler ----------------------------------------

#[allow(non_snake_case)]
#[no_mangle]
fn MachineExternal() {
    // peripherals
    let peripherals = unsafe { pac::Peripherals::steal() };
    let usb0 = unsafe { hal::Usb1::summon() };
    let usb1 = unsafe { hal::Usb1::summon() };

    let pending = interrupt::reg_pending();

    // - usb1 interrupts - "host_phy" --

    // USB1 HandleInterrupt
    let message = if usb1.is_pending(pac::Interrupt::USB1) {
        usb1.clear_pending(pac::Interrupt::USB1);
        Message::HandleInterrupt(pac::Interrupt::USB1)

    // USB1_EP_CONTROL UsbReceiveSetupPacket
    } else if usb1.is_pending(pac::Interrupt::USB1_EP_CONTROL) {
        let mut setup_packet_buffer = [0_u8; 8];
        usb1.read_control(&mut setup_packet_buffer);
        let setup_packet = match SetupPacket::try_from(setup_packet_buffer) {
            Ok(packet) => packet,
            Err(e) => {
                error!("MachineExternal USB1_EP_CONTROL - {:?}", e);
                return;
            }
        };
        usb1.clear_pending(pac::Interrupt::USB1_EP_CONTROL);
        Message::UsbReceiveSetupPacket(1, setup_packet)

    // USB1_EP_IN transfer complete
    } else if usb1.is_pending(pac::Interrupt::USB1_EP_IN) {
        usb1.clear_pending(pac::Interrupt::USB1_EP_IN);
        Message::HandleInterrupt(pac::Interrupt::USB1_EP_IN)

    // USB1_EP_OUT UsbReceiveData
    } else if usb1.is_pending(pac::Interrupt::USB1_EP_OUT) {
        let endpoint = usb1.ep_out.data_ep.read().bits() as u8;
        let mut buffer = [0_u8; cynthion::EP_MAX_RECEIVE_LENGTH];
        let bytes_read = usb1.read(endpoint, &mut buffer);
        usb1.clear_pending(pac::Interrupt::USB1_EP_OUT);
        Message::UsbReceiveData(1, endpoint, bytes_read, buffer)

    // - usb0 interrupts - "target_phy" --

    // USB0 HandleInterrupt
    } else if usb0.is_pending(pac::Interrupt::USB0) {
        usb0.clear_pending(pac::Interrupt::USB0);
        Message::HandleInterrupt(pac::Interrupt::USB0)

    // USB0_EP_CONTROL UsbReceiveSetupPacket
    } else if usb0.is_pending(pac::Interrupt::USB0_EP_CONTROL) {
        let mut setup_packet_buffer = [0_u8; 8];
        usb0.read_control(&mut setup_packet_buffer);
        let setup_packet = match SetupPacket::try_from(setup_packet_buffer) {
            Ok(packet) => packet,
            Err(e) => {
                error!("MachineExternal USB0_EP_CONTROL - {:?}", e);
                return;
            }
        };
        usb0.clear_pending(pac::Interrupt::USB0_EP_CONTROL);
        Message::UsbReceiveSetupPacket(0, setup_packet)

    // USB0_EP_IN transfer complete
    } else if usb0.is_pending(pac::Interrupt::USB0_EP_IN) {
        usb0.clear_pending(pac::Interrupt::USB0_EP_IN);
        Message::HandleInterrupt(pac::Interrupt::USB0_EP_IN)

    // USB0_EP_OUT UsbReceiveData
    } else if usb0.is_pending(pac::Interrupt::USB0_EP_OUT) {
        let endpoint = usb0.ep_out.data_ep.read().bits() as u8;
        let mut buffer = [0_u8; cynthion::EP_MAX_RECEIVE_LENGTH];
        let bytes_read = usb0.read(endpoint, &mut buffer);
        usb0.clear_pending(pac::Interrupt::USB0_EP_OUT);
        Message::UsbReceiveData(0, endpoint, bytes_read, buffer)

    // - Unknown Interrupt --
    } else {
        Message::HandleUnknownInterrupt(pending)
    };

    match MESSAGE_QUEUE.enqueue(message) {
        Ok(()) => (),
        Err(_) => error!("MachineExternal - message queue overflow"),
    }
}

// - main entry point ---------------------------------------------------------

#[entry]
fn main() -> ! {
    // initialize firmware
    let mut firmware = Firmware::new(pac::Peripherals::take().unwrap());
    match firmware.initialize() {
        Ok(()) => (),
        Err(e) => {
            error!("Firmware panicked during initialization: {}", e);
            panic!("Firmware panicked during initialization: {}", e)
        },
    }

    // enter main loop
    match firmware.main_loop() {
        Ok(()) => {
            error!("Firmware exited unexpectedly in main loop");
            panic!("Firmware exited unexpectedly in main loop")
        },
        Err(e) => {
            error!("Firmware panicked in main loop: {}", e);
            panic!("Firmware panicked in main loop: {}", e)
        },
    }
}

// - Firmware -----------------------------------------------------------------

struct Firmware<'a> {
    // peripherals
    leds: pac::LEDS,
    usb1: UsbDevice<'a, hal::Usb1>,

    // state
    active_response: Option<GcpResponse<'a>>,

    // classes
    core: gcp::class_core::Core,
    greatdancer: cynthion::class::greatdancer::Greatdancer<'a>,
}

impl<'a> Firmware<'a> {
    fn new(peripherals: pac::Peripherals) -> Self {
        // initialize logging
        cynthion::log::init(hal::Serial::new(peripherals.UART));
        trace!("logging initialized");

        // usb1: host
        let usb1 = UsbDevice::new(
            hal::Usb1::new(
                peripherals.USB1,
                peripherals.USB1_EP_CONTROL,
                peripherals.USB1_EP_IN,
                peripherals.USB1_EP_OUT,
            ),
            &class::cynthion::DEVICE_DESCRIPTOR,
            &class::cynthion::CONFIGURATION_DESCRIPTOR_0,
            &class::cynthion::USB_STRING_DESCRIPTOR_0,
            &class::cynthion::USB_STRING_DESCRIPTORS,
        );

        // usb0: target
        let usb0 = UsbDevice::new(
            hal::Usb0::new(
                peripherals.USB0,
                peripherals.USB0_EP_CONTROL,
                peripherals.USB0_EP_IN,
                peripherals.USB0_EP_OUT,
            ),
            &class::cynthion::DEVICE_DESCRIPTOR,
            &class::cynthion::CONFIGURATION_DESCRIPTOR_0,
            &class::cynthion::USB_STRING_DESCRIPTOR_0,
            &class::cynthion::USB_STRING_DESCRIPTORS,
        );

        // initialize class registry
        static CLASSES: [gcp::Class; 2] =
            [gcp::class_core::CLASS, cynthion::class::firmware::CLASS];
        let classes = gcp::Classes(&CLASSES);

        // initialize classes
        let core = gcp::class_core::Core::new(classes, cynthion::BOARD_INFORMATION);
        let greatdancer = cynthion::class::greatdancer::Greatdancer::new(usb0);

        Self {
            leds: peripherals.LEDS,
            usb1,
            active_response: None,
            core,
            greatdancer,
        }
    }

    fn initialize(&mut self) -> cynthion::GreatResult<()> {
        // leds: starting up
        self.leds
            .output
            .write(|w| unsafe { w.output().bits(1 << 2) });

        // connect usb1
        let speed = self.usb1.hal_driver.connect();
        trace!("Connected usb1 device: {:?}", Speed::from(speed));

        // enable interrupts
        self.usb1.hal_driver.enable_interrupts();
        unsafe {
            // set mstatus register: interrupt enable
            riscv::interrupt::enable();

            // set mie register: machine external interrupts enable
            riscv::register::mie::set_mext();

            // write csr: enable interrupts
            interrupt::enable(pac::Interrupt::USB1);
            interrupt::enable(pac::Interrupt::USB1_EP_CONTROL);
            interrupt::enable(pac::Interrupt::USB1_EP_IN);
            interrupt::enable(pac::Interrupt::USB1_EP_OUT);
        }

        // leds: ready
        self.leds
            .output
            .write(|w| unsafe { w.output().bits(1 << 1) });

        Ok(())
    }

    #[inline(always)]
    fn main_loop(&'a mut self) -> cynthion::GreatResult<()> {
        // leds: main loop
        self.leds
            .output
            .write(|w| unsafe { w.output().bits(1 << 0) });

        loop {
            while let Some(message) = MESSAGE_QUEUE.dequeue() {
                match message {
                    // - usb1 message handlers --

                    // Usb1 received setup packet
                    Message::UsbReceiveSetupPacket(1, packet) => {
                        self.handle_usb1_receive_setup_packet(packet)?;
                    }

                    // Usb1 received data on control endpoint
                    Message::UsbReceiveData(1, 0, bytes_read, buffer) => {
                        self.handle_usb1_receive_control_data(bytes_read, buffer)?;
                    }

                    // Usb1 received data on endpoint
                    Message::UsbReceiveData(1, endpoint, bytes_read, buffer) => {
                        self.handle_usb1_receive_data(endpoint, bytes_read, buffer)?;
                        debug!(
                            "Received {} bytes on usb1 endpoint: {} - {:?}",
                            endpoint,
                            bytes_read,
                            &buffer[0..bytes_read]
                        );
                    }

                    // Usb1 interrupts
                    Message::HandleInterrupt(pac::Interrupt::USB1) => {
                        self.usb1.reset();
                        trace!("MachineExternal - USB1\n");
                    }
                    Message::HandleInterrupt(pac::Interrupt::USB1_EP_IN) => {
                        // TODO
                        trace!("MachineExternal - USB1_EP_IN\n");
                    }

                    // - usb0 message handlers --

                    // TODO

                    // Unhandled message
                    _ => {
                        error!("Unhandled message: {:?}\n", message);
                    }
                }
            }
        }

        #[allow(unreachable_code)] // TODO
        Ok(())
    }

    fn handle_usb1_receive_setup_packet(
        &mut self,
        setup_packet: SetupPacket,
    ) -> cynthion::GreatResult<()> {
        let request_type = setup_packet.request_type();
        let vendor_request = VendorRequest::from(setup_packet.request);

        match (&request_type, &vendor_request) {
            (RequestType::Vendor, VendorRequest::UsbCommandRequest) => {
                self.usb1_handle_vendor_request(&setup_packet)?;
            }
            (RequestType::Vendor, vendor_request) => {
                // TODO this is from one of the legacy boards which we
                // need to support to get `greatfet info` to finish
                // enumerating through the supported devices.
                //
                // see: host/greatfet/boards/legacy.py
                error!(" gcp: Unknown vendor request '{:?}'", vendor_request);
                self.usb1.hal_driver.write(0, [0].into_iter());
            }
            _ => match self.usb1.handle_setup_request(&setup_packet) {
                Ok(()) => debug!("OK\n"),
                Err(e) => {
                    error!("  handle_setup_request: {:?}: {:?}", e, setup_packet);
                    panic!("  handle_setup_request: {:?}: {:?}", e, setup_packet)
                },
            },
        }
        Ok(())
    }

    /// Usb1: gcp vendor request handler
    fn usb1_handle_vendor_request(&mut self, setup_packet: &SetupPacket) -> cynthion::GreatResult<()> {
        let direction = setup_packet.direction();
        let request = VendorRequest::from(setup_packet.request);
        let request_value = VendorRequestValue::from(setup_packet.value);
        let length = setup_packet.length as usize;

        debug!(
            "  gcp: CYNTHION vendor_request: {:?} dir:{:?} value:{:?} length:{} index:{}",
            request, direction, request_value, length, setup_packet.index
        );

        match (&direction, &request, &request_value) {
            // host is starting a new command sequence
            (
                Direction::HostToDevice,
                VendorRequest::UsbCommandRequest,
                VendorRequestValue::Start,
            ) => {
                self.usb1.hal_driver.ack_status_stage(setup_packet);
                debug!("ORDER: #1");
                debug!("  gcp: TODO state = Command::Begin");
                debug!("  gcp: ack {}", length);
            }

            // host is ready to receive a response
            (
                Direction::DeviceToHost,
                VendorRequest::UsbCommandRequest,
                VendorRequestValue::Start,
            ) => {
                debug!("ORDER: #3");
                debug!("  gcp: TODO state = Command::Send");
                // do we have a response ready? should we wait if we don't?
                if let Some(response) = &mut self.active_response {
                    // send it
                    debug!(
                        "  gcp: sending command response of {} bytes",
                        response.len()
                    );
                    self.usb1
                        .hal_driver
                        .write(0, response.take(setup_packet.length as usize));
                    self.active_response = None;
                } else {
                    // TODO something has gone wrong
                    error!("  gcp: stall: gcp response requested but no response queued");
                    self.usb1.hal_driver.stall_request();
                }
                debug!("ORDER: fin");
            }

            // host would like to abort the current command sequence
            (
                Direction::DeviceToHost,
                VendorRequest::UsbCommandRequest,
                VendorRequestValue::Cancel,
            ) => {
                // cancel any queued response
                self.active_response = None;

                // TODO - how long? ack?
                self.usb1
                    .hal_driver
                    .write(0, [0xde, 0xad, 0xde, 0xad].into_iter());
                //self.usb1.hal_driver.ack_status_stage(setup_packet);
                //self.usb1.hal_driver.stall_request();

                // TODO cancel
                debug!("  gcp: TODO state = Command::Cancel");
                debug!(
                    "  gcp: TODO cancel cynthion vendor request sequence: {}",
                    length
                );
            }
            _ => {
                error!(
                    "  gcp: stall: unknown vendor request and/or value: {:?} {:?} {:?}",
                    direction, request, request_value
                );
                self.usb1.hal_driver.stall_request();
            }
        }

        Ok(())
    }

    fn handle_usb1_receive_control_data(
        &mut self,
        bytes_read: usize,
        buffer: [u8; cynthion::EP_MAX_RECEIVE_LENGTH],
    ) -> cynthion::GreatResult<()> {
        // TODO state == Command::Send

        debug!(
            "  gcp: Received {} bytes on usb1 control endpoint: {:?}",
            bytes_read,
            &buffer[0..bytes_read]
        );

        if bytes_read < 8 {
            // short read
            warn!("  gcp: short read of {} bytes\n", bytes_read);
            return Ok(());
        }

        // parse & dispatch command
        if let Some(command) = gcp::Command::parse(&buffer[0..bytes_read]) {
            debug!("ORDER: #2");
            debug!("  gcp: dispatching command: {:?}", command);
            // let response = self.classes.dispatch(command, &self.some_state);
            let response_buffer: [u8; GCP_MAX_RESPONSE_LENGTH] = [0; GCP_MAX_RESPONSE_LENGTH];
            let response = self.dispatch_gcp_command(
                command.class_id(),
                command.verb_number(),
                command.arguments,
                response_buffer,
            );
            match response {
                Ok(response) => {
                    // TODO we really need a better way to get this to the vendor request
                    // NEXT so what's happening with greatfet info is that we queue
                    //      the response but the host errors out before we get the
                    //      vendor_request telling us we can send it ???
                    debug!("  gcp: queueing next response");
                    self.active_response = Some(response);
                }
                Err(e) => {
                    error!("  gcp: stall: failed to dispatch command {}", e);
                    self.usb1.hal_driver.stall_request();
                }
            }
            debug!("\n");
        }
        Ok(())
    }

    fn handle_usb1_receive_data(
        &mut self,
        endpoint: u8,
        bytes_read: usize,
        buffer: [u8; cynthion::EP_MAX_RECEIVE_LENGTH],
    ) -> cynthion::GreatResult<()> {
        Ok(())
    }

    fn dispatch_gcp_command(
        &self,
        class_id: gcp::ClassId,
        verb_id: u32,
        arguments: &[u8],
        response_buffer: [u8; GCP_MAX_RESPONSE_LENGTH],
    ) -> cynthion::GreatResult<GcpResponse> {
        let no_context: Option<u8> = None;

        match (class_id, verb_id) {
            // class: core
            (gcp::ClassId::core, verb_id) => {
                self.core.dispatch(verb_id, arguments, response_buffer)
            }
            // class: firmware
            (gcp::ClassId::firmware, verb_id) => {
                cynthion::class::firmware::dispatch(verb_id, arguments, response_buffer)
            }
            // class: greatdancer
            (gcp::ClassId::greatdancer, verb_id) => {
                self.greatdancer
                    .dispatch(verb_id, arguments, response_buffer)
            }

            _ => Err(GreatError::Message(
                "class or verb not found",
            )),
        }
    }
}
