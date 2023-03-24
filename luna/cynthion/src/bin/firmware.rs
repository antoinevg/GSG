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

use libgreat::gcp::{self, iter_to_response, GcpResponse, GCP_MAX_RESPONSE_LENGTH};
use libgreat::{GreatError, GreatResult};

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
    let timer = unsafe { hal::Timer::summon() };
    let usb0 = unsafe { hal::Usb0::summon() };
    let usb1 = unsafe { hal::Usb1::summon() };

    let pending = interrupt::reg_pending();

    // - usb1 interrupts - "host_phy" --

    // USB1 UsbBusReset
    let message = if usb1.is_pending(pac::Interrupt::USB1) {
        usb1.clear_pending(pac::Interrupt::USB1);
        usb1.bus_reset();
        return;

    // USB1_EP_CONTROL UsbReceiveSetupPacket
    } else if usb1.is_pending(pac::Interrupt::USB1_EP_CONTROL) {
        let mut setup_packet_buffer = [0_u8; 8];
        usb1.read_control(&mut setup_packet_buffer);
        usb1.clear_pending(pac::Interrupt::USB1_EP_CONTROL);
        match SetupPacket::try_from(setup_packet_buffer) {
            Ok(setup_packet) => Message::UsbReceiveSetupPacket(1, setup_packet),
            Err(e) => Message::ErrorMessage("USB1_EP_CONTROL failed to read setup packet"),
        }

    // USB1_EP_IN UsbTransferComplete
    } else if usb1.is_pending(pac::Interrupt::USB1_EP_IN) {
        usb1.clear_pending(pac::Interrupt::USB1_EP_IN);
        let endpoint = usb1.ep_in.epno.read().bits() as u8;

        // TODO something a little bit safer would be nice
        unsafe {
            smolusb::device::USB1_TX_ACK_ACTIVE = false;
        }

        Message::UsbTransferComplete(1, endpoint)

    // USB1_EP_OUT UsbReceiveData
    } else if usb1.is_pending(pac::Interrupt::USB1_EP_OUT) {
        let endpoint = usb1.ep_out.data_ep.read().bits() as u8;
        let mut buffer = [0_u8; cynthion::EP_MAX_RECEIVE_LENGTH];
        let bytes_read = usb1.read(endpoint, &mut buffer);
        usb1.clear_pending(pac::Interrupt::USB1_EP_OUT);

        Message::UsbReceiveData(1, endpoint, bytes_read, buffer)

    // - usb0 interrupts - "target_phy" --

    // USB0 UsbBusReset
    } else if usb0.is_pending(pac::Interrupt::USB0) {
        usb0.clear_pending(pac::Interrupt::USB0);
        usb0.bus_reset();
        return;

    // USB0_EP_CONTROL UsbReceiveSetupPacket
    } else if usb0.is_pending(pac::Interrupt::USB0_EP_CONTROL) {
        let mut setup_packet_buffer = [0_u8; 8];
        usb0.read_control(&mut setup_packet_buffer);
        usb0.clear_pending(pac::Interrupt::USB0_EP_CONTROL);
        match SetupPacket::try_from(setup_packet_buffer) {
            Ok(setup_packet) => Message::UsbReceiveSetupPacket(0, setup_packet),
            Err(e) => Message::ErrorMessage("USB0_EP_CONTROL failed to read setup packet"),
        }

    // USB0_EP_IN UsbTransferComplete
    } else if usb0.is_pending(pac::Interrupt::USB0_EP_IN) {
        let endpoint = usb0.ep_in.epno.read().bits() as u8;
        usb0.clear_pending(pac::Interrupt::USB0_EP_IN);

        // TODO something a little bit safer would be nice
        unsafe {
            smolusb::device::USB0_TX_ACK_ACTIVE = false;
        }

        Message::UsbTransferComplete(0, endpoint)

    // USB0_EP_OUT UsbReceiveData
    } else if usb0.is_pending(pac::Interrupt::USB0_EP_OUT) {
        let endpoint = usb0.ep_out.data_ep.read().bits() as u8;
        let mut buffer = [0_u8; cynthion::EP_MAX_RECEIVE_LENGTH];
        let bytes_read = usb0.read(endpoint, &mut buffer);
        usb0.clear_pending(pac::Interrupt::USB0_EP_OUT);

        Message::UsbReceiveData(0, endpoint, bytes_read, buffer)
    } else if timer.is_pending() {
        timer.clear_pending();
        Message::TimerEvent(0)

    // - Unknown Interrupt --
    } else {
        Message::HandleUnknownInterrupt(pending)
    };

    match MESSAGE_QUEUE.enqueue(message) {
        Ok(()) => (),
        Err(_) => {
            error!("MachineExternal - message queue overflow");
            panic!("MachineExternal - message queue overflow");
        }
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
        }
    }

    // enter main loop
    match firmware.main_loop() {
        Ok(()) => {
            error!("Firmware exited unexpectedly in main loop");
            panic!("Firmware exited unexpectedly in main loop")
        }
        Err(e) => {
            error!("Firmware panicked in main loop: {}", e);
            panic!("Firmware panicked in main loop: {}", e)
        }
    }
}

