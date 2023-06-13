#![allow(dead_code, unused_imports, unused_variables)] // TODO
#![no_std]
#![no_main]

use moondancer::usb::vendor::{VendorRequest, VendorRequestValue};
use moondancer::{hal, pac, Message};

use pac::csr::interrupt;

use smolusb::class;
use smolusb::control::{Direction, RequestType, SetupPacket};
use smolusb::device::{Speed, UsbDevice};
use smolusb::traits::{
    ControlRead, EndpointRead, EndpointWrite, EndpointWriteRef, UnsafeUsbDriverOperations,
    UsbDriverOperations,
};

use libgreat::gcp::{iter_to_response, GcpResponse, GCP_MAX_RESPONSE_LENGTH};
use libgreat::{GreatError, GreatResult};

use heapless::mpmc::MpMcQueue as Queue;
use log::{debug, error, info, trace, warn};

use core::any::Any;
use core::{array, iter, slice};

// - global static state ------------------------------------------------------

static MESSAGE_QUEUE: Queue<Message, 128> = Queue::new();

#[inline(always)]
fn dispatch_message(message: Message) {
    match MESSAGE_QUEUE.enqueue(message) {
        Ok(()) => (),
        Err(_) => {
            error!("MachineExternal - message queue overflow");
            //panic!("MachineExternal - message queue overflow");
            loop {
                unsafe { riscv::asm::nop(); }
            }
        }
    }
}

// - MachineExternal interrupt handler ----------------------------------------

#[allow(non_snake_case)]
#[no_mangle]
fn MachineExternal() {
    use moondancer::UsbInterface::{Aux, Target};

    // peripherals
    let peripherals = unsafe { pac::Peripherals::steal() };
    let usb0 = unsafe { hal::Usb0::summon() };
    let usb1 = unsafe { hal::Usb1::summon() };

    let pending = interrupt::reg_pending();

    // - usb1 interrupts - "aux_phy" (host on r0.4) --

    // USB1 UsbBusReset
    if usb1.is_pending(pac::Interrupt::USB1) {
        usb1.clear_pending(pac::Interrupt::USB1);
        usb1.bus_reset();
        dispatch_message(Message::UsbBusReset(Aux));

    // USB1_EP_CONTROL UsbReceiveSetupPacket
    } else if usb1.is_pending(pac::Interrupt::USB1_EP_CONTROL) {
        let mut setup_packet_buffer = [0_u8; 8];
        usb1.read_control(&mut setup_packet_buffer);
        usb1.clear_pending(pac::Interrupt::USB1_EP_CONTROL);
        let message = match SetupPacket::try_from(setup_packet_buffer) {
            Ok(setup_packet) => Message::UsbReceiveSetupPacket(Aux, setup_packet),
            Err(e) => Message::ErrorMessage("USB1_EP_CONTROL failed to read setup packet"),
        };
        dispatch_message(message);

    // USB1_EP_OUT UsbReceiveData
    } else if usb1.is_pending(pac::Interrupt::USB1_EP_OUT) {
        let endpoint = usb1.ep_out.data_ep.read().bits() as u8;
        usb1.clear_pending(pac::Interrupt::USB1_EP_OUT);
        dispatch_message(Message::UsbReceivePacket(Aux, endpoint, 0));

    // USB1_EP_IN UsbTransferComplete
    } else if usb1.is_pending(pac::Interrupt::USB1_EP_IN) {
        let endpoint = usb1.ep_in.epno.read().bits() as u8;
        usb1.clear_pending(pac::Interrupt::USB1_EP_IN);

        // TODO something a little bit safer would be nice
        unsafe {
            usb1.clear_tx_ack_active();
        }

        dispatch_message(Message::UsbTransferComplete(Aux, endpoint));

    // - usb0 interrupts - "target_phy" --

    // USB0 UsbBusReset
    } else if usb0.is_pending(pac::Interrupt::USB0) {
        usb0.clear_pending(pac::Interrupt::USB0);
        dispatch_message(Message::UsbBusReset(Target));

    // USB0_EP_CONTROL UsbReceiveSetupPacket
    } else if usb0.is_pending(pac::Interrupt::USB0_EP_CONTROL) {
        let mut setup_packet_buffer = [0_u8; 8];
        usb0.read_control(&mut setup_packet_buffer);
        usb0.clear_pending(pac::Interrupt::USB0_EP_CONTROL);
        let message = match SetupPacket::try_from(setup_packet_buffer) {
            Ok(setup_packet) => Message::UsbReceiveSetupPacket(Target, setup_packet),
            Err(e) => Message::ErrorMessage("USB0_EP_CONTROL failed to read setup packet"),
        };
        dispatch_message(message);

    // USB0_EP_OUT UsbReceiveData
    } else if usb0.is_pending(pac::Interrupt::USB0_EP_OUT) {
        let endpoint = usb0.ep_out.data_ep.read().bits() as u8;
        usb0.clear_pending(pac::Interrupt::USB0_EP_OUT);
        dispatch_message(Message::UsbReceivePacket(Target, endpoint, 0));

    // USB0_EP_IN UsbTransferComplete
    } else if usb0.is_pending(pac::Interrupt::USB0_EP_IN) {
        let endpoint = usb0.ep_in.epno.read().bits() as u8;
        usb0.clear_pending(pac::Interrupt::USB0_EP_IN);

        // TODO something a little bit safer would be nice
        unsafe {
            usb0.clear_tx_ack_active();
        }

        dispatch_message(Message::UsbTransferComplete(Target, endpoint));

    // - Unknown Interrupt --
    } else {
        dispatch_message(Message::HandleUnknownInterrupt(pending));
    }
}

