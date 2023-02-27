#![no_std]
#![no_main]

use firmware::{hal, pac};
use lunasoc_firmware as firmware;

use pac::csr::interrupt;

use hal::smolusb;
use smolusb::class::cynthion;
use smolusb::control::{Direction, SetupPacket};
use smolusb::device::{Speed, UsbDevice};
use smolusb::traits::{
    ControlRead, EndpointRead, EndpointWrite, EndpointWriteRef, UsbDriverOperations,
};

use libgreat::gcp;
use zerocopy::FromBytes;

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
                    // TODO state == VendorRequest::UsbCommandRequest

                    if bytes_read == 0 {
                        // ignore

                    } else if bytes_read >= 8 {
                        debug!(
                            "Received {} bytes on usb1 endpoint: {} - {:?}",
                            bytes_read, endpoint, &buffer[0..bytes_read]
                        );

                        // read prelude
                        let data = &buffer[0..8];
                        if let Some(prelude) = gcp::CommandPrelude::read_from(data) {
                            info!(
                                "  COMMAND PRELUDE: {:?} => {:?}.{:?}\n",
                                prelude,
                                gcp::Class::from(prelude.class),
                                gcp::Core::from(prelude.verb),
                            );
                        } else {
                            // actually infallible
                            error!("  failed to read prelude: {:?}\n", data);
                        }

                    } else {
                        debug!(
                            "Received {} bytes on usb1 endpoint: {} - {:?}",
                            bytes_read, endpoint, &buffer[0..bytes_read]
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

fn handle_vendor_request<'a, D>(device: &UsbDevice<'a, D>, setup_packet: &SetupPacket, request: u8)
where
    D: ControlRead + EndpointRead + EndpointWrite + EndpointWriteRef + UsbDriverOperations,
{
    let request = cynthion::vendor::VendorRequest::from(request);
    debug!("  CYNTHION vendor_request: {:?} length:{} value:{} index:{}",
           request, setup_packet.length, setup_packet.value, setup_packet.index);

    // ?
    let command = setup_packet.value;
    let length = setup_packet.length as usize;

    match command {
        0x0000 => {  // command
            if setup_packet.direction() == Direction::HostToDevice {
                debug!("  ack: {}", length);
                device.hal_driver.ack_status_stage(setup_packet);

            } else if setup_packet.direction() == Direction::DeviceToHost {
                let board_id = 0x0000_u32.to_le_bytes();
                debug!("  sending board id: {:?}", board_id);
                device.hal_driver.write(0, board_id.into_iter().take(length));
                device.hal_driver.ack_status_stage(setup_packet);

            } else {
                debug!("  SHRUG");
                device.hal_driver.ack_status_stage(setup_packet);
            }
        }
        0xdead => {  // cancel ?
        }
        _ => {
            error!("  stall: unknown vendor command: {}", command);
            device.hal_driver.stall_request();
        }
    }
}