// - Firmware -----------------------------------------------------------------

struct Firmware<'a> {
    // peripherals
    leds: pac::LEDS,
    timer: hal::Timer,
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

        // timer
        let timer = hal::Timer::new(peripherals.TIMER, pac::clock::sysclk());

        // usb1: host
        let mut usb1 = UsbDevice::new(
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
        usb1.device_qualifier_descriptor = Some(&class::cynthion::DEVICE_QUALIFIER_DESCRIPTOR);
        usb1.other_speed_configuration_descriptor =
            Some(class::cynthion::OTHER_SPEED_CONFIGURATION_DESCRIPTOR_0);

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
        static CLASSES: [gcp::Class; 3] = [
            gcp::class_core::CLASS,
            cynthion::class::firmware::CLASS,
            cynthion::class::greatdancer::CLASS,
        ];
        let classes = gcp::Classes(&CLASSES);

        // initialize classes
        let core = gcp::class_core::Core::new(classes, cynthion::BOARD_INFORMATION);
        let greatdancer = cynthion::class::greatdancer::Greatdancer::new(usb0);

        Self {
            leds: peripherals.LEDS,
            timer,
            usb1,
            active_response: None,
            core,
            greatdancer,
        }
    }

    fn initialize(&mut self) -> GreatResult<()> {
        // leds: starting up
        self.leds
            .output
            .write(|w| unsafe { w.output().bits(1 << 2) });

        // configure and enable timer
        let one_second = pac::clock::sysclk();
        self.timer.set_timeout_ticks(one_second / 10);
        //self.timer.enable();
        //self.timer.listen(hal::timer::Event::TimeOut);

        // connect usb1
        let speed = self.usb1.connect();
        debug!("Connected usb1 device: {:?}", speed);

        // enable interrupts
        unsafe {
            // set mstatus register: interrupt enable
            riscv::interrupt::enable();

            // set mie register: machine external interrupts enable
            riscv::register::mie::set_mext();

            // write csr: enable timer interrupt
            //interrupt::enable(pac::Interrupt::TIMER);

            // write csr: enable usb1 interrupts and events
            self.enable_usb1_interrupts();
        }

        // leds: ready
        self.leds
            .output
            .write(|w| unsafe { w.output().bits(1 << 1) });

        Ok(())
    }

    #[inline(always)]
    fn main_loop(&'a mut self) -> GreatResult<()> {
        let mut max_queue_length = 0;
        let mut queue_length = 0;

        loop {
            if queue_length > max_queue_length {
                max_queue_length = queue_length;
                debug!("max_queue_length: {}", max_queue_length);
            }
            queue_length = 0;

            while let Some(message) = MESSAGE_QUEUE.dequeue() {
                queue_length += 1;

                trace!("MachineExternal: {:?}", message);
                match message {
                    // - usb1 message handlers --

                    // Usb1 received setup packet
                    Message::UsbReceiveSetupPacket(1, packet) => {
                        self.handle_usb1_receive_setup_packet(packet)?;
                    }

                    // Usb1 transfer complete
                    Message::UsbTransferComplete(1, endpoint) => {
                        self.handle_usb1_transfer_complete(endpoint)?;
                        trace!("MachineExternal - USB1_EP_IN {}\n", endpoint);
                    }

                    // Usb1 received data on control endpoint
                    Message::UsbReceiveData(1, 0, bytes_read, buffer) => {
                        self.handle_usb1_receive_control_data(bytes_read, buffer)?;
                    }

                    // Usb1 received data on endpoint
                    Message::UsbReceiveData(1, endpoint, bytes_read, buffer) => {
                        self.handle_usb1_receive_data(endpoint, bytes_read, buffer)?;
                        debug!(
                            "Usb1 received {} bytes on usb1 endpoint: {} - {:?}",
                            endpoint,
                            bytes_read,
                            &buffer[0..bytes_read]
                        );
                    }

                    // - usb0 message handlers --

                    // Usb0 received setup packet
                    Message::UsbReceiveSetupPacket(0, packet) => {
                        self.greatdancer.handle_usb_receive_setup_packet(packet)?;
                    }

                    // Usb0 transfer complete
                    Message::UsbTransferComplete(0, endpoint) => {
                        self.greatdancer.handle_usb_transfer_complete(endpoint)?;
                        trace!("MachineExternal - USB0_EP_IN {}\n", endpoint);
                    }

                    // Usb0 received data on control endpoint
                    Message::UsbReceiveData(0, 0, bytes_read, buffer) => {
                        self.greatdancer
                            .handle_usb_receive_control_data(bytes_read, buffer)?;
                    }

                    // Usb0 received data on endpoint
                    Message::UsbReceiveData(0, endpoint, bytes_read, buffer) => {
                        self.greatdancer
                            .handle_usb_receive_data(endpoint, bytes_read, buffer)?;
                        debug!(
                            "Usb0 received {} bytes on usb0 endpoint: {} - {:?}",
                            endpoint,
                            bytes_read,
                            &buffer[0..bytes_read]
                        );
                    }

                    // Error Message
                    Message::ErrorMessage(message) => {
                        error!("MachineExternal Error - {}\n", message);
                    }

                    // Timer
                    Message::TimerEvent(_n) => {}

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
                error!(" gcp: Unknown vendor request '{:?}'", vendor_request);
                self.usb1.hal_driver.write(0, [0].into_iter());
            }
            _ => match self.usb1.handle_setup_request(&setup_packet) {
                Ok(()) => (),
                Err(e) => {
                    error!("  handle_setup_request: {:?}: {:?}", e, setup_packet);
                    panic!("  handle_setup_request: {:?}: {:?}", e, setup_packet)
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
            "GCP  vendor_request: {:?} dir:{:?} value:{:?} length:{} index:{}",
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
                trace!("GCP   TODO state = Command::Begin");
                //trace!("GCP   ack {}", length);
            }

            // host is ready to receive a response
            (
                Direction::DeviceToHost,
                VendorRequest::UsbCommandRequest,
                VendorRequestValue::Start,
            ) => {
                trace!("ORDER: #3");
                trace!("GCP   TODO state = Command::Send");
                // do we have a response ready? should we wait if we don't?
                if let Some(response) = &mut self.active_response {
                    // send it
                    trace!("GCP   sending command response of {} bytes", response.len());
                    self.usb1
                        .hal_driver
                        .write(0, response.take(setup_packet.length as usize));
                    self.active_response = None;
                } else {
                    // TODO something has gone wrong
                    error!("GCP   stall: gcp response requested but no response queued");
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
                trace!("GCP   TODO state = Command::Cancel");
                debug!(
                    "GCP   TODO cancel cynthion vendor request sequence: {}",
                    length
                );
            }
            _ => {
                error!(
                    "GCP   stall: unknown vendor request and/or value: {:?} {:?} {:?}",
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
    ) -> GreatResult<()> {
        // TODO state == Command::Send

        trace!(
            "GCP   Received {} bytes on usb1 control endpoint: {:?}",
            bytes_read,
            &buffer[0..bytes_read]
        );

        if bytes_read < 8 {
            // short read
            //warn!("GCP   short read of {} bytes\n", bytes_read);
            return Ok(());
        }

        // parse & dispatch command
        if let Some(command) = gcp::Command::parse(&buffer[0..bytes_read]) {
            trace!("ORDER: #2");
            trace!("GCP   dispatching command: {:?}", command);
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
                    trace!("GCP   queueing next response");
                    self.active_response = Some(response);
                    //self.usb1.hal_driver.ep_out_prime_receive(0);
                    //self.usb1.hal_driver.write(0, [].into_iter());
                }
                Err(e) => {
                    error!("GCP   stall: failed to dispatch command {}", e);
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
        buffer: [u8; cynthion::EP_MAX_RECEIVE_LENGTH],
    ) -> GreatResult<()> {
        Ok(())
    }

    pub fn handle_usb1_transfer_complete(&mut self, endpoint: u8) -> GreatResult<()> {
        Ok(())
    }

    fn dispatch_gcp_command(
        &mut self,
        class_id: gcp::ClassId,
        verb_id: u32,
        arguments: &[u8],
        response_buffer: [u8; GCP_MAX_RESPONSE_LENGTH],
    ) -> GreatResult<GcpResponse> {
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

            _ => Err(GreatError::Message("class or verb not found")),
        }
    }
}
