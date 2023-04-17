#![allow(dead_code, unused_imports, unused_variables)] // TODO
#![no_std]
#![no_main]

use cynthion::{hal, pac};

use hal::{smolusb, Serial};

use smolusb::control::SetupPacket;
use smolusb::descriptor::*;
use smolusb::device::{DeviceState, Speed, UsbDevice};
use smolusb::traits::{
    ControlRead, EndpointRead, EndpointWrite, EndpointWriteRef, UnsafeUsbDriverOperations,
    UsbDriverOperations,
};

use libgreat::{GreatError, GreatResult};

use log::{debug, error, info};

use riscv_rt::entry;

// - global static state ------------------------------------------------------

const TEST_WRITE_SIZE: usize = 512;

use cynthion::Message;
use heapless::mpmc::MpMcQueue as Queue;
static MESSAGE_QUEUE: Queue<Message, 32> = Queue::new();

// - MachineExternal interrupt handler ----------------------------------------

#[allow(non_snake_case)]
#[no_mangle]
fn MachineExternal() {
    let usb1 = unsafe { hal::Usb1::summon() };

    // - usb1 interrupts - "host_phy" --

    // USB1 UsbBusReset
    let message = if usb1.is_pending(pac::Interrupt::USB1) {
        usb1.clear_pending(pac::Interrupt::USB1);
        usb1.bus_reset();
        Message::UsbBusReset(1)

    // USB1_EP_CONTROL UsbReceiveSetupPacket
    } else if usb1.is_pending(pac::Interrupt::USB1_EP_CONTROL) {
        let mut setup_packet_buffer = [0_u8; 8];
        usb1.read_control(&mut setup_packet_buffer);
        usb1.clear_pending(pac::Interrupt::USB1_EP_CONTROL);

        match SetupPacket::try_from(setup_packet_buffer) {
            Ok(setup_packet) => Message::UsbReceiveSetupPacket(1, setup_packet),
            Err(e) => Message::ErrorMessage("USB1_EP_CONTROL failed to read setup packet"),
        }

    // USB1_EP_OUT UsbReceiveData
    } else if usb1.is_pending(pac::Interrupt::USB1_EP_OUT) {
        let endpoint = usb1.ep_out.data_ep.read().bits() as u8;
        let mut buffer = [0_u8; cynthion::EP_MAX_RECEIVE_LENGTH];
        let bytes_read = usb1.read(endpoint, &mut buffer);
        usb1.clear_pending(pac::Interrupt::USB1_EP_OUT);

        Message::UsbReceiveData(1, endpoint, bytes_read, buffer)

    // USB1_EP_IN UsbTransferComplete
    } else if usb1.is_pending(pac::Interrupt::USB1_EP_IN) {
        usb1.clear_pending(pac::Interrupt::USB1_EP_IN);
        let endpoint = usb1.ep_in.epno.read().bits() as u8;

        // TODO something a little bit safer would be nice
        unsafe {
            usb1.clear_tx_ack_active();
        }

        Message::UsbTransferComplete(1, endpoint)

    // - Unknown Interrupt --
    } else {
        let pending = pac::csr::interrupt::reg_pending();
        Message::HandleUnknownInterrupt(pending)
    };

    match MESSAGE_QUEUE.enqueue(message) {
        Ok(()) => (),
        Err(_) => {
            error!("MachineExternal - message queue overflow");
            panic!("MachineExternal - message queue overflow");
        }
    }
}

// - main entry point ---------------------------------------------------------

#[entry]
fn main() -> ! {
    match main_loop() {
        Ok(()) => {
            error!("Firmware exited unexpectedly in main loop");
            panic!("Firmware exited unexpectedly in main loop")
        }
        Err(e) => {
            error!("Fatal error in firmware main loop: {}", e);
            panic!("Fatal error in firmware main loop: {}", e)
        }
    }
}

// - main loop ----------------------------------------------------------------

