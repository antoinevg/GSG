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

use lunasoc_firmware as firmware;

use firmware::{hal, pac};
use hal::smolusb;
use pac::csr::interrupt;

use libgreat::Result;
use smolusb::control::{Direction, Recipient, Request, RequestType, SetupPacket};
use smolusb::descriptor::{
    ConfigurationDescriptor, DescriptorType, DeviceDescriptor, DeviceQualifierDescriptor, EndpointDescriptor,
    InterfaceDescriptor, LanguageId, StringDescriptor, StringDescriptorZero,
};

use hal::UsbInterface0;

use log::{debug, error, info, trace, warn};

// - MachineExternal interrupt handler ----------------------------------------

static mut STATE: bool = false;

#[allow(non_snake_case)]
#[no_mangle]
fn MachineExternal() {
    // peripherals
    let peripherals = unsafe { pac::Peripherals::steal() };
    let leds = &peripherals.LEDS;
    let mut usb0 = unsafe { hal::UsbInterface0::summon() };

    // debug
    let pending = interrupt::reg_pending();
    leds.output
        .write(|w| unsafe { w.output().bits(pending as u8) });
    //let mask = unsafe { interrupt::reg_mask() };
    //trace!("MachineExternal - 0b{:032b} 0b{:032b}", mask, pending);

    if usb0.is_pending(pac::Interrupt::USB0) {
        usb0.clear_pending(pac::Interrupt::USB0);
        usb0.reset();

    } else if usb0.is_pending(pac::Interrupt::USB0_EP_CONTROL) {
        usb0.clear_pending(pac::Interrupt::USB0_EP_CONTROL);
        panic!("MachineExternal - usb0.ep_control interrupt");

    } else if usb0.is_pending(pac::Interrupt::USB0_EP_IN) {
        usb0.clear_pending(pac::Interrupt::USB0_EP_IN);
        panic!("MachineExternal - usb0.ep_in interrupt");

    } else if usb0.is_pending(pac::Interrupt::USB0_EP_OUT) {
        usb0.clear_pending(pac::Interrupt::USB0_EP_OUT);

        let endpoint = usb0.ep_out.data_ep.read().bits();

        trace!("MachineExternal - usb0.ep_out interrupt ep:{}", endpoint);

    }  else {
        error!("MachineExternal - unknown interrupt");
        error!("  pend: {:#035b}", pending);
    }
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

    // usb
    let mut usb0 = hal::UsbInterface0::new(
        peripherals.USB0,
        peripherals.USB0_EP_CONTROL,
        peripherals.USB0_EP_IN,
        peripherals.USB0_EP_OUT,
    );
    info!("Connecting USB device...");
    let speed = usb0.connect();
    info!("Connected: {}", speed);


    // enable interrupts
    usb0.enable_interrupts();
    unsafe {
        // set mstatus register: interrupt enable
        riscv::interrupt::enable();

        // set mie register: machine external interrupts enable
        riscv::register::mie::set_mext();

        // write csr: enable interrupts
        interrupt::enable(pac::Interrupt::USB0);
        interrupt::enable(pac::Interrupt::USB0_EP_OUT);
    }

    // prime endpoints
    usb0.ep_out.epno.write(|w| unsafe { w.epno().bits(1) });
    usb0.ep_out.prime.write(|w| w.prime().bit(true));
    usb0.ep_out.epno.write(|w| unsafe { w.epno().bits(0) });
    usb0.ep_out.prime.write(|w| w.prime().bit(true));

    loop {
        // read setup request and handle it
        let packet = match read_setup_request(&usb0) {
            Ok(packet) => match handle_setup_request(&mut usb0, &packet) {
                Ok(()) => {
                    debug!("OK");
                    debug!("");
                }
                Err(e) => {
                    error!("  handle_setup_request: {:?}: {:?}", e, packet);
                    leds.output.write(|w| unsafe { w.output().bits(128) });
                    loop {}
                }
            }
            Err(e) => {
                error!("  read_setup_request: {:?}", e);
                continue;
            }
        };

    }
}


// - handle_read_ep_out -------------------------------------------------------

fn handle_read_ep_out(usb0: &UsbInterface0) -> Result<()> {
    debug!("# handle_read_ep_out()");

    // read data packet
    let mut buffer = [0_u8; 8];
    usb0.ep_out_read(0, &mut buffer)?;

    Ok(())
}

// - handle_write_ep_in -------------------------------------------------------

fn handle_write_ep_out(usb0: &UsbInterface0) -> Result<()> {
    debug!("# handle_write_ep_out()");
    Ok(())
}

