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

use libgreat::gcp;

use log::{debug, error, info, trace, warn};
use riscv_rt::entry;

use core::any::Any;
use core::array;
use core::iter;
use core::slice;

// - global constants ---------------------------------------------------------

// TODO get rid of this
pub const GCP_MAX_RESPONSE_LENGTH: usize = 32;

// - global static state ------------------------------------------------------

use cynthion::Message;
use heapless::mpmc::MpMcQueue as Queue;
static MESSAGE_QUEUE: Queue<Message, 128> = Queue::new();

// - MachineExternal interrupt handler ----------------------------------------

#[allow(non_snake_case)]
#[no_mangle]
fn MachineExternal() {
    // peripherals
    let peripherals = unsafe { pac::Peripherals::steal() };
    let _leds = &peripherals.LEDS;
    let usb1 = unsafe { hal::Usb1::summon() };

    let pending = interrupt::reg_pending();

    // leds: debug
    //_leds.output
    //    .write(|w| unsafe { w.output().bits(pending as u8) });

    // - usb1 interrupts - "host_phy" --

    // USB1 HandleInterrupt
    let message = if usb1.is_pending(pac::Interrupt::USB1) {
        usb1.clear_pending(pac::Interrupt::USB1);
        Message::HandleInterrupt(pac::Interrupt::USB1)

    // USB1_EP_CONTROL Usb1ReceiveSetupPacket
    } else if usb1.is_pending(pac::Interrupt::USB1_EP_CONTROL) {
        let mut buffer = [0_u8; 8];
        usb1.read_control(&mut buffer);
        let setup_packet = match SetupPacket::try_from(buffer) {
            Ok(packet) => packet,
            Err(e) => {
                error!("MachineExternal USB1_EP_CONTROL - {:?}", e);
                return;
            }
        };
        usb1.clear_pending(pac::Interrupt::USB1_EP_CONTROL);
        Message::Usb1ReceiveSetupPacket(setup_packet)

    // USB1_EP_IN transfer complete
    } else if usb1.is_pending(pac::Interrupt::USB1_EP_IN) {
        usb1.clear_pending(pac::Interrupt::USB1_EP_IN);
        Message::HandleInterrupt(pac::Interrupt::USB1_EP_IN)

    // USB1_EP_OUT Usb1ReceiveData
    } else if usb1.is_pending(pac::Interrupt::USB1_EP_OUT) {
        let endpoint = usb1.ep_out.data_ep.read().bits() as u8;
        /*if endpoint == 0 {
            // skip control endpoint, we're going to try read it in
            // the vendor request handler instead.
            debug!("MachineExternal - skipping USB1_EP_OUT(0)");
            usb1.clear_pending(pac::Interrupt::USB1_EP_OUT);
            return;
        }*/
        let mut buffer = [0_u8; cynthion::EP_MAX_RECEIVE_LENGTH];
        let bytes_read = usb1.read(endpoint, &mut buffer);
        usb1.clear_pending(pac::Interrupt::USB1_EP_OUT);
        Message::Usb1ReceiveData(endpoint, bytes_read, buffer)

    // - Unknown Interrupt --
    } else {
        Message::HandleUnknownInterrupt(pending)
    };

    MESSAGE_QUEUE
        .enqueue(message)
        .expect("MachineExternal - message queue overflow")
}

// - main entry point ---------------------------------------------------------

#[entry]
fn main() -> ! {
    // initialize class registry
    let verbs_core = gcp::class_core::verbs();
    let class_core = gcp::Class {
        id: gcp::ClassId::core,
        verbs: &verbs_core,
    };
    let verbs_firmware = cynthion::class::firmware::verbs();
    let class_firmware = gcp::Class {
        id: gcp::ClassId::firmware,
        verbs: &verbs_firmware,
    };
    let classes = [class_core, class_firmware];

    // initialize firmware
    let mut firmware = Firmware::new(pac::Peripherals::take().unwrap(), gcp::Classes(&classes));
    match firmware.initialize() {
        Ok(()) => (),
        Err(e) => panic!("Firmware panicked during initialization: {}", e),
    }

    // enter main loop
    match firmware.main_loop() {
        Ok(()) => panic!("Firmware exited unexpectedly in main loop"),
        Err(e) => panic!("Firmware panicked in main loop: {}", e),
    }
}

// - Firmware -----------------------------------------------------------------

type GcpResponse = iter::Take<array::IntoIter<u8, GCP_MAX_RESPONSE_LENGTH>>;

struct Firmware<'a> {
    // peripherals
    leds: pac::LEDS,
    usb1: UsbDevice<'a, hal::Usb1>,

    // state
    classes: gcp::Classes<'a>,
    active_response: Option<GcpResponse>,
}

