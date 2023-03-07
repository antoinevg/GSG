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
    let leds = &peripherals.LEDS;
    let usb1 = unsafe { hal::Usb1::summon() };

    // debug
    let pending = interrupt::reg_pending();
    leds.output
        .write(|w| unsafe { w.output().bits(pending as u8) });

    // - usb1 interrupts - "host_phy" --
    let message = if usb1.is_pending(pac::Interrupt::USB1) {
        usb1.clear_pending(pac::Interrupt::USB1);
        Message::HandleInterrupt(pac::Interrupt::USB1)
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
    } else if usb1.is_pending(pac::Interrupt::USB1_EP_IN) {
        usb1.clear_pending(pac::Interrupt::USB1_EP_IN);
        Message::HandleInterrupt(pac::Interrupt::USB1_EP_IN)
    } else if usb1.is_pending(pac::Interrupt::USB1_EP_OUT) {
        let endpoint = usb1.ep_out.data_ep.read().bits() as u8;
        let mut buffer = [0_u8; 64];
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
    let peripherals = pac::Peripherals::take().unwrap();
    let leds = &peripherals.LEDS;
    leds.output.write(|w| unsafe { w.output().bits(0x0) });

    // initialize logging
    let serial = hal::Serial::new(peripherals.UART);
    cynthion::log::init(serial);
    info!("logging initialized");

    // usb1 - "host_phy"
    let usb1 = hal::Usb1::new(
        peripherals.USB1,
        peripherals.USB1_EP_CONTROL,
        peripherals.USB1_EP_IN,
        peripherals.USB1_EP_OUT,
    );
    let speed = usb1.connect();
    info!("Connected usb1 device: {:?}", Speed::from(speed));

    // usb1_device
    let mut usb1_device = UsbDevice::new(
        &usb1,
        &class::cynthion::DEVICE_DESCRIPTOR,
        &class::cynthion::CONFIGURATION_DESCRIPTOR_0,
        &class::cynthion::USB_STRING_DESCRIPTOR_0,
        &class::cynthion::USB_STRING_DESCRIPTORS,
    );
    //usb1_device.cb_vendor_request = Some(handle_vendor_request);

    // interrupts
    usb1.enable_interrupts();
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
    let supported_classes = [class_core, class_firmware];
    let classes = gcp::Classes(&supported_classes);

    let some_state: u32 = 45;
    let mut next_response: Option<&[u8]> = None;

    loop {
        if let Some(message) = MESSAGE_QUEUE.dequeue() {
            match message {
                // usb1 message handlers
                Message::Usb1ReceiveSetupPacket(packet) => {
                    let request_type = packet.request_type();
                    let vendor_request = VendorRequest::from(packet.request);

                    match (&request_type, &vendor_request) {
                        // TODO add a Any parametr to handle_setup_packet, or just catch vendor requests here?
                        (RequestType::Vendor, VendorRequest::UsbCommandRequest) => {
                            if handle_vendor_request(&usb1_device, &packet, packet.request) {
                                // do we have a response ready? should we wait if we don't?
                                if let Some(response) = next_response {
                                    // send it
                                    debug!("  sending gcp response: {:?}", response);
                                    usb1_device.hal_driver.write_ref(
                                        0,
                                        response.into_iter().take(packet.length as usize),
                                    );
                                    usb1_device.hal_driver.ack_status_stage(&packet);
                                } else {
                                    // TODO something has gone wrong
                                    error!(
                                        "  stall: gcp response requested but no response queued"
                                    );
                                    usb1_device.hal_driver.stall_request();
                                }
                            }
                        }
                        _ => match usb1_device.handle_setup_request(&packet) {
                            Ok(()) => debug!("OK\n"),
                            Err(e) => panic!("  handle_setup_request: {:?}: {:?}", e, packet),
                        },
                    }
                }
                Message::Usb1ReceiveData(endpoint, bytes_read, buffer) => {
                    // TODO endpoint == 0 && state == Command::Send

                    if bytes_read == 0 {
                        // ignore
                    } else if bytes_read >= 8 {
                        debug!(
                            "Received {} bytes on usb1 endpoint: {} - {:?}",
                            bytes_read,
                            endpoint,
                            &buffer[0..bytes_read]
                        );

                        // read & dispatch command prelude
                        let data = &buffer[0..8];
                        if let Some(command) = gcp::Command::parse(data) {
                            info!("  COMMAND: {:?}", command);
                            // TODO we really need a better way to get this to the vendor request
                            //let reply = gcp_class_dispatch.dispatch(command, &some_state);
                            //next_response = Some(reply.as_slice());
                            match classes.dispatch(command, &some_state) {
                                Ok(response) => next_response = Some(response.as_slice()),
                                Err(e) => {
                                    error!("  failed to dispatch command: {}", e);
                                }
                            }
                            //debug!("  sending gcp response: {:?}", response);
                            //usb1_device.hal_driver.write_ref(0, data.into_iter());
                            //usb1_device.hal_driver.write_ref(0, [].into_iter());
                            //usb1_device.hal_driver.ack_status_stage(&packet);
                            info!("\n");
                        } else {
                            // actually infallible
                            error!("  failed to read prelude: {:?}\n", data);
                        }
                    } else {
                        debug!(
                            "Received {} bytes on usb1 endpoint: {} - {:?}",
                            bytes_read,
                            endpoint,
                            &buffer[0..bytes_read]
                        );
                        error!("  short read: {} bytes\n", bytes_read);
                    }
                }

                // usb1 interrupts
                Message::HandleInterrupt(pac::Interrupt::USB1) => {
                    usb1_device.reset();
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
}

// - vendor request handlers --------------------------------------------------

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
        "  CYNTHION vendor_request: {:?} dir:{:?} value:{:?} length:{} index:{}",
        request, direction, request_value, length, setup_packet.index
    );

    match (&direction, &request, &request_value) {
        // host is starting a new command sequence
        (Direction::HostToDevice, VendorRequest::UsbCommandRequest, VendorRequestValue::Start) => {
            device.hal_driver.ack_status_stage(setup_packet);
            debug!("  TODO state = Command::Begin");
            debug!("  ack: {}", length);
        }

        // host is ready to receive a response
        (Direction::DeviceToHost, VendorRequest::UsbCommandRequest, VendorRequestValue::Start) => {
            debug!("  TODO state = Command::Send");
            return true;
        }

        // host would like to abort the current command sequence
        (Direction::HostToDevice, VendorRequest::UsbCommandRequest, VendorRequestValue::Cancel) => {
            // TODO cancel
            debug!("  TODO state = Command::Cancel");
            debug!("  TODO cancel cynthion vendor request sequence");
        }
        _ => {
            error!(
                "  stall: unknown vendor request and/or value: {:?} {:?} {:?}",
                direction, request, request_value
            );
            device.hal_driver.stall_request();
        }
    }

    false
}
