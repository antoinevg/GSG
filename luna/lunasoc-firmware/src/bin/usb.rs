#![feature(error_in_core)]
#![allow(dead_code, unused_imports, unused_variables)] // TODO
#![no_std]
#![no_main]

use panic_halt as _;
use riscv_rt::entry;

use firmware::{hal, pac};
use lunasoc_firmware as firmware;

use firmware::usb::{
    DescriptorType, Direction, Recipient, Request, RequestType, SetupPacket, UsbInterface0,
};
use firmware::{Error, Result};

use log::{debug, error, info, trace, warn};

#[entry]
fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();

    // initialize logging
    let serial = hal::Serial::new(peripherals.UART);
    firmware::log::init(serial);

    let leds = &peripherals.LEDS;
    let usb0 = UsbInterface0 {
        usb: peripherals.USB0,
        ep_setup: peripherals.USB0_SETUP,
        ep_in: peripherals.USB0_EP_IN,
        ep_out: peripherals.USB0_EP_OUT,
    };


    info!("Connecting USB device...");
    leds.output.write(|w| unsafe { w.output().bits(0x00) });
    usb0.connect();
    // 0: High, 1: Full, 2: Low, 3:SuperSpeed (incl SuperSpeed+)
    let speed = usb0.usb.speed.read().bits();
    info!("Connected: {}", speed);
    leds.output.write(|w| unsafe { w.output().bits(0x01) });

    loop {
        // read setup request
        let packet = match read_setup_request(&usb0) {
            Ok(packet) => packet,
            Err(e) => {
                error!("  Error {:?}", e);
                continue;
            }
        };

        // handle setup request
        match handle_setup_request(&usb0, &packet) {
            Ok(()) => {
                debug!("OK");
                debug!("");
            }
            Err(e) => {
                error!("  Error {:?}: {:?}", e, packet);
                leds.output.write(|w| unsafe { w.output().bits(128) });
                loop {}
            }
        };
    }
}

// - read_setup_request -------------------------------------------------------

fn read_setup_request(usb0: &UsbInterface0) -> Result<SetupPacket> {
    debug!("# read_setup_request()");

    // read data packet
    let mut buffer = [0_u8; 8];
    usb0.ep_setup_read_packet(&mut buffer)?;

    // Deserialize into a SetupRequest in the most cursed manner available to us
    let packet = unsafe { core::mem::transmute::<[u8; 8], SetupPacket>(buffer) };

    Ok(packet)
}

// - handle_setup_request -----------------------------------------------------

fn handle_setup_request(usb0: &UsbInterface0, packet: &SetupPacket) -> Result<()> {
    debug!("# handle_setup_request()",);

    // extract the recipient (e.g. device/interface/endpoint)
    let bits: u8 = packet.request_type & 0b0001_1111;
    let recipient = match Recipient::try_from(bits) {
        Ok(recipient) => recipient,
        Err(e) => {
            warn!("  stall: invalid recipient: {}", bits);
            usb0.stall_request();
            return Ok(());
        }
    };

    // extract the request type (e.g. standard/class/vendor) from our SETUP request.
    let bits: u8 = (packet.request_type >> 5) & 0b0000_0011;
    let request_type = match RequestType::try_from(bits) {
        Ok(request_type) => request_type,
        Err(e) => {
            warn!("  stall: invalid request type: {}", bits);
            usb0.stall_request();
            return Ok(());
        }
    };

    // extract the direction
    let bits: u8 = (packet.request_type >> 7) & 0b0000_0001;
    let direction = match Direction::try_from(bits) {
        Ok(direction) => direction,
        Err(e) => {
            warn!("  stall: invalid direction: {}", bits);
            usb0.stall_request();
            return Ok(());
        }
    };

    // TODO: Get rid of this once we move to be fully compatible with ValentyUSB.
    usb0.ep_in.pid.write(|w| w.pid().bit(true));

    // if this isn't a standard request, stall it.
    if request_type != RequestType::Standard {
        warn!("  stall: unsupported request type {:?}", request_type);
        usb0.stall_request();
        return Ok(());
    }

    // Extract the request
    let request = match Request::try_from(packet.request) {
        Ok(request) => request,
        Err(e) => {
            warn!("  stall: invalid request: {}", packet.request);
            usb0.stall_request();
            return Ok(());
        }
    };

    debug!("  dispatch: {:?} {:?} {:?} {}, {}", recipient, direction, request, packet.value, packet.length);

    match request {
        Request::SetAddress => handle_set_address(usb0, packet),
        Request::GetStatus => handle_get_status(usb0, packet),
        Request::SetDescriptor => handle_set_descriptor(usb0, packet),
        Request::GetDescriptor => handle_get_descriptor(usb0, packet),
        Request::SetConfiguration => handle_set_configuration(usb0, packet),
        _ => {
            warn!("  stall: unhandled request {:?}", request);
            usb0.stall_request();
            Ok(())
        }
    }
}