impl<'a> Firmware<'a> {
    fn new(peripherals: pac::Peripherals, classes: gcp::Classes<'a>) -> Self {
        // initialize logging
        cynthion::log::init(hal::Serial::new(peripherals.UART));
        trace!("logging initialized");

        Self {
            leds: peripherals.LEDS,
            usb1: UsbDevice::new(
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
            ),
            classes,
            active_response: None,
        }
    }

    fn initialize(&mut self) -> cynthion::Result<()> {
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
    fn main_loop(&'a mut self) -> cynthion::Result<()> {
        // leds: main loop
        self.leds
            .output
            .write(|w| unsafe { w.output().bits(1 << 0) });

        loop {
            while let Some(message) = MESSAGE_QUEUE.dequeue() {
                match message {
                    // usb1 message handlers
                    Message::Usb1ReceiveSetupPacket(packet) => {
                        let request_type = packet.request_type();
                        let vendor_request = VendorRequest::from(packet.request);
                        let vendor_request_value = VendorRequestValue::from(packet.value);

                        match (&request_type, &vendor_request) {
                            // TODO add a Any parameter to handle_setup_packet,
                            // or just catch vendor requests here?
                            (RequestType::Vendor, VendorRequest::UsbCommandRequest) => {
                                // TODO we should pass active_response to handle_vendor_request
                                if vendor_request_value == VendorRequestValue::Cancel {
                                    self.active_response = None;
                                }

                                if handle_vendor_request(&self.usb1, &packet, packet.request) {
                                    warn!("ORDER: #4");
                                    // do we have a response ready? should we wait if we don't?
                                    if let Some(response) = &mut self.active_response {
                                        // send it
                                        debug!(
                                            "  gcp: sending command response of {} bytes: {:?}",
                                            packet.length, response
                                        );
                                        self.usb1
                                            .hal_driver
                                            .write(0, response.take(packet.length as usize));
                                        // TODO do we need to ACK?
                                        self.usb1.hal_driver.ack_status_stage(&packet);
                                        self.active_response = None;
                                    } else {
                                        // TODO something has gone wrong
                                        error!(
                                            "  gcp: stall: gcp response requested but no response queued"
                                        );
                                        self.usb1.hal_driver.stall_request();
                                    }
                                    warn!("ORDER: fin");
                                }
                            }
                            _ => match self.usb1.handle_setup_request(&packet) {
                                Ok(()) => debug!("OK\n"),
                                Err(e) => panic!("  handle_setup_request: {:?}: {:?}", e, packet),
                            },
                        }
                    }

                    // received data on control endpoint
                    Message::Usb1ReceiveData(0, bytes_read, buffer) => {
                        // TODO state == Command::Send

                        debug!(
                            "  gcp: Received {} bytes on usb1 control endpoint: {:?}",
                            bytes_read,
                            &buffer[0..bytes_read]
                        );

                        if bytes_read < 8 {
                            // short read
                            warn!("  gcp: short read of {} bytes\n", bytes_read);
                            continue;
                        }

                        // parse & dispatch command
                        if let Some(command) = gcp::Command::parse(&buffer[0..8]) {
                            warn!("ORDER: #2");
                            debug!("  gcp: dispatching command: {:?}", command);
                            // let response = self.classes.dispatch(command, &self.some_state);
                            let response = giga_dispatch(
                                command.class_id(),
                                command.verb_id(),
                                command.arguments,
                                &self.classes,
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
                    }

                    // received data on endpoint
                    Message::Usb1ReceiveData(endpoint, bytes_read, buffer) => {
                        debug!(
                            "Received {} bytes on usb1 endpoint: {} - {:?}",
                            endpoint,
                            bytes_read,
                            &buffer[0..bytes_read]
                        );
                    }

                    // usb1 interrupts
                    Message::HandleInterrupt(pac::Interrupt::USB1) => {
                        self.usb1.reset();
                        trace!("MachineExternal - USB1\n");
                    }
                    Message::HandleInterrupt(pac::Interrupt::USB1_EP_IN) => {
                        // TODO
                        trace!("MachineExternal - USB1_EP_IN\n");
                    }

                    // unhandled
                    _ => {
                        warn!("Unhandled message: {:?}\n", message);
                    }
                }
            }
        }

        #[allow(unreachable_code)] // TODO
        Ok(())
    }
}

// - gigantic manual dispatch -------------------------------------------------

fn giga_dispatch<'a, 'b>(
    class_id: gcp::ClassId,
    verb_id: u32,
    arguments: &'a [u8],
    classes: &'b gcp::Classes,
) -> cynthion::Result<iter::Take<array::IntoIter<u8, GCP_MAX_RESPONSE_LENGTH>>> {
    static CLASSES: [u32; 2] = [
        gcp::ClassId::core.into_u32(),
        gcp::ClassId::firmware.into_u32(),
    ];

    match (class_id, verb_id) {
        (gcp::ClassId::core, 0x0) => {
            // read_board_id
            let iter = gcp::class_core::man_read_board_id(&cynthion::BOARD_INFORMATION);
            let response = unsafe { iter_to_response(iter) };
            Ok(response)
        }
        (gcp::ClassId::core, 0x1) => {
            // read_version_string
            let iter = cynthion::BOARD_INFORMATION
                .version_string
                .as_bytes()
                .into_iter();
            let response = unsafe { iter_ref_to_response(iter) };
            Ok(response)
        }
        (gcp::ClassId::core, 0x2) => {
            // read_part_id
            let iter = cynthion::BOARD_INFORMATION.part_id.into_iter();
            let response = unsafe { iter_to_response(iter) };
            Ok(response)
        }
        (gcp::ClassId::core, 0x3) => {
            // read_serial_number
            let iter = cynthion::BOARD_INFORMATION.serial_number.into_iter();
            let response = unsafe { iter_to_response(iter) };
            Ok(response)
        }
        (gcp::ClassId::core, 0x4) => {
            // get_available_classes
            //unimplemented!();
            /*let classes = [
                gcp::ClassId::core.into_u32(),
                gcp::ClassId::firmware.into_u32(),
            ];
            let response = classes.iter().flat_map(|class| class.to_le_bytes());
            Ok(response)*/

            //let iter = CLASSES.iter().flat_map(|class| class.to_le_bytes());
            let iter = [].into_iter();
            let response = unsafe { iter_to_response(iter) };
            Ok(response)
        }
        _ => Err(&libgreat::error::GreatError::NotFound(
            "class or verb not found",
        )),
    }
}

// TODO an ugly hack to tide us over while I try abstain from digging a deeper hole
unsafe fn iter_to_response(
    iter: impl Iterator<Item = u8>,
) -> iter::Take<array::IntoIter<u8, GCP_MAX_RESPONSE_LENGTH>> {
    let mut response: [u8; GCP_MAX_RESPONSE_LENGTH] = [0; GCP_MAX_RESPONSE_LENGTH];
    let mut length = 0;
    for (ret, src) in response.iter_mut().zip(iter) {
        *ret = src;
        length += 1;
    }
    response.into_iter().take(length)
}

unsafe fn iter_ref_to_response<'a>(
    iter: impl Iterator<Item = &'a u8>,
) -> iter::Take<array::IntoIter<u8, GCP_MAX_RESPONSE_LENGTH>> {
    let mut response: [u8; GCP_MAX_RESPONSE_LENGTH] = [0; GCP_MAX_RESPONSE_LENGTH];
    let mut length = 0;
    for (ret, src) in response.iter_mut().zip(iter) {
        *ret = *src;
        length += 1;
    }
    response.into_iter().take(length)
}

// - gcp vendor request handler -----------------------------------------------

fn handle_vendor_request<'a, D>(
    device: &UsbDevice<'a, D>,
    setup_packet: &SetupPacket,
    request: u8,
) -> bool
where
    D: ControlRead + EndpointRead + EndpointWrite + EndpointWriteRef + UsbDriverOperations,
{
    let direction = setup_packet.direction();
    let request = VendorRequest::from(request); // aka setup_packet.request
    let request_value = VendorRequestValue::from(setup_packet.value);
    let length = setup_packet.length as usize;

    debug!(
        "  gcp: CYNTHION vendor_request: {:?} dir:{:?} value:{:?} length:{} index:{}",
        request, direction, request_value, length, setup_packet.index
    );

    match (&direction, &request, &request_value) {
        // host is starting a new command sequence
        (Direction::HostToDevice, VendorRequest::UsbCommandRequest, VendorRequestValue::Start) => {
            device.hal_driver.ack_status_stage(setup_packet);
            warn!("ORDER: #1");
            debug!("  gcp: TODO state = Command::Begin");
            debug!("  gcp: ack {}", length);
        }

        // host is ready to receive a response
        (Direction::DeviceToHost, VendorRequest::UsbCommandRequest, VendorRequestValue::Start) => {
            warn!("ORDER: #3");
            debug!("  gcp: TODO state = Command::Send");
            return true;
        }

        // host would like to abort the current command sequence
        (Direction::DeviceToHost, VendorRequest::UsbCommandRequest, VendorRequestValue::Cancel) => {
            // TODO cancel
            debug!("  gcp: TODO state = Command::Cancel");
            debug!(
                "  gcp: TODO cancel cynthion vendor request sequence: {}",
                length
            );
            // TODO - how long? ack?
            device.hal_driver.write(0, [0xde, 0xad, 0xde, 0xad].into_iter());
            device.hal_driver.stall_request();
        }
        _ => {
            error!(
                "  gcp: stall: unknown vendor request and/or value: {:?} {:?} {:?}",
                direction, request, request_value
            );
            device.hal_driver.stall_request();
        }
    }

    false
}
