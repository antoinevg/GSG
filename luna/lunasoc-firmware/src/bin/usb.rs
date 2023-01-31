#![feature(error_in_core)]
#![feature(panic_info_message)]
#![allow(
    dead_code,
    unused_imports,
    unreachable_code,
    unused_mut,
    unused_variables
)] // TODO
#![no_std]
#![no_main]

//use panic_halt as _;
#[panic_handler]
#[no_mangle]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    if let Some(message) = panic_info.message() {
        error!("Panic: {}", message);
    } else {
        error!("Panic: Unknown");
    }

    if let Some(location) = panic_info.location() {
        error!("'{}' : {}", location.file(), location.line(),);
    }

    loop {}
}

use riscv_rt::entry;

use firmware::{hal, pac};
use lunasoc_firmware as firmware;

use libgreat::Result;
use libgreat::smolusb::control::{
    DescriptorType, Direction, Recipient, Request, RequestType, SetupPacket,
};

use hal::UsbInterface0;

use log::{debug, error, info, trace, warn};

#[entry]
fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();
    let leds = &peripherals.LEDS;
    leds.output.write(|w| unsafe { w.output().bits(0x0) });

    // initialize logging
    let serial = hal::Serial::new(peripherals.UART);
    firmware::log::init(serial);
    info!("logging initialized");

    // timer
    let sysclk = firmware::SYSTEM_CLOCK_FREQUENCY;
    let mut timer = hal::Timer::new(peripherals.TIMER, sysclk);
    timer.set_timeout_ticks(sysclk * 1);
    timer.enable();
    timer.listen(hal::timer::Event::TimeOut);

    // usb
    let usb0 = hal::UsbInterface0::new(
        peripherals.USB0,
        peripherals.USB0_EP_CONTROL,
        peripherals.USB0_EP_IN,
        peripherals.USB0_EP_OUT,
    );
    info!("Connecting USB device...");
    let speed = usb0.connect();
    //usb0.listen();
    info!("Connected: {}", speed);

    unsafe {
        // set mstatus register: interrupt enable
        riscv::interrupt::enable();

        // set mie register: machine external interrupts enable
        riscv::register::mie::set_mext();

        // write csr: enable interrupts
        pac::csr::interrupt::enable(pac::Interrupt::TIMER);
        pac::csr::interrupt::enable(pac::Interrupt::USB0);
        //pac::csr::interrupt::enable(pac::Interrupt::USB0_SETUP);
        //pac::csr::interrupt::enable(pac::Interrupt::USB0_EP_IN);
        //pac::csr::interrupt::enable(pac::Interrupt::USB0_EP_OUT);
    }

    loop {
        //unsafe { riscv::asm::delay(sysclk) };
        //continue;

        // read setup request
        let packet = match read_setup_request(&usb0) {
            Ok(packet) => packet,
            Err(e) => {
                error!("  Setup Error {:?}", e);
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
                error!("  Request Error {:?}: {:?}", e, packet);
                leds.output.write(|w| unsafe { w.output().bits(128) });
                loop {}
            }
        };
    }
}

// - MachineExternal interrupt handler ----------------------------------------

static mut STATE: bool = false;

