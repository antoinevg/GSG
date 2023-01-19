#![feature(error_in_core)]
#![allow(dead_code, unused_imports, unused_variables)] // TODO
#![no_std]
#![no_main]

use panic_halt as _;
use riscv_rt::entry;

//use hal::hal::delay::DelayUs;

use firmware::{hal, pac};
use lunasoc_firmware as firmware;

use firmware::usb::{
    Descriptor, Direction, Recipient, Request, RequestType, SetupPacket, UsbInterface0,
};
//use firmware::SYSTEM_CLOCK_FREQUENCY;
use firmware::{Error, Result};

use log::{debug, error, info, warn};

#[entry]
fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();

    // initialize logging
    let serial = hal::Serial::new(peripherals.UART);
    firmware::log::init(serial);

    let leds = &peripherals.LEDS;
    //let mut timer = hal::Timer::new(peripherals.TIMER, SYSTEM_CLOCK_FREQUENCY);

    let usb0 = UsbInterface0 {
        usb: peripherals.USB0,
        setup: peripherals.USB0_SETUP,
        ep_in: peripherals.USB0_EP_IN,
        ep_out: peripherals.USB0_EP_OUT,
    };

    leds.output.write(|w| unsafe { w.output().bits(0xff) });
    //timer.delay_ms(2000).unwrap();

    info!("Connecting USB device...");
    usb0.usb.connect.write(|w| w.connect().bit(true));
    info!("Connected.");

    loop {
        leds.output.write(|w| unsafe { w.output().bits(0x00) });
        // read setup request
        let setup_request = match read_setup_request(&usb0) {
            Ok(setup_request) => setup_request,
            Err(e) => {
                error!("  Error {:?}", e);
                continue;
            }
        };

        // handle setup request
        match handle_setup_request(&usb0, &setup_request) {
            Ok(()) => {
                debug!("  Ok: {:?}", setup_request);
            }
            Err(e) => {
                error!("  Error {:?}: {:?}", e, setup_request);
                loop {}
            }
        };
        leds.output.write(|w| unsafe { w.output().bits(0xff) });
    }
}

// - read_setup_request -------------------------------------------------------

fn read_setup_request(usb0: &UsbInterface0) -> Result<SetupPacket> {
    debug!("# read_setup_request()");

    let mut counter = 0;
    let mut buf = [0_u8; 8];
    for i in 0..7 {
        // block until setup data is available
        while !usb0.setup.have.read().have().bit() {
            counter += 1;
            if counter > 60_000_000 {
                return Err(Error::Timeout);
            }
        }

        // read next byte
        buf[i] = usb0.setup.data.read().data().bits();
    }

    // Deserialize into a SetupRequest in the most cursed manner available to us.
    let setup_request = unsafe { core::mem::transmute::<[u8; 8], SetupPacket>(buf) };

    Ok(setup_request)
}

// - handle_setup_request -----------------------------------------------------

fn handle_setup_request(usb0: &UsbInterface0, setup_request: &SetupPacket) -> Result<()> {
    debug!("# handle_setup_request()",);

    // Extract the recipient (e.g. device/interface/endpoint)
    let bits: u8 = setup_request.request_type & 0b0001_1111;
    let recipient = match Recipient::try_from(bits) {
        Ok(recipient) => recipient,
        Err(e) => {
            warn!("  stall: unknown recipient: {}", bits);
            usb0.stall_request();
            return Ok(());
        }
    };

    // Extract the request type (e.g. standard/class/vendor) from our SETUP request.
    let bits: u8 = (setup_request.request_type >> 5) & 0b0000_0011;
    let request_type = match RequestType::try_from(bits) {
        Ok(request_type) => request_type,
        Err(e) => {
            warn!("  stall: unknown request type: {}", bits);
            usb0.stall_request();
            return Ok(());
        }
    };

    // Extract the direction
    let bits: u8 = (setup_request.request_type >> 7) & 0b0000_0001;
    let direction = match Direction::try_from(bits) {
        Ok(direction) => direction,
        Err(e) => {
            warn!("  stall: unknown direction: {}", bits);
            usb0.stall_request();
            return Ok(());
        }
    };

    // TODO: Get rid of this once we move to be fully compatible with ValentyUSB.
    usb0.ep_in.pid.write(|w| w.pid().bit(true));

    // If this isn't a standard request, stall it.
    if request_type != RequestType::Standard {
        warn!("  stall: not standard");
        usb0.stall_request();
        return Ok(());
    }

    // Extract the request
    let request = match Request::try_from(setup_request.request) {
        Ok(request) => request,
        Err(e) => {
            warn!("  stall: unknown request: {}", setup_request.request);
            usb0.stall_request();
            return Ok(());
        }
    };

    debug!("  dispatch: {:?} {:?} {:?}", recipient, direction, request);

    match request {
        Request::SetAddress => handle_set_address(usb0, setup_request),
        Request::GetStatus => handle_get_status(usb0, setup_request),
        Request::SetDescriptor => handle_set_descriptor(usb0, setup_request),
        Request::GetDescriptor => handle_get_descriptor(usb0, setup_request),
        Request::SetConfiguration => handle_set_configuration(usb0, setup_request),
        _ => {
            warn!("  stall: unhandled request {:?}", request);
            usb0.stall_request();
            Ok(())
        }
    }
}

