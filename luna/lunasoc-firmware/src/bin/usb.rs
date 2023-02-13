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

use riscv_rt::entry;

use lunasoc_firmware as firmware;

use firmware::{hal, pac};
use hal::smolusb;
use pac::csr::interrupt;

use libgreat::Result;
use smolusb::control::{Direction, Recipient, Request, RequestType, SetupPacket};
use smolusb::descriptor::{
    ConfigurationDescriptor, DescriptorType, DeviceDescriptor, DeviceQualifierDescriptor,
    EndpointDescriptor, InterfaceDescriptor, LanguageId, StringDescriptor, StringDescriptorZero,
};
use smolusb::device::UsbDevice;
use smolusb::traits::AsByteSliceIterator;
use smolusb::traits::{
    ControlRead, EndpointRead, EndpointWrite, EndpointWriteRef, UsbDriverOperations,
};

use hal::Usb0;

use log::{debug, error, info, trace, warn};

// - panic handler ------------------------------------------------------------

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

// - global static state ------------------------------------------------------

use firmware::Message;
use heapless::mpmc::MpMcQueue as Queue;
static MESSAGE_QUEUE: Queue<Message, 128> = Queue::new();

// - MachineExternal interrupt handler ----------------------------------------

static mut TX_READY: bool = false;

#[allow(non_snake_case)]
#[no_mangle]
fn MachineExternal() {
    // peripherals
    let peripherals = unsafe { pac::Peripherals::steal() };
    let leds = &peripherals.LEDS;
    let mut usb0 = unsafe { hal::Usb0::summon() };

    // debug
    let pending = interrupt::reg_pending();
    leds.output
        .write(|w| unsafe { w.output().bits(pending as u8) });

    let message = if usb0.is_pending(pac::Interrupt::USB0) {
        usb0.clear_pending(pac::Interrupt::USB0);
        Message::Interrupt(pac::Interrupt::USB0)
    } else if usb0.is_pending(pac::Interrupt::USB0_EP_CONTROL) {
        // read packet
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
        Message::ReceivedSetupPacket(setup_packet)
    } else if usb0.is_pending(pac::Interrupt::USB0_EP_IN) {
        usb0.clear_pending(pac::Interrupt::USB0_EP_IN);
        Message::Interrupt(pac::Interrupt::USB0_EP_IN)
    } else if usb0.is_pending(pac::Interrupt::USB0_EP_OUT) {
        // read data from endpoint
        let endpoint = usb0.ep_out.data_ep.read().bits() as u8;
        let mut buffer = [0_u8; 64];
        let bytes_read = usb0.read(endpoint, &mut buffer);

        // TODO this does feel somewhat dodge to put it mildly
        for ep in (0..=4).rev() {
            usb0.ep_out.epno.write(|w| unsafe { w.epno().bits(ep) });
            usb0.ep_out.prime.write(|w| w.prime().bit(true));
            usb0.ep_out.enable.write(|w| w.enable().bit(true));
        }

        // TODO
        if endpoint != 0 && unsafe { TX_READY } == false {
            unsafe {
                TX_READY = true;
            }
        };

        // clear pending IRQ after data is read
        usb0.clear_pending(pac::Interrupt::USB0_EP_OUT);

        Message::ReceivedData(endpoint, bytes_read, buffer)
    } else {
        Message::UnknownInterrupt(pending)
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

    // usb
    let mut usb0 = hal::Usb0::new(
        peripherals.USB0,
        peripherals.USB0_EP_CONTROL,
        peripherals.USB0_EP_IN,
        peripherals.USB0_EP_OUT,
    );
    let speed = usb0.connect();
    info!("Connected USB device: {}", speed);

    // enable interrupts
    usb0.enable_interrupts();
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
    }

    let usb0_device = UsbDevice::new(
        &usb0,
        &USB_DEVICE_DESCRIPTOR,
        &USB_CONFIG_DESCRIPTOR_0,
        &USB_STRING_DESCRIPTOR_0,
        &USB_STRING_DESCRIPTORS,
    );
    let mut counter: usize = 1;

    loop {
        // send some data occasionally
        if unsafe { TX_READY } && counter % 300_000 == 0 {
            // interrupt
            let endpoint = 2;
            let data: heapless::Vec<u8, 8> = (0..8)
                .collect::<heapless::Vec<u8, 8>>()
                .try_into()
                .unwrap();
            let bytes_written = data.len();
            usb0.write(endpoint, data.into_iter());
            info!("Sent {} bytes to interrupt endpoint: {}", bytes_written, endpoint);

            // bulk out
            let endpoint = 1;
            let data: heapless::Vec<u8, 64> = (0..64)
                .collect::<heapless::Vec<u8, 64>>()
                .try_into()
                .unwrap();
            let bytes_written = data.len();
            usb0.write(endpoint, data.into_iter());
            info!("Sent {} bytes to endpoint: {}", bytes_written, endpoint);

            counter = 1;
        } else {
            counter += 1;
        }

        if let Some(message) = MESSAGE_QUEUE.dequeue() {
            match message {
                Message::ReceivedSetupPacket(packet) => {
                    match usb0_device.handle_setup_request(&packet) {
                        Ok(()) => {
                            debug!("OK");
                            debug!("");
                        }
                        Err(e) => {
                            error!("  handle_setup_request: {:?}: {:?}", e, packet);
                            loop {}
                        }
                    }
                }

                Message::ReceivedData(endpoint, bytes_read, buffer) => {
                    if endpoint != 0 {
                        debug!(
                            "Received {} bytes on endpoint: {} - {:?}\n",
                            bytes_read, endpoint, buffer
                        );
                    }
                }

                // interrupts
                Message::Interrupt(pac::Interrupt::USB0) => {
                    usb0.reset();
                    trace!("MachineExternal - USB0\n");
                }
                Message::Interrupt(pac::Interrupt::USB0_EP_CONTROL) => {
                    // handled in MachineExternal which queues a Message::ReceivedSetupPacket
                }
                Message::Interrupt(pac::Interrupt::USB0_EP_IN) => {
                    // TODO
                    trace!("MachineExternal - USB0_EP_IN\n");
                }
                Message::Interrupt(pac::Interrupt::USB0_EP_OUT) => {
                    // handled in MachineExternal which queues a Message::ReceivedData
                }

                Message::Interrupt(interrupt) => {
                    warn!("Unhandled interrupt: {:?}\n", interrupt);
                }
                Message::UnknownInterrupt(pending) => {
                    error!("Unknown interrupt pending: {:#035b}\n", pending);
                }
                _ => {
                    panic!("Unhandled message: {:?}\n", message);
                }
            }
        }
    }
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
    ..DeviceDescriptor::new()
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
    interface_protocol: 0x00, // 0x02 is CDC
    interface_string_index: 2,
    endpoint_descriptors: &[
        &USB_ENDPOINT_DESCRIPTOR_01,
        &USB_ENDPOINT_DESCRIPTOR_02,
        &USB_ENDPOINT_DESCRIPTOR_03,
        &USB_ENDPOINT_DESCRIPTOR_04,
        &USB_ENDPOINT_DESCRIPTOR_81,
        &USB_ENDPOINT_DESCRIPTOR_82,
    ],
};