#[allow(non_snake_case)]
#[no_mangle]
fn MachineExternal() {
    // peripherals
    let peripherals = unsafe { pac::Peripherals::steal() };
    let leds = &peripherals.LEDS;
    let timer = &peripherals.TIMER;
    let mut usb0 = unsafe { UsbInterface0::summon() };

    // debug
    let pending = unsafe { pac::csr::interrupt::reg_pending() };
    leds.output
        .write(|w| unsafe { w.output().bits((1 << pending) as u8) });

    if usb0.device.ev_pending.read().pending().bit() {
        usb0.device
            .ev_pending
            .modify(|r, w| w.pending().bit(r.pending().bit()));

        usb0.reset();
        //debug!("MachineExternal - usb0.device interrupt");
    } else if usb0.ep_control.ev_pending.read().pending().bit() {
        usb0.ep_control
            .ev_pending
            .modify(|r, w| w.pending().bit(r.pending().bit()));
        //debug!("MachineExternal - usb0.ep_setup interrupt");
    } else if usb0.ep_in.ev_pending.read().pending().bit() {
        usb0.ep_in
            .ev_pending
            .modify(|r, w| w.pending().bit(r.pending().bit()));
        //debug!("MachineExternal - usb0.ep_in interrupt");
    } else if usb0.ep_out.ev_pending.read().pending().bit() {
        usb0.ep_out
            .ev_pending
            .modify(|r, w| w.pending().bit(r.pending().bit()));
        //debug!("MachineExternal - usb0.ep_out interrupt");
    } else if timer.ev_pending.read().pending().bit() {
        timer
            .ev_pending
            .modify(|r, w| w.pending().bit(r.pending().bit()));
    } else {
        error!("MachineExternal - unknown interrupt");
        error!("  pend: {:#035b}", pending);
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
            warn!("   stall: invalid recipient: {}", bits);
            usb0.stall_request();
            return Ok(());
        }
    };

    // extract the request type (e.g. standard/class/vendor) from our SETUP request.
    let bits: u8 = (packet.request_type >> 5) & 0b0000_0011;
    let request_type = match RequestType::try_from(bits) {
        Ok(request_type) => request_type,
        Err(e) => {
            warn!("   stall: invalid request type: {}", bits);
            usb0.stall_request();
            return Ok(());
        }
    };

    // extract the direction
    let bits: u8 = (packet.request_type >> 7) & 0b0000_0001;
    let direction = match Direction::try_from(bits) {
        Ok(direction) => direction,
        Err(e) => {
            warn!("   stall: invalid direction: {}", bits);
            usb0.stall_request();
            return Ok(());
        }
    };

    // TODO: Get rid of this once we move to be fully compatible with ValentyUSB.
    //usb0.ep_in.pid.write(|w| w.pid().bit(true));

    // if this isn't a standard request, stall it.
    if request_type != RequestType::Standard {
        warn!("   stall: unsupported request type {:?}", request_type);
        usb0.stall_request();
        return Ok(());
    }

    // Extract the request
    let request = match Request::try_from(packet.request) {
        Ok(request) => request,
        Err(e) => {
            warn!("   stall: invalid request: {}", packet.request);
            usb0.stall_request();
            return Ok(());
        }
    };

    debug!(
        "  dispatch: {:?} {:?} {:?} {}, {}",
        recipient, direction, request, packet.value, packet.length
    );

    match request {
        Request::SetAddress => handle_set_address(usb0, packet),
        Request::GetStatus => handle_get_status(usb0, packet),
        Request::SetDescriptor => handle_set_descriptor(usb0, packet),
        Request::GetDescriptor => handle_get_descriptor(usb0, packet),
        Request::SetConfiguration => handle_set_configuration(usb0, packet),
        _ => {
            warn!("   stall: unhandled request {:?}", request);
            usb0.stall_request();
            Ok(())
        }
    }
}

fn handle_set_address(usb0: &UsbInterface0, packet: &SetupPacket) -> Result<()> {
    usb0.ack_status_stage(packet);

    let address: u8 = (packet.value & 0x7f) as u8;
    usb0.ep_setup_set_address(address);
    debug!("  -> handle_set_address({})", address);

    Ok(())
}

/// ???
fn handle_get_status(usb0: &UsbInterface0, packet: &SetupPacket) -> Result<()> {
    usb0.ack_status_stage(packet);

    debug!("  -> handle_get_status()");

    Ok(())
}

/// ???
fn handle_set_descriptor(usb0: &UsbInterface0, packet: &SetupPacket) -> Result<()> {
    usb0.ack_status_stage(packet);

    debug!("  -> handle_set_descriptor()");

    let descriptor = packet.value;
    if descriptor > 1 {
        warn!("   stall: unknown descriptor {}", descriptor);
        usb0.stall_request();
        return Ok(());
    }

    Ok(())
}

fn handle_get_descriptor(usb0: &UsbInterface0, packet: &SetupPacket) -> Result<()> {
    // extract the descriptor type and number from our SETUP request
    let [descriptor_number, descriptor_type_bits] = packet.value.to_le_bytes();
    let descriptor_type = match DescriptorType::try_from(descriptor_type_bits) {
        Ok(descriptor_type) => descriptor_type,
        Err(e) => {
            warn!(
                "   stall: invalid descriptor type: {} {}",
                descriptor_type_bits, descriptor_number
            );
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
                "   stall: unhandled descriptor {:?}, {}",
                descriptor_type, descriptor_number
            );
            usb0.stall_request();
            return Ok(());
        }
    }

    usb0.ack_status_stage(packet);

    debug!(
        "  -> handle_get_descriptor({:?}({}), {})",
        descriptor_type, descriptor_type_bits, descriptor_number
    );

    Ok(())
}

fn handle_set_configuration(usb0: &UsbInterface0, packet: &SetupPacket) -> Result<()> {
    usb0.ack_status_stage(packet);

    debug!("  -> handle_set_configuration()");

    let configuration = packet.value;
    if configuration > 1 {
        warn!("   stall: unknown configuration {}", configuration);
        usb0.stall_request();
        return Ok(());
    }

    Ok(())
}

