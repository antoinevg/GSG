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

use smolusb::control::SetupPacket;
use smolusb::descriptor::*;
use smolusb::device::{DeviceState, Speed, UsbDevice};
use smolusb::traits::{ControlRead, EndpointRead, EndpointWrite, UsbDriverOperations};

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
        Message::Usb0ReceivedSetupPacket(setup_packet)
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

        // clear pending IRQ after data is read
        usb0.clear_pending(pac::Interrupt::USB0_EP_OUT);

        Message::Usb0ReceivedData(endpoint, bytes_read, buffer)
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
    info!("Connected USB device: {:?}", Speed::from(speed));

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

    let mut usb0_device = UsbDevice::new(
        &usb0,
        &USB_DEVICE_DESCRIPTOR,
        &USB_CONFIG_DESCRIPTOR_0,
        &USB_STRING_DESCRIPTOR_0,
        &USB_STRING_DESCRIPTORS,
    );
    let mut counter: usize = 1;
    let mut start_polling = false;

    loop {
        // send some data occasionally
        /*if usb0_device.state == DeviceState::Configured && counter % 300_000 == 0 {
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
        }*/

        // queue a little test data on interrupt endpoint occasionally
        if start_polling
            && (usb0_device.state == DeviceState::Configured)
            && (counter % 10_000 == 0)
        {
            let endpoint = 2;
            const SIZE: usize = 8;
            let data: heapless::Vec<u8, SIZE> = (0..(SIZE as u8))
                .collect::<heapless::Vec<u8, SIZE>>()
                .try_into()
                .unwrap();
            let bytes_written = data.len();
            usb0.write(endpoint, data.into_iter());
            info!(
                "Sent {} bytes to interrupt endpoint: {}",
                bytes_written, endpoint
            );
            counter = 1;
        } else {
            counter += 1;
        }

        if let Some(message) = MESSAGE_QUEUE.dequeue() {
            match message {
                Message::Usb0ReceivedSetupPacket(packet) => {
                    start_polling = false;
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

                Message::Usb0ReceivedData(endpoint, bytes_read, buffer) => {
                    if endpoint != 0 {
                        debug!(
                            "Received {} bytes on endpoint: {} - {:?}\n",
                            bytes_read, endpoint, buffer
                        );
                        start_polling = true;
                        /*
                        // queue a little test data on interrupt endpoint
                        let endpoint = 2;
                        const SIZE: usize = 8;
                        let data: heapless::Vec<u8, SIZE> =
                            (0..(SIZE as u8)).collect::<heapless::Vec<u8, SIZE>>().try_into().unwrap();
                        let bytes_written = data.len();
                        usb0.write(endpoint, data.into_iter());
                        info!(
                            "Sent {} bytes to interrupt endpoint: {}",
                            bytes_written, endpoint
                        );*/
                    }
                }

                // interrupts
                Message::Interrupt(pac::Interrupt::USB0) => {
                    usb0_device.reset();
                    trace!("MachineExternal - USB0\n");
                }
                Message::Interrupt(pac::Interrupt::USB0_EP_CONTROL) => {
                    // handled in MachineExternal which queues a Message::ReceivedSetupPacket
                }
                Message::Interrupt(pac::Interrupt::USB0_EP_IN) => {
                    // TODO - handle transmission complete
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
    descriptor_version: 0x0200,
    device_class: 0x00,
    device_subclass: 0x00,
    device_protocol: 0x00,
    max_packet_size: 64,
    num_configurations: 1,
    reserved: 0,
    ..DeviceQualifierDescriptor::new()
};

static USB_CONFIG_DESCRIPTOR_0: ConfigurationDescriptor = ConfigurationDescriptor::new(
    ConfigurationDescriptorHeader {
        configuration_value: 1,
        configuration_string_index: 1,
        attributes: 0x80, // 0b1000_0000
        max_power: 50,    // 50 * 2 mA = 100 mA
        ..ConfigurationDescriptorHeader::new()
    },
    &[InterfaceDescriptor::new(
        InterfaceDescriptorHeader {
            interface_number: 0,
            alternate_setting: 0,
            interface_class: 0xff, // Vendor Specific - https://www.usb.org/defined-class-codes
            interface_subclass: 0x00,
            interface_protocol: 0x00, // 0x02 is CDC
            interface_string_index: 2,
            ..InterfaceDescriptorHeader::new()
        },
        &[
            EndpointDescriptor {
                endpoint_address: 0x01, // OUT
                attributes: 0x02,       // Bulk
                max_packet_size: 512,
                interval: 0,
                ..EndpointDescriptor::new()
            },
            EndpointDescriptor {
                endpoint_address: 0x02, // OUT
                attributes: 0x02,       // Bulk
                max_packet_size: 512,
                interval: 0,
                ..EndpointDescriptor::new()
            },
            EndpointDescriptor {
                endpoint_address: 0x04, // OUT
                attributes: 0x02,       // Bulk
                max_packet_size: 512,
                interval: 0,
                ..EndpointDescriptor::new()
            },
            EndpointDescriptor {
                endpoint_address: 0x81, // IN
                attributes: 0x02,       // Bulk
                max_packet_size: 512,
                interval: 0,
                ..EndpointDescriptor::new()
            },
            EndpointDescriptor {
                endpoint_address: 0x82, // IN
                attributes: 0x03,       // Interrupt
                max_packet_size: 8,
                interval: 1, // x 1ms for low/full speed, 125us for high speed
                ..EndpointDescriptor::new()
            },
        ],
    )],
);

static USB_OTHER_SPEED_CONFIG_DESCRIPTOR_0: ConfigurationDescriptorHeader =
    ConfigurationDescriptorHeader {
        descriptor_type: DescriptorType::OtherSpeedConfiguration as u8,
        configuration_value: 1,
        configuration_string_index: 1,
        attributes: 0x80,                       // 0b1000_0000
        max_power: 50,                          // 50 * 2 mA = 100 mA
        ..ConfigurationDescriptorHeader::new() //interface_descriptors: &[&USB_INTERFACE_DESCRIPTOR_0],
    };

static USB_STRING_DESCRIPTOR_0: StringDescriptorZero =
    StringDescriptorZero::new(&[LanguageId::EnglishUnitedStates]);

static USB_STRING_DESCRIPTOR_1: StringDescriptor = StringDescriptor::new("LUNA");
static USB_STRING_DESCRIPTOR_2: StringDescriptor = StringDescriptor::new("Simple Endpoint Test");
static USB_STRING_DESCRIPTOR_3: StringDescriptor = StringDescriptor::new("v1.0");

static USB_STRING_DESCRIPTORS: &[&StringDescriptor] = &[
    &USB_STRING_DESCRIPTOR_1,
    &USB_STRING_DESCRIPTOR_2,
    &USB_STRING_DESCRIPTOR_3,
];