fn main_loop() -> GreatResult<()> {
    let peripherals = pac::Peripherals::take().unwrap();
    let leds = &peripherals.LEDS;

    // initialize logging
    cynthion::log::init(hal::Serial::new(peripherals.UART));
    info!("Logging initialized");

    // usb1: host
    let mut usb1 = UsbDevice::new(
        hal::Usb1::new(
            peripherals.USB1,
            peripherals.USB1_EP_CONTROL,
            peripherals.USB1_EP_IN,
            peripherals.USB1_EP_OUT,
        ),
        &USB_DEVICE_DESCRIPTOR,
        &USB_CONFIGURATION_DESCRIPTOR_0,
        &USB_STRING_DESCRIPTOR_0,
        &USB_STRING_DESCRIPTORS,
    );
    usb1.device_qualifier_descriptor = Some(&USB_DEVICE_QUALIFIER_DESCRIPTOR);
    usb1.other_speed_configuration_descriptor = Some(USB_OTHER_SPEED_CONFIGURATION_DESCRIPTOR_0);
    let speed = usb1.connect();
    debug!("Connected usb1 device: {:?}", speed);

    // enable interrupts
    unsafe {
        // set mstatus register: interrupt enable
        riscv::interrupt::enable();

        // set mie register: machine external interrupts enable
        riscv::register::mie::set_mext();

        // write csr: enable usb1 interrupts and events
        pac::csr::interrupt::enable(pac::Interrupt::USB1);
        pac::csr::interrupt::enable(pac::Interrupt::USB1_EP_CONTROL);
        pac::csr::interrupt::enable(pac::Interrupt::USB1_EP_IN);
        pac::csr::interrupt::enable(pac::Interrupt::USB1_EP_OUT);
        usb1.hal_driver.enable_interrupts();
    }

    info!("Peripherals initialized, entering main loop.");

    let mut max_queue_length = 0;
    let mut queue_length = 0;
    let mut start = false;

    let test_data = [0_u8; TEST_WRITE_SIZE];

    //let mut test_data = heapless::Vec::<u8, TEST_WRITE_SIZE>::new();
    //test_data.resize(TEST_WRITE_SIZE, 0).unwrap();

    loop {
        while let Some(message) = MESSAGE_QUEUE.dequeue() {
            match message {
                // - usb1 message handlers --

                // Usb1 received USB bus reset
                Message::UsbBusReset(1) => (),

                // Usb1 received setup packet
                Message::UsbReceiveSetupPacket(1, setup_packet) => {
                    start = false;
                    usb1.handle_setup_request(&setup_packet)
                        .map_err(|_| GreatError::BadMessage)?;
                }

                // Usb1 received data on endpoint
                Message::UsbReceiveData(1, endpoint, bytes_read, buffer) => {
                    match (endpoint, bytes_read, buffer[0]) {
                        (1, 1, 0x42) => {
                            info!("starting transmission");
                            start = true;
                        }
                        _ => {
                            info!("stopping transmission");
                            start = false;
                        }
                    }
                }

                // Usb1 transfer complete
                Message::UsbTransferComplete(1, endpoint) => {
                    leds.output.write(|w| unsafe { w.output().bits(0b10_0000) });
                }

                // Error Message
                Message::ErrorMessage(message) => {
                    error!("MachineExternal Error - {}", message);
                }

                // Unhandled message
                _ => {
                    error!("Unhandled message: {:?}", message);
                }
            }

            queue_length += 1;
        }

        // send test data as fast as we can
        if usb1.state() == DeviceState::Configured && start {
            // TODO hrmmm... btw 662 is 1001 cycles
            unsafe { riscv::asm::delay(1000) };

            leds.output.write(|w| unsafe { w.output().bits(0b00_0001) });
            usb1.hal_driver.write_ref(0x1, test_data.iter());
            leds.output.write(|w| unsafe { w.output().bits(0b00_0010) });
        }

        // queue diagnostics
        if queue_length > max_queue_length {
            max_queue_length = queue_length;
            debug!("max_queue_length: {}", max_queue_length);
        }
        queue_length = 0;
    }
}

// - usb descriptors ----------------------------------------------------------

static USB_DEVICE_DESCRIPTOR: DeviceDescriptor = DeviceDescriptor {
    descriptor_version: 0x0200,
    device_class: 0x00,
    device_subclass: 0x00,
    device_protocol: 0x00,
    max_packet_size: 64,
    vendor_id: 0x16d0,
    product_id: 0x0f3b,
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

static USB_CONFIGURATION_DESCRIPTOR_0: ConfigurationDescriptor = ConfigurationDescriptor::new(
    ConfigurationDescriptorHeader {
        configuration_value: 1,
        configuration_string_index: 1,
        attributes: 0x80, // 0b1000_0000 = bus-powered
        max_power: 50,    // 50 * 2 mA = 100 mA
        ..ConfigurationDescriptorHeader::new()
    },
    &[InterfaceDescriptor::new(
        InterfaceDescriptorHeader {
            interface_number: 0,
            alternate_setting: 0,
            interface_class: 0x00,
            interface_subclass: 0x00,
            interface_protocol: 0x00,
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
                endpoint_address: 0x81, // IN
                attributes: 0x02,       // Bulk
                max_packet_size: 512,
                interval: 0,
                ..EndpointDescriptor::new()
            },
        ],
    )],
);

static USB_OTHER_SPEED_CONFIGURATION_DESCRIPTOR_0: ConfigurationDescriptor =
    ConfigurationDescriptor::new(
        ConfigurationDescriptorHeader {
            descriptor_type: DescriptorType::OtherSpeedConfiguration as u8,
            configuration_value: 1,
            configuration_string_index: 1,
            attributes: 0x80, // 0b1000_0000 = bus-powered
            max_power: 50,    // 50 * 2 mA = 100 mA
            ..ConfigurationDescriptorHeader::new()
        },
        &[InterfaceDescriptor::new(
            InterfaceDescriptorHeader {
                interface_number: 0,
                alternate_setting: 0,
                interface_class: 0x00,
                interface_subclass: 0x00,
                interface_protocol: 0x00,
                interface_string_index: 2,
                ..InterfaceDescriptorHeader::new()
            },
            &[
                EndpointDescriptor {
                    endpoint_address: 0x01, // OUT
                    attributes: 0x02,       // Bulk
                    max_packet_size: 64,
                    interval: 0,
                    ..EndpointDescriptor::new()
                },
                EndpointDescriptor {
                    endpoint_address: 0x81, // IN
                    attributes: 0x02,       // Bulk
                    max_packet_size: 64,
                    interval: 0,
                    ..EndpointDescriptor::new()
                },
            ],
        )],
    );

static USB_STRING_DESCRIPTOR_0: StringDescriptorZero =
    StringDescriptorZero::new(&[LanguageId::EnglishUnitedStates]);

static USB_STRING_DESCRIPTOR_1: StringDescriptor = StringDescriptor::new("LUNA");
static USB_STRING_DESCRIPTOR_2: StringDescriptor = StringDescriptor::new("IN speed test");
static USB_STRING_DESCRIPTOR_3: StringDescriptor = StringDescriptor::new("no serial");

static USB_STRING_DESCRIPTORS: &[&StringDescriptor] = &[
    &USB_STRING_DESCRIPTOR_1,
    &USB_STRING_DESCRIPTOR_2,
    &USB_STRING_DESCRIPTOR_3,
];