fn handle_set_address(usb0: &UsbInterface0, packet: &SetupPacket) -> Result<()> {
    usb0.ack_status_stage(packet);

    // TODO: SetupRequest.value is u16 but register expects u8 - is this correct?
    let address: u8 = packet.value.try_into()?;
    usb0.ep_setup
        .address
        .write(|w| unsafe { w.address().bits(address) });

    debug!("  -> handle_set_address({})", address);

    Ok(())
}

/// ???
fn handle_get_status(usb0: &UsbInterface0, packet: &SetupPacket) -> Result<()> {
    debug!("  -> handle_get_status()");

    usb0.ack_status_stage(packet);

    Ok(())
}

/// ???
fn handle_set_descriptor(usb0: &UsbInterface0, packet: &SetupPacket) -> Result<()> {
    debug!("  -> handle_set_descriptor()");

    let descriptor = packet.value;
    if descriptor > 1 {
        warn!("  stall: unknown descriptor {}", descriptor);
        usb0.stall_request();
        return Ok(());
    }

    usb0.ack_status_stage(packet);

    Ok(())
}

fn handle_get_descriptor(usb0: &UsbInterface0, packet: &SetupPacket) -> Result<()> {
    // extract the descriptor type and number from our SETUP request
    let [descriptor_number, descriptor_type_bits] = packet.value.to_le_bytes();
    let descriptor_type = match DescriptorType::try_from(descriptor_type_bits) {
        Ok(descriptor_type) => descriptor_type,
        Err(e) => {
            warn!("  stall: invalid descriptor type: {} {}", descriptor_type_bits, descriptor_number);
            usb0.stall_request();
            return Ok(());
        }
    };

    match (&descriptor_type, descriptor_number) {
        (DescriptorType::Device, _) => {
            usb0.ep_in_send_control_response(packet, USB_DEVICE_DESCRIPTOR)
        }
        (DescriptorType::Configuration, 0) => {
            usb0.ep_in_send_control_response(packet, USB_CONFIG_DESCRIPTOR)
        }
        (DescriptorType::String, 0) => {
            usb0.ep_in_send_control_response(packet, USB_STRING0_DESCRIPTOR)
        }
        (DescriptorType::String, 1) => {
            usb0.ep_in_send_control_response(packet, USB_STRING1_DESCRIPTOR)
        }
        (DescriptorType::String, 2) => {
            usb0.ep_in_send_control_response(packet, USB_STRING2_DESCRIPTOR)
        }
        _ => {
            warn!(
                "  stall: unhandled descriptor {:?}, {}",
                descriptor_type, descriptor_number
            );
            usb0.stall_request();
            return Ok(());
        }
    }

    usb0.ack_status_stage(packet);

    debug!("  -> handle_get_descriptor({:?}({}), {})", descriptor_type, descriptor_type_bits, descriptor_number);

    Ok(())
}

fn handle_set_configuration(usb0: &UsbInterface0, packet: &SetupPacket) -> Result<()> {
    debug!("  -> handle_set_configuration()");

    let configuration = packet.value;
    if configuration > 1 {
        warn!("  stall: unknown configuration {}", configuration);
        usb0.stall_request();
        return Ok(());
    }

    usb0.ack_status_stage(packet);

    Ok(())
}

// - usb constants ------------------------------------------------------------

const _USB_DEVICE_DESCRIPTOR: &[u8] = &[
    0x12, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x40,
    0xd0, 0x16, 0x3b, 0x0f, 0x01, 0x01, 0x01, 0x02,
    0x00, 0x01
];

const USB_DEVICE_DESCRIPTOR: &[u8] = &[
    0x12,       // DEVICE DESCRIPTOR
    0x01,       // DescriptorType = DEVICE
    0x00, 0x02, // bcdUSB = 0x0200
    0x00,       // DeviceClass
    0x00,       // DeviceSubClass
    0x00,       // DeviceProtocol
    0x40,       // MaxPacketSize = 64

    0xd0, 0x16, // idVendor
    0x3b, 0x0f, // idProduct
    0x00, 0x00, // bcdDevice
    0x01,       // iManufacturer
    0x02,       // iProduct

    0x03,       // iSerialNumber
    0x01,       // bNumConfigurations
];

const USB_CONFIG_DESCRIPTOR: &[u8] = &[
    0x09, 0x02, 0x12, 0x00, 0x01, 0x01, 0x01, 0x80, 0x32, 0x09, 0x04, 0x00, 0x00, 0x00, 0xfe, 0x00,
    0x00, 0x02,
];

const USB_STRING0_DESCRIPTOR: &[u8] = &[0x04, 0x03, 0x09, 0x04];

const USB_STRING1_DESCRIPTOR: &[u8] = &[0x0a, 0x03, b'L', 0x00, b'U', 0x00, b'N', 0x00, b'A', 0x00];

const USB_STRING2_DESCRIPTOR: &[u8] = &[
    0x22, 0x03, b'T', 0, b'r', 0, b'i', 0, b'-', 0, b'F', 0, b'I', 0, b'F', 0, b'O', 0, b' ', 0,
    b'E', 0, b'x', 0, b'a', 0, b'm', 0, b'p', 0, b'l', 0, b'e', 0,
];
