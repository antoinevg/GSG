#![no_std]
#![no_main]

use cynthion::pac;
use pac::csr::interrupt;

use cynthion::hal;

use hal::smolusb;
use smolusb::class::cdc;
use smolusb::control::SetupPacket;
use smolusb::device::{Speed, UsbDevice};
use smolusb::traits::{
    ControlRead, EndpointRead, EndpointWrite, EndpointWriteRef, UsbDriverOperations,
};

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
    use cynthion::UsbInterface::{Aux, Control, Target};

    // peripherals
    let peripherals = unsafe { pac::Peripherals::steal() };
    let leds = &peripherals.LEDS;
    let usb0 = unsafe { hal::Usb0::summon() };
    let usb1 = unsafe { hal::Usb1::summon() };
    let usb2 = unsafe { hal::Usb2::summon() };

    // debug
    let pending = interrupt::reg_pending();
    leds.output
        .write(|w| unsafe { w.output().bits(pending as u8) });

    // - Usb0 interrupts --
    let message = if usb0.is_pending(pac::Interrupt::USB0) {
        usb0.clear_pending(pac::Interrupt::USB0);
        Message::HandleInterrupt(pac::Interrupt::USB0)
    } else if usb0.is_pending(pac::Interrupt::USB0_EP_CONTROL) {
        let mut buffer = [0_u8; 8];
        usb0.read_control(&mut buffer);
        let setup_packet = match SetupPacket::try_from(buffer) {
            Ok(packet) => packet,
            Err(e) => {
                error!("MachineExternal USB0_EP_CONTROL - {:?}", e);
                return;
            }
        };
        usb0.clear_pending(pac::Interrupt::USB0_EP_CONTROL);
        Message::UsbReceiveSetupPacket(Target, setup_packet)
    } else if usb0.is_pending(pac::Interrupt::USB0_EP_IN) {
        usb0.clear_pending(pac::Interrupt::USB0_EP_IN);
        Message::HandleInterrupt(pac::Interrupt::USB0_EP_IN)
    } else if usb0.is_pending(pac::Interrupt::USB0_EP_OUT) {
        let endpoint = usb0.ep_out.data_ep.read().bits() as u8;
        let mut buffer = [0_u8; 64];
        let bytes_read = usb0.read(endpoint, &mut buffer);
        usb0.clear_pending(pac::Interrupt::USB0_EP_OUT);
        Message::UsbReceiveData(Target, endpoint, bytes_read, buffer)

    // - Usb1 interrupts --
    } else if usb1.is_pending(pac::Interrupt::USB1) {
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
        Message::UsbReceiveSetupPacket(Aux, setup_packet)
    } else if usb1.is_pending(pac::Interrupt::USB1_EP_IN) {
        usb1.clear_pending(pac::Interrupt::USB1_EP_IN);
        Message::HandleInterrupt(pac::Interrupt::USB1_EP_IN)
    } else if usb1.is_pending(pac::Interrupt::USB1_EP_OUT) {
        let endpoint = usb1.ep_out.data_ep.read().bits() as u8;
        let mut buffer = [0_u8; 64];
        let bytes_read = usb1.read(endpoint, &mut buffer);
        usb1.clear_pending(pac::Interrupt::USB1_EP_OUT);
        Message::UsbReceiveData(Aux, endpoint, bytes_read, buffer)

    // - Usb2 interrupts --
    } else if usb2.is_pending(pac::Interrupt::USB2) {
        usb2.clear_pending(pac::Interrupt::USB2);
        Message::HandleInterrupt(pac::Interrupt::USB2)
    } else if usb2.is_pending(pac::Interrupt::USB2_EP_CONTROL) {
        let mut buffer = [0_u8; 8];
        usb2.read_control(&mut buffer);
        let setup_packet = match SetupPacket::try_from(buffer) {
            Ok(packet) => packet,
            Err(e) => {
                error!("MachineExternal USB2_EP_CONTROL - {:?}", e);
                return;
            }
        };
        usb2.clear_pending(pac::Interrupt::USB2_EP_CONTROL);
        Message::UsbReceiveSetupPacket(Control, setup_packet)
    } else if usb2.is_pending(pac::Interrupt::USB2_EP_IN) {
        usb2.clear_pending(pac::Interrupt::USB2_EP_IN);
        Message::HandleInterrupt(pac::Interrupt::USB2_EP_IN)
    } else if usb2.is_pending(pac::Interrupt::USB2_EP_OUT) {
        let endpoint = usb2.ep_out.data_ep.read().bits() as u8;
        let mut buffer = [0_u8; 64];
        let bytes_read = usb2.read(endpoint, &mut buffer);
        usb2.clear_pending(pac::Interrupt::USB2_EP_OUT);
        Message::UsbReceiveData(Control, endpoint, bytes_read, buffer)

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

    // usb0
    let usb0 = hal::Usb0::new(
        peripherals.USB0,
        peripherals.USB0_EP_CONTROL,
        peripherals.USB0_EP_IN,
        peripherals.USB0_EP_OUT,
    );
    let speed = usb0.connect();
    info!("Connected USB0 device: {:?}", Speed::from(speed));

    // usb1
    let usb1 = hal::Usb1::new(
        peripherals.USB1,
        peripherals.USB1_EP_CONTROL,
        peripherals.USB1_EP_IN,
        peripherals.USB1_EP_OUT,
    );
    let speed = usb1.connect();
    info!("Connected USB1 device: {:?}", Speed::from(speed));

    // usb2
    let usb2 = hal::Usb2::new(
        peripherals.USB2,
        peripherals.USB2_EP_CONTROL,
        peripherals.USB2_EP_IN,
        peripherals.USB2_EP_OUT,
    );
    let speed = usb2.connect();
    info!("Connected USB2 device: {:?}", Speed::from(speed));

    // enable interrupts
    usb0.enable_interrupts();
    usb1.enable_interrupts();
    usb2.enable_interrupts();
    unsafe {
        // set mstatus register: interrupt enable
        riscv::interrupt::enable();

        // set mie register: machine external interrupts enable
        riscv::register::mie::set_mext();

        // write csr: enable interrupts
        interrupt::enable(pac::Interrupt::USB0);
        interrupt::enable(pac::Interrupt::USB0_EP_CONTROL);
        interrupt::enable(pac::Interrupt::USB0_EP_IN);
        interrupt::enable(pac::Interrupt::USB0_EP_OUT);
        interrupt::enable(pac::Interrupt::USB1);
        interrupt::enable(pac::Interrupt::USB1_EP_CONTROL);
        interrupt::enable(pac::Interrupt::USB1_EP_IN);
        interrupt::enable(pac::Interrupt::USB1_EP_OUT);
        interrupt::enable(pac::Interrupt::USB2);
        interrupt::enable(pac::Interrupt::USB2_EP_CONTROL);
        interrupt::enable(pac::Interrupt::USB2_EP_IN);
        interrupt::enable(pac::Interrupt::USB2_EP_OUT);
    }

    // usb0_device
    let mut usb0_device = UsbDevice::new(
        usb0,
        &cdc::DEVICE_DESCRIPTOR,
        &cdc::CONFIGURATION_DESCRIPTOR_0,
        &cdc::USB_STRING_DESCRIPTOR_0,
        &cdc::USB_STRING_DESCRIPTORS,
    );
    usb0_device.cb_vendor_request = Some(handle_vendor_request);
    usb0_device.cb_string_request = Some(handle_string_request);

    // usb1_device
    let mut usb1_device = UsbDevice::new(
        usb1,
        &cdc::DEVICE_DESCRIPTOR,
        &cdc::CONFIGURATION_DESCRIPTOR_0,
        &cdc::USB_STRING_DESCRIPTOR_0,
        &cdc::USB_STRING_DESCRIPTORS,
    );
    usb1_device.cb_vendor_request = Some(handle_vendor_request);
    usb1_device.cb_string_request = Some(handle_string_request);

    // usb2_device
    let mut usb2_device = UsbDevice::new(
        usb2,
        &cdc::DEVICE_DESCRIPTOR,
        &cdc::CONFIGURATION_DESCRIPTOR_0,
        &cdc::USB_STRING_DESCRIPTOR_0,
        &cdc::USB_STRING_DESCRIPTORS,
    );
    usb2_device.cb_vendor_request = Some(handle_vendor_request);
    usb2_device.cb_string_request = Some(handle_string_request);

    loop {
        if let Some(message) = MESSAGE_QUEUE.dequeue() {
            use cynthion::UsbInterface::{Aux, Control, Target};

            match message {
                // usb0 message handlers
                Message::UsbReceiveSetupPacket(Target, packet) => {
                    match usb0_device.handle_setup_request(&packet) {
                        Ok(()) => debug!("OK\n"),
                        Err(e) => panic!("  handle_setup_request: {:?}: {:?}", e, packet),
                    }
                }
                Message::UsbReceiveData(Target, endpoint, bytes_read, buffer) => {
                    if endpoint != 0 {
                        debug!(
                            "Received {} bytes on usb0 endpoint: {} - {:?}\n",
                            bytes_read, endpoint, buffer
                        );
                        usb1_device
                            .hal_driver
                            .write_ref(endpoint, buffer.iter().take(bytes_read).into_iter());
                        info!("Sent {} bytes to usb1 endpoint: {}", bytes_read, endpoint);
                        usb2_device
                            .hal_driver
                            .write_ref(endpoint, buffer.iter().take(bytes_read).into_iter());
                        info!("Sent {} bytes to usb2 endpoint: {}", bytes_read, endpoint);
                    }
                }

                // usb1 message handlers
                Message::UsbReceiveSetupPacket(Aux, packet) => {
                    match usb1_device.handle_setup_request(&packet) {
                        Ok(()) => debug!("OK\n"),
                        Err(e) => panic!("  handle_setup_request: {:?}: {:?}", e, packet),
                    }
                }
                Message::UsbReceiveData(Aux, endpoint, bytes_read, buffer) => {
                    if endpoint != 0 {
                        debug!(
                            "Received {} bytes on usb1 endpoint: {} - {:?}\n",
                            bytes_read, endpoint, buffer
                        );
                        usb0_device
                            .hal_driver
                            .write_ref(endpoint, buffer.iter().take(bytes_read).into_iter());
                        info!("Sent {} bytes to usb0 endpoint: {}", bytes_read, endpoint);
                        usb2_device
                            .hal_driver
                            .write_ref(endpoint, buffer.iter().take(bytes_read).into_iter());
                        info!("Sent {} bytes to usb2 endpoint: {}", bytes_read, endpoint);
                    }
                }

                // usb2 message handlers
                Message::UsbReceiveSetupPacket(Control, packet) => {
                    match usb2_device.handle_setup_request(&packet) {
                        Ok(()) => debug!("OK\n"),
                        Err(e) => panic!("  handle_setup_request: {:?}: {:?}", e, packet),
                    }
                }
                Message::UsbReceiveData(Control, endpoint, bytes_read, buffer) => {
                    if endpoint != 0 {
                        debug!(
                            "Received {} bytes on usb2 endpoint: {} - {:?}\n",
                            bytes_read, endpoint, buffer
                        );
                        usb0_device
                            .hal_driver
                            .write_ref(endpoint, buffer.iter().take(bytes_read).into_iter());
                        info!("Sent {} bytes to usb0 endpoint: {}", bytes_read, endpoint);
                        usb1_device
                            .hal_driver
                            .write_ref(endpoint, buffer.iter().take(bytes_read).into_iter());
                        info!("Sent {} bytes to usb1 endpoint: {}", bytes_read, endpoint);
                    }
                }

                // usb0 interrupts
                Message::HandleInterrupt(pac::Interrupt::USB0) => {
                    usb0_device.reset();
                    trace!("MachineExternal - USB0\n");
                }

                // usb1 interrupts
                Message::HandleInterrupt(pac::Interrupt::USB1) => {
                    usb1_device.reset();
                    trace!("MachineExternal - USB1\n");
                }

                // usb2 interrupts
                Message::HandleInterrupt(pac::Interrupt::USB2) => {
                    usb2_device.reset();
                    trace!("MachineExternal - USB2\n");
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

fn handle_vendor_request<'a, D>(device: &UsbDevice<'a, D>, setup_packet: &SetupPacket, request: u8)
where
    D: ControlRead + EndpointRead + EndpointWrite + EndpointWriteRef + UsbDriverOperations,
{
    let request = cdc::ch34x::VendorRequest::from(request);
    debug!("  CDC-SERIAL vendor_request: {:?}", request);

    // we can just spoof these
    device.hal_driver.write(0, [0, 0].into_iter());
    device.hal_driver.ack_status_stage(setup_packet);
}

fn handle_string_request<'a, D>(device: &UsbDevice<'a, D>, setup_packet: &SetupPacket, index: u8)
where
    D: ControlRead + EndpointRead + EndpointWrite + EndpointWriteRef + UsbDriverOperations,
{
    debug!("  CDC-SERIAL string_request: {}", index);

    // we can just spoof this too
    device.hal_driver.write(0, [].into_iter());
    device.hal_driver.ack_status_stage(setup_packet);
}
