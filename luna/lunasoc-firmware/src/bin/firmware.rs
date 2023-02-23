#![no_std]
#![no_main]

use firmware::{hal, pac};
use lunasoc_firmware as firmware;

use pac::csr::interrupt;

use hal::smolusb;
use smolusb::class::cynthion;
use smolusb::control::SetupPacket;
use smolusb::device::{Speed, UsbDevice};
use smolusb::traits::{
    ControlRead, EndpointRead, EndpointWrite, EndpointWriteRef, UsbDriverOperations,
};

use log::{debug, error, info, trace, warn};

use riscv_rt::entry;

// - global static state ------------------------------------------------------

use firmware::Message;
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
    firmware::log::init(serial);
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
        &cynthion::DEVICE_DESCRIPTOR,
        &cynthion::CONFIGURATION_DESCRIPTOR_0,
        &cynthion::USB_STRING_DESCRIPTOR_0,
        &cynthion::USB_STRING_DESCRIPTORS,
    );
    usb1_device.cb_vendor_request = Some(handle_vendor_request);
    usb1_device.cb_string_request = Some(handle_string_request);

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

    loop {
        while let Some(message) = MESSAGE_QUEUE.dequeue() {
            match message {
                // usb1 message handlers
                Message::Usb1ReceiveSetupPacket(packet) => {
                    match usb1_device.handle_setup_request(&packet) {
                        Ok(()) => debug!("OK\n"),
                        Err(e) => panic!("  handle_setup_request: {:?}: {:?}", e, packet),
                    }
                }
                Message::Usb1ReceiveData(endpoint, bytes_read, buffer) => {
                    if endpoint != 0 {
                        debug!(
                            "Received {} bytes on usb1 endpoint: {} - {:?}\n",
                            bytes_read, endpoint, buffer
                        );
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

fn handle_vendor_request<'a, D>(device: &UsbDevice<'a, D>, setup_packet: &SetupPacket, request: u8)
where
    D: ControlRead + EndpointRead + EndpointWrite + EndpointWriteRef + UsbDriverOperations,
{
    let request = cynthion::vendor::VendorRequest::from(request);
    debug!("  CYNTHION vendor_request: {:?}", request);
/*
    let mut buffer = [0_u8; 64];
    let bytes_read = device.hal_driver.read(0, &mut buffer);
    debug!("  read 0: {:?}", buffer);

    let mut buffer = [0_u8; 64];
    let bytes_read = device.hal_driver.read(1, &mut buffer);
    debug!("  read 1: {:?}", buffer);
*/
    // we can just spoof these for now
    //device.hal_driver.write(0, [].into_iter());
    device.hal_driver.ack_status_stage(setup_packet);
}

fn handle_string_request<'a, D>(device: &UsbDevice<'a, D>, setup_packet: &SetupPacket, index: u8)
where
    D: ControlRead + EndpointRead + EndpointWrite + EndpointWriteRef + UsbDriverOperations,
{
    debug!("  CYNTHION string_request: {}", index);

    // we can just spoof this too for now
    device.hal_driver.write(0, [].into_iter());
    device.hal_driver.ack_status_stage(setup_packet);
}