// - usb constants ------------------------------------------------------------

const USB_DEVICE_DESCRIPTOR: &[u8] = &[
    0x12, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x40, 0xd0, 0x16, 0x3b, 0x0f, 0x01, 0x01, 0x01, 0x02,
    0x00, 0x01,
];

// different serial number to above
const _USB_DEVICE_DESCRIPTOR: &[u8] = &[
    0x12, // Length = 18
    0x01, // DescriptorType = DEVICE
    0x00, 0x02, // bcdUSB = 0x0200
    0x00, // DeviceClass
    0x00, // DeviceSubClass
    0x00, // DeviceProtocol
    0x40, // MaxPacketSize = 64
    0xd0, 0x16, // idVendor
    0x3b, 0x0f, // idProduct
    0x00, 0x00, // bcdDevice
    0x01, // iManufacturer
    0x02, // iProduct
    0x03, // iSerialNumber
    0x01, // bNumConfigurations
];

const USB_CONFIG_DESCRIPTOR: &[u8] = &[
    0x09, 0x02, 0x12, 0x00, 0x01, 0x01, 0x01, 0x80, 0x32, 0x09, 0x04, 0x00, 0x00, 0x00, 0xfe, 0x00,
    0x00, 0x02,
];

// same as above
const _USB_CONFIG_DESCRIPTOR: &[u8] = &[
    0x09, // Length
    0x02, // DescriptorType = CONFIG
    0x12, 0x00, // TotalLength = 18 bytes
    0x01, // NumInterfaces
    0x01, // ConfigurationValue
    0x01, // ConfigurationIndex
    0x80, // Attributes = 0b1000_0000
    0x32, // MaxPower = 50 * 2 mA = 100 mA
    // INTERFACE
    0x09, // Length
    0x04, // DescriptorType = INTERFACE
    0x00, // InterfaceNumber
    0x00, // AlternateSetting
    0x00, // NumEndpoints
    0xfe, // InterfaceClass
    0x00, // InterfaceSubClass
    0x00, // InterfaceProtocol
    0x02, // StringIndex

          // ENDPOINT
];

// counter example
const __USB_CONFIG_DESCRIPTOR: &[u8] = &[
    0x09, 0x02, 0x19, 0x00, 0x01, 0x01, 0x00, 0x80, 0xfa, 0x09, 0x04, 0x00, 0x00, 0x01, 0xff, 0xff,
    0xff, 0x00, 0x07, 0x05, 0x81, 0x02, 0x00, 0x02, 0xff,
];

const USB_STRING0_DESCRIPTOR: &[u8] = &[0x04, 0x03, 0x09, 0x04];

const USB_STRING1_DESCRIPTOR: &[u8] = &[0x0a, 0x03, b'L', 0x00, b'U', 0x00, b'N', 0x00, b'A', 0x00];

const _USB_STRING2_DESCRIPTOR: &[u8] = &[
    0x22, 0x03, b'T', 0, b'r', 0, b'i', 0, b'-', 0, b'F', 0, b'I', 0, b'F', 0, b'O', 0, b' ', 0,
    b'E', 0, b'x', 0, b'a', 0, b'm', 0, b'p', 0, b'l', 0, b'e', 0,
];

// counter example
const USB_STRING2_DESCRIPTOR: &[u8] = &[
    0x30, 0x03, 0x43, 0x00, 0x6f, 0x00, 0x75, 0x00, 0x6e, 0x00, 0x74, 0x00, 0x65, 0x00, 0x72, 0x00,
    0x2f, 0x00, 0x54, 0x00, 0x68, 0x00, 0x72, 0x00, 0x6f, 0x00, 0x75, 0x00, 0x67, 0x00, 0x68, 0x00,
    0x70, 0x00, 0x75, 0x00, 0x74, 0x00, 0x20, 0x00, 0x54, 0x00, 0x65, 0x00, 0x73, 0x00, 0x74, 0x00,
];

/*
# Reference enumeration process (quirks merged from Linux, macOS, and Windows):
# - Read 8 bytes of device descriptor.
# + Read 64 bytes of device descriptor.
# + Set address.
# + Read exact device descriptor length.
# - Read device qualifier descriptor, three times.
# - Read config descriptor (without subordinates).
# - Read language descriptor.
# - Read Windows extended descriptors. [optional]
# - Read string descriptors from device descriptor (wIndex=language id).
# - Set configuration.
# - Read back configuration number and validate.

*/