// - read_setup_request -------------------------------------------------------

fn read_setup_request(usb0: &UsbInterface0) -> Result<SetupPacket> {
    debug!("# read_setup_request()");

    // read data packet
    let mut buffer = [0_u8; 8];
    usb0.ep_control_read_packet(&mut buffer)?;

    // parse data packet
    SetupPacket::try_from(buffer)
}

// - handle_setup_request -----------------------------------------------------

fn handle_setup_request(usb0: &mut UsbInterface0, packet: &SetupPacket) -> Result<()> {
    debug!("# handle_setup_request()",);

    // if this isn't a standard request, stall it.
    if packet.request_type() != RequestType::Standard {
        warn!(
            "   stall: unsupported request type {:?}",
            packet.request_type
        );
        usb0.stall_request();
        return Ok(());
    }

    // extract the request
    let request = match packet.request() {
        Ok(request) => request,
        Err(e) => {
            warn!("   stall: unsupported request {}: {:?}", packet.request, e);
            usb0.stall_request();
            return Ok(());
        }
    };

    debug!(
        "  dispatch: {:?} {:?} {:?} {}, {}",
        packet.recipient(),
        packet.direction(),
        request,
        packet.value,
        packet.length
    );

    match request {
        Request::SetAddress => handle_set_address(usb0, packet),
        Request::GetDescriptor => handle_get_descriptor(usb0, packet),
        Request::SetConfiguration => handle_set_configuration(usb0, packet),
        Request::GetConfiguration => handle_get_configuration(usb0, packet),
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
    usb0.set_address(address);

    debug!("  -> handle_set_address({})", address);

    Ok(())
}

fn handle_get_descriptor(usb0: &mut UsbInterface0, packet: &SetupPacket) -> Result<()> {
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

    // if the host is requesting less than the maximum amount of data,
    // only respond with the amount requested
    let requested_length = packet.length as usize;

    match (&descriptor_type, descriptor_number) {
        (DescriptorType::Device, 0) => {
            usb0.ep_in_write(0, USB_DEVICE_DESCRIPTOR.into_iter().take(requested_length));
        }
        (DescriptorType::Configuration, 0) => {
            usb0.ep_in_write(0, USB_CONFIG_DESCRIPTOR_0.iter().take(requested_length))
        }
        (DescriptorType::DeviceQualifier, 0) => {
            usb0.ep_in_write(0, USB_DEVICE_QUALIFIER_DESCRIPTOR.into_iter().take(requested_length));
        }
        (DescriptorType::OtherSpeedConfiguration, 0) => {
            usb0.ep_in_write(0, USB_OTHER_SPEED_CONFIG_DESCRIPTOR_0.iter().take(requested_length))
        }
        (DescriptorType::String, 0) => {
            usb0.ep_in_write(0, USB_STRING_DESCRIPTOR_0.iter().take(requested_length),
        )},
        (DescriptorType::String, 1) => {
            usb0.ep_in_write(0, USB_STRING_DESCRIPTOR_1.iter().take(requested_length))
        }
        (DescriptorType::String, 2) => {
            usb0.ep_in_write(0, USB_STRING_DESCRIPTOR_2.iter().take(requested_length))
        }
        (DescriptorType::String, 3) => {
            usb0.ep_in_write(0, USB_STRING_DESCRIPTOR_3.iter().take(requested_length))
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
        "  -> handle_get_descriptor({:?}({}), {}, {})",
        descriptor_type, descriptor_type_bits, descriptor_number, requested_length
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

fn handle_get_configuration(usb0: &UsbInterface0, packet: &SetupPacket) -> Result<()> {
    debug!("  -> handle_get_configuration()");

    let requested_length = packet.length as usize;

    usb0.ep_in_write(0, [1].into_iter().take(requested_length));
    usb0.ack_status_stage(packet);

    Ok(())
}


// - usb descriptors ----------------------------------------------------------

// fun product id's in 0x1d50:
//
// 604b  HackRF Jawbreaker Software-Defined Radio
// 6089  Great Scott Gadgets HackRF One SDR
// 60e6  replacement for GoodFET/FaceDancer - GreatFet
// 60e7  replacement for GoodFET/FaceDancer - GreatFet target
//
// From: http://www.linux-usb.org/usb.ids
static USB_DEVICE_DESCRIPTOR: DeviceDescriptor = DeviceDescriptor {
    descriptor_version: 0x0200,
    device_class: 0x00,
    device_subclass: 0x00,
    device_protocol: 0x00,
    max_packet_size: 64,
    vendor_id: 0x1d50,
    product_id: 0x60e7,
    device_version_number: 0x1234,
    manufacturer_string_index: 1,
    product_string_index: 2,
    serial_string_index: 3,
    num_configurations: 1,
};

static USB_DEVICE_QUALIFIER_DESCRIPTOR: DeviceQualifierDescriptor = DeviceQualifierDescriptor {
    _length: 10,
    _descriptor_type: DescriptorType::Device as u8,
    descriptor_version: 0x0200,
    device_class: 0x00,
    device_subclass: 0x00,
    device_protocol: 0x00,
    max_packet_size: 64,
    num_configurations: 1,
    reserved: 0,
};

static USB_CONFIG_DESCRIPTOR_0: ConfigurationDescriptor = ConfigurationDescriptor {
    _length: 9,
    descriptor_type: DescriptorType::Configuration, // TODO
    _total_length: 24, // config descriptor + interface descriptors + endpoint descriptors
    _num_interfaces: 1,
    configuration_value: 1,
    configuration_string_index: 1,
    attributes: 0x80, // 0b1000_0000
    max_power: 50,    // 50 * 2 mA = 100 mA
    interface_descriptors: &[&USB_INTERFACE_DESCRIPTOR_0],
};

static USB_OTHER_SPEED_CONFIG_DESCRIPTOR_0: ConfigurationDescriptor = ConfigurationDescriptor {
    _length: 9,
    descriptor_type: DescriptorType::OtherSpeedConfiguration, // TODO
    _total_length: 36, // config descriptor + interface descriptors + endpoint descriptors
    _num_interfaces: 1,
    configuration_value: 1,
    configuration_string_index: 1,
    attributes: 0x80, // 0b1000_0000
    max_power: 50,    // 50 * 2 mA = 100 mA
    interface_descriptors: &[&USB_INTERFACE_DESCRIPTOR_0],
};

static USB_INTERFACE_DESCRIPTOR_0: InterfaceDescriptor = InterfaceDescriptor {
    _length: 9,
    _descriptor_type: DescriptorType::Interface as u8,
    interface_number: 0,
    alternate_setting: 0,
    _num_endpoints: 1,
    interface_class: 0xff, // Vendor Specific - https://www.usb.org/defined-class-codes
    interface_subclass: 0x00,
    interface_protocol: 0x00,
    interface_string_index: 2,
    endpoint_descriptors: &[
        &USB_ENDPOINT_DESCRIPTOR_0,
        &USB_ENDPOINT_DESCRIPTOR_1,
        //&USB_ENDPOINT_DESCRIPTOR_2,
    ],
};

static USB_ENDPOINT_DESCRIPTOR_0: EndpointDescriptor = EndpointDescriptor {
    _length: 7,
    _descriptor_type: DescriptorType::Endpoint as u8,
    endpoint_address: 0x82, // IN
    attributes: 0x02,       // Bulk
    max_packet_size: 64,
    interval: 0,
};

static USB_ENDPOINT_DESCRIPTOR_1: EndpointDescriptor = EndpointDescriptor {
    _length: 7,
    _descriptor_type: DescriptorType::Endpoint as u8,
    endpoint_address: 0x02, // OUT
    attributes: 0x02,       // Bulk
    max_packet_size: 64,
    interval: 0,
};

static USB_ENDPOINT_DESCRIPTOR_2: EndpointDescriptor = EndpointDescriptor {
    _length: 7,
    _descriptor_type: DescriptorType::Endpoint as u8,
    endpoint_address: 0x81, // IN
    attributes: 0x03,       // Interrupt
    max_packet_size: 8,
    interval: 1, // 1ms
};

static USB_STRING_DESCRIPTOR_0: StringDescriptorZero = StringDescriptorZero {
    _length: 10,
    _descriptor_type: DescriptorType::String as u8,
    language_ids: &[
        LanguageId::EnglishUnitedStates,
        //LanguageId::EnglishUnitedKingdom,
        //LanguageId::EnglishCanadian,
        //LanguageId::EnglishSouthAfrica,
    ],
};
static USB_STRING_DESCRIPTOR_1: StringDescriptor = StringDescriptor::new("LUNA");
static USB_STRING_DESCRIPTOR_2: StringDescriptor = StringDescriptor::new("Simple Endpoint Test");
static USB_STRING_DESCRIPTOR_3: StringDescriptor = StringDescriptor::new("v1.0");

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