// - main entry point ---------------------------------------------------------

#[cfg(feature = "vexriscv")]
#[riscv_rt::pre_init]
unsafe fn pre_main() {
    pac::cpu::vexriscv::flush_icache();
    #[cfg(feature = "vexriscv_dcache")]
    pac::cpu::vexriscv::flush_dcache();
}

#[riscv_rt::entry]
fn main() -> ! {
    // initialize firmware
    let mut firmware = Firmware::new(pac::Peripherals::take().unwrap());
    match firmware.initialize() {
        Ok(()) => (),
        Err(e) => {
            error!("Firmware panicked during initialization: {}", e);
            //panic!("Firmware panicked during initialization: {}", e)
        }
    }

    // enter main loop
    match firmware.main_loop() {
        Ok(()) => {
            error!("Firmware exited unexpectedly in main loop");
            //panic!("Firmware exited unexpectedly in main loop")
        }
        Err(e) => {
            error!("Firmware panicked in main loop: {}", e);
            //panic!("Firmware panicked in main loop: {}", e)
        }
    }

    loop {
        unsafe { riscv::asm::nop(); }
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
    core: libgreat::gcp::class_core::Core,
    moondancer: moondancer::gcp::moondancer::Moondancer,
}

impl<'a> Firmware<'a> {
    fn new(peripherals: pac::Peripherals) -> Self {
        // initialize logging
        moondancer::log::init(hal::Serial::new(peripherals.UART));
        info!("Logging initialized");

        // usb1: aux (host on r0.4)
        let mut usb1 = UsbDevice::new(
            hal::Usb1::new(
                peripherals.USB1,
                peripherals.USB1_EP_CONTROL,
                peripherals.USB1_EP_IN,
                peripherals.USB1_EP_OUT,
            ),
            &moondancer::usb::DEVICE_DESCRIPTOR,
            &moondancer::usb::CONFIGURATION_DESCRIPTOR_0,
            &moondancer::usb::USB_STRING_DESCRIPTOR_0,
            &moondancer::usb::USB_STRING_DESCRIPTORS,
        );
        usb1.device_qualifier_descriptor = Some(&moondancer::usb::DEVICE_QUALIFIER_DESCRIPTOR);
        usb1.other_speed_configuration_descriptor =
            Some(moondancer::usb::OTHER_SPEED_CONFIGURATION_DESCRIPTOR_0);

        // usb0: target
        let usb0 = hal::Usb0::new(
            peripherals.USB0,
            peripherals.USB0_EP_CONTROL,
            peripherals.USB0_EP_IN,
            peripherals.USB0_EP_OUT,
        );

        // initialize class registry
        static CLASSES: [libgreat::gcp::Class; 3] = [
            libgreat::gcp::class_core::CLASS,
            moondancer::gcp::firmware::CLASS,
            moondancer::gcp::moondancer::CLASS,
        ];
        let classes = libgreat::gcp::Classes(&CLASSES);

        // initialize classes
        let core = libgreat::gcp::class_core::Core::new(classes, moondancer::BOARD_INFORMATION);
        let moondancer = moondancer::gcp::moondancer::Moondancer::new(usb0);

        Self {
            leds: peripherals.LEDS,
            usb1,
            active_response: None,
            core,
            moondancer,
        }
    }

    fn initialize(&mut self) -> GreatResult<()> {
        // leds: starting up
        self.leds
            .output
            .write(|w| unsafe { w.output().bits(1 << 2) });

        // connect usb1
        let speed = self.usb1.connect();
        debug!("Connected usb1 device: {:?}", speed);

        // enable interrupts
        unsafe {
            // set mstatus register: interrupt enable
            riscv::interrupt::enable();

            // set mie register: machine external interrupts enable
            riscv::register::mie::set_mext();

            // write csr: enable usb1 interrupts and events
            self.enable_usb1_interrupts();
        }

        Ok(())
    }

    #[inline(always)]
    fn main_loop(&'a mut self) -> GreatResult<()> {
        let mut rx_buffer: [u8; moondancer::EP_MAX_PACKET_SIZE] =
            [0; moondancer::EP_MAX_PACKET_SIZE];
        let mut max_queue_length = 0;
        let mut queue_length = 0;

        // leds: ready
        self.leds
            .output
            .write(|w| unsafe { w.output().bits(1 << 1) });

        loop {
            if queue_length > max_queue_length {
                max_queue_length = queue_length;
                debug!("max_queue_length: {}", max_queue_length);
            }
            queue_length = 0;

            while let Some(message) = MESSAGE_QUEUE.dequeue() {
                use moondancer::{
                    Message::*,
                    UsbInterface::{Aux, Target},
                };

                queue_length += 1;

                trace!("MachineExternal: {:?}", message);
                match message {
                    // - usb1 message handlers --

                    // Usb1 received USB bus reset
                    UsbBusReset(Aux) => {
                        // handled in MachineExternal
                    }

                    // Usb1 received setup packet
                    UsbReceiveSetupPacket(Aux, packet) => {
                        self.handle_usb1_receive_setup_packet(packet)?;
                    }

                    // Usb1 transfer complete
                    UsbTransferComplete(Aux, endpoint) => {
                        self.handle_usb1_transfer_complete(endpoint)?;
                        trace!("MachineExternal - USB1_EP_IN {}\n", endpoint);
                    }

                    // Usb1 received data on control endpoint
                    UsbReceivePacket(Aux, 0, _) => {
                        let bytes_read = self.usb1.hal_driver.read(0, &mut rx_buffer);
                        self.handle_usb1_receive_control_data(bytes_read, rx_buffer)?;
                        self.usb1.hal_driver.ep_out_prime_receive(0);
                    }

                    // Usb1 received data on endpoint
                    UsbReceivePacket(Aux, endpoint, _) => {
                        let bytes_read = self.usb1.hal_driver.read(endpoint, &mut rx_buffer);
                        self.handle_usb1_receive_data(endpoint, bytes_read, rx_buffer)?;
                        self.usb1.hal_driver.ep_out_prime_receive(endpoint);
                        debug!(
                            "Usb1 received {} bytes on usb1 endpoint: {}",
                            endpoint,
                            bytes_read,
                        );
                    }

                    // - usb0 message handlers --

                    // Usb0 received USB bus reset
                    UsbBusReset(Target) => {
                        self.moondancer.handle_usb_bus_reset()?;
                    }

                    // Usb0 received setup packet
                    UsbReceiveSetupPacket(Target, packet) => {
                        self.moondancer.handle_usb_receive_setup_packet(packet)?;
                    }

                    // Usb0 transfer complete
                    UsbTransferComplete(Target, endpoint) => {
                        self.moondancer.handle_usb_transfer_complete(endpoint)?;
                        trace!("MachineExternal - USB0_EP_IN {}\n", endpoint);
                    }

                    // Usb0 received data on control endpoint
                    UsbReceivePacket(Target, 0, _) => {
                        // TODO maybe handle the read in moondancer.rs ?
                        let bytes_read = self.moondancer.usb0.read(0, &mut rx_buffer);
                        self.moondancer
                            .handle_usb_receive_control_data(bytes_read, rx_buffer)?;
                        self.moondancer.usb0.ep_out_prime_receive(0);
                    }

                    // Usb0 received data on endpoint
                    UsbReceivePacket(Target, endpoint, _) => {
                        // TODO maybe handle the read in moondancer.rs ?
                        let bytes_read = self.moondancer.usb0.read(endpoint, &mut rx_buffer);
                        self.moondancer
                            .handle_usb_receive_data(endpoint, bytes_read, rx_buffer)?;
                        self.moondancer.usb0.ep_out_prime_receive(endpoint);
                        debug!(
                            "Usb0 received {} bytes on usb0 endpoint: {}",
                            endpoint,
                            bytes_read,
                        );
                    }

                    // Error Message
                    ErrorMessage(message) => {
                        error!("MachineExternal Error - {}\n", message);
                    }

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

    // - usb1 interrupt handlers ----------------------------------------------

    unsafe fn enable_usb1_interrupts(&self) {
        interrupt::enable(pac::Interrupt::USB1);
        interrupt::enable(pac::Interrupt::USB1_EP_CONTROL);
        interrupt::enable(pac::Interrupt::USB1_EP_IN);
        interrupt::enable(pac::Interrupt::USB1_EP_OUT);

        // enable all usb events
        self.usb1.hal_driver.enable_interrupts();
    }

    fn handle_usb1_receive_setup_packet(&mut self, setup_packet: SetupPacket) -> GreatResult<()> {
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
                warn!(" gcp: Unknown vendor request '{:?}'", vendor_request);
                self.usb1.hal_driver.write(0, [0].into_iter());
            }
            _ => match self.usb1.handle_setup_request(&setup_packet) {
                Ok(()) => (),
                Err(e) => {
                    error!("  handle_setup_request: {:?}: {:?}", e, setup_packet);
                    //panic!("  handle_setup_request: {:?}: {:?}", e, setup_packet)
                    GreatError::Message("FATAL: failed to handle setup request");
                }
            },
        }
        Ok(())
    }

    /// Usb1: gcp vendor request handler
    fn usb1_handle_vendor_request(&mut self, setup_packet: &SetupPacket) -> GreatResult<()> {
        let direction = setup_packet.direction();
        let request = VendorRequest::from(setup_packet.request);
        let request_value = VendorRequestValue::from(setup_packet.value);
        let length = setup_packet.length as usize;

        trace!(
            "GCP vendor_request: {:?} dir:{:?} value:{:?} length:{} index:{}",
            request,
            direction,
            request_value,
            length,
            setup_packet.index
        );

        match (&direction, &request, &request_value) {
            // host is starting a new command sequence
            (
                Direction::HostToDevice,
                VendorRequest::UsbCommandRequest,
                VendorRequestValue::Start,
            ) => {
                self.usb1.hal_driver.ack_status_stage(setup_packet);
                trace!("ORDER: #1");
                trace!("GCP TODO state = Command::Begin");
                //trace!("GCP   ack {}", length);
            }

            // host is ready to receive a response
            (
                Direction::DeviceToHost,
                VendorRequest::UsbCommandRequest,
                VendorRequestValue::Start,
            ) => {
                trace!("ORDER: #3");
                trace!("GCP TODO state = Command::Send");
                // do we have a response ready? should we wait if we don't?
                if let Some(response) = &mut self.active_response {
                    // send it
                    trace!("GCP sending command response of {} bytes", response.len());
                    self.usb1
                        .hal_driver
                        .write(0, response.take(setup_packet.length as usize));
                    self.active_response = None;
                } else {
                    // TODO something has gone wrong
                    error!("GCP stall: gcp response requested but no response queued");
                    self.usb1.hal_driver.stall_request();
                }
                trace!("ORDER: fin");
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
                trace!("GCP TODO state = Command::Cancel");
                debug!(
                    "GCP TODO cancel cynthion vendor request sequence: {}",
                    length
                );
            }
            _ => {
                error!(
                    "GCP stall: unknown vendor request and/or value: {:?} {:?} {:?}",
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
        buffer: [u8; moondancer::EP_MAX_PACKET_SIZE],
    ) -> GreatResult<()> {
        // TODO state == Command::Send

        trace!(
            "GCP Received {} bytes on usb1 control endpoint: {:?}",
            bytes_read,
            &buffer[0..bytes_read]
        );

        if bytes_read < 8 {
            // short read
            //warn!("GCP   short read of {} bytes\n", bytes_read);
            return Ok(());
        }

        // parse & dispatch command
        if let Some(command) = libgreat::gcp::Command::parse(&buffer[0..bytes_read]) {
            trace!("ORDER: #2");
            trace!("GCP dispatching command: {:?}", command);
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
                    trace!("GCP queueing next response");
                    self.active_response = Some(response);
                    //self.usb1.hal_driver.ep_out_prime_receive(0);
                    //self.usb1.hal_driver.write(0, [].into_iter());
                }
                Err(e) => {
                    error!("GCP stall: failed to dispatch command {}", e);
                    error!("    {:?}", command);
                    self.usb1.hal_driver.stall_request();
                }
            }
            trace!("\n");
        }
        Ok(())
    }

    fn handle_usb1_receive_data(
        &mut self,
        endpoint: u8,
        bytes_read: usize,
        buffer: [u8; moondancer::EP_MAX_PACKET_SIZE],
    ) -> GreatResult<()> {
        Ok(())
    }

    pub fn handle_usb1_transfer_complete(&mut self, endpoint: u8) -> GreatResult<()> {
        Ok(())
    }

    fn dispatch_gcp_command(
        &mut self,
        class_id: libgreat::gcp::ClassId,
        verb_id: u32,
        arguments: &[u8],
        response_buffer: [u8; GCP_MAX_RESPONSE_LENGTH],
    ) -> GreatResult<GcpResponse> {
        let no_context: Option<u8> = None;

        match (class_id, verb_id) {
            // class: core
            (libgreat::gcp::ClassId::core, verb_id) => {
                self.core.dispatch(verb_id, arguments, response_buffer)
            }
            // class: firmware
            (libgreat::gcp::ClassId::firmware, verb_id) => {
                moondancer::gcp::firmware::dispatch(verb_id, arguments, response_buffer)
            }
            // class: moondancer
            (libgreat::gcp::ClassId::moondancer, verb_id) => {
                self.moondancer
                    .dispatch(verb_id, arguments, response_buffer)
            }

            _ => Err(GreatError::Message("class or verb not found")),
        }
    }
}