static USB_ENDPOINT_DESCRIPTOR_01: EndpointDescriptor = EndpointDescriptor {
    _length: 7,
    _descriptor_type: DescriptorType::Endpoint as u8,
    endpoint_address: 0x01, // OUT
    attributes: 0x02,       // Bulk
    max_packet_size: 64,
    interval: 0,
};

static USB_ENDPOINT_DESCRIPTOR_02: EndpointDescriptor = EndpointDescriptor {
    _length: 7,
    _descriptor_type: DescriptorType::Endpoint as u8,
    endpoint_address: 0x02, // OUT
    attributes: 0x02,       // Bulk
    max_packet_size: 64,
    interval: 0,
};

static USB_ENDPOINT_DESCRIPTOR_03: EndpointDescriptor = EndpointDescriptor {
    _length: 7,
    _descriptor_type: DescriptorType::Endpoint as u8,
    endpoint_address: 0x03, // OUT
    attributes: 0x02,       // Bulk
    max_packet_size: 64,
    interval: 0,
};

static USB_ENDPOINT_DESCRIPTOR_04: EndpointDescriptor = EndpointDescriptor {
    _length: 7,
    _descriptor_type: DescriptorType::Endpoint as u8,
    endpoint_address: 0x04, // OUT
    attributes: 0x02,       // Bulk
    max_packet_size: 64,
    interval: 0,
};

static USB_ENDPOINT_DESCRIPTOR_81: EndpointDescriptor = EndpointDescriptor {
    _length: 7,
    _descriptor_type: DescriptorType::Endpoint as u8,
    endpoint_address: 0x81, // IN
    attributes: 0x02,       // Bulk
    max_packet_size: 64,
    interval: 0,
};

static USB_ENDPOINT_DESCRIPTOR_82: EndpointDescriptor = EndpointDescriptor {
    _length: 7,
    _descriptor_type: DescriptorType::Endpoint as u8,
    endpoint_address: 0x82, // IN
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

static USB_STRING_DESCRIPTORS: &[&StringDescriptor] = &[
    &USB_STRING_DESCRIPTOR_1,
    &USB_STRING_DESCRIPTOR_2,
    &USB_STRING_DESCRIPTOR_3,
];