fn handle_set_address(usb0: &UsbInterface0, setup_request: &SetupPacket) -> Result<()> {
    debug!("  -> handle_set_address()");

    usb0.ack_status_stage(setup_request);

    // TODO: SetupRequest.value is u16 but register expects u8 - is this correct?
    let address: u8 = setup_request.value.try_into()?;

    usb0.setup
        .address
        .write(|w| unsafe { w.address().bits(address) });

    Ok(())
}

/// ???
fn handle_get_status(usb0: &UsbInterface0, setup_request: &SetupPacket) -> Result<()> {
    debug!("  -> handle_get_status()");

    usb0.ack_status_stage(setup_request);

    //let status: u16 = 0x01; // 0b01 = self powered, 0b10 = remote wakeup enabled
    //usb0.send_packet_control_response(setup_request, &status.to_le_bytes());
    //usb0.send_packet(0, &status.to_le_bytes());

    Ok(())
}

/// ???
fn handle_set_descriptor(usb0: &UsbInterface0, setup_request: &SetupPacket) -> Result<()> {
    debug!("  -> handle_set_descriptor()");

    usb0.ack_status_stage(setup_request);

    Ok(())
}

fn handle_get_descriptor(usb0: &UsbInterface0, setup_request: &SetupPacket) -> Result<()> {
    debug!("  -> handle_get_descriptor()");

    let [descriptor_number, descriptor_type] = setup_request.value.to_le_bytes();
    let descriptor_type = Descriptor::try_from(descriptor_type)?;

    match (descriptor_number, &descriptor_type) {
        (_, Descriptor::Device) => {
            usb0.send_packet_control_response(setup_request, USB_DEVICE_DESCRIPTOR)
        }
        (0, Descriptor::Configuration) => {
            usb0.send_packet_control_response(setup_request, USB_CONFIG_DESCRIPTOR)
        }
        (0, Descriptor::String) => {
            usb0.send_packet_control_response(setup_request, USB_STRING0_DESCRIPTOR)
        }
        (1, Descriptor::String) => {
            usb0.send_packet_control_response(setup_request, USB_STRING1_DESCRIPTOR)
        }
        (2, Descriptor::String) => {
            usb0.send_packet_control_response(setup_request, USB_STRING2_DESCRIPTOR)
        }
        _ => {
            warn!(
                "  stall: unhandled descrptor {}.{:?}",
                descriptor_number, descriptor_type
            );
            usb0.stall_request();
            return Ok(());
        }
    }

    Ok(())
}

fn handle_set_configuration(usb0: &UsbInterface0, setup_request: &SetupPacket) -> Result<()> {
    debug!("  -> handle_set_configuration()");

    let configuration = setup_request.value;
    if configuration > 1 {
        warn!("  stall: unknown configuration {}", configuration);
        usb0.stall_request();
        return Ok(());
    }

    usb0.ack_status_stage(setup_request);

    Ok(())
}

// - usb constants ------------------------------------------------------------

const USB_DEVICE_DESCRIPTOR: &[u8] = &[
    0x12, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x40, 0xd0, 0x16, 0x3b, 0x0f, 0x01, 0x01, 0x01, 0x02,
    0x00, 0x01,
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
