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

#[cfg(feature = "vexriscv")]
#[riscv_rt::pre_init]
unsafe fn pre_main() {
    pac::cpu::vexriscv::flush_icache();
    #[cfg(feature = "vexriscv_dcache")]
    pac::cpu::vexriscv::flush_dcache();
}

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

    let mut test_command = TestCommand::Stop;
    let mut test_stats = TestStats::new();

    // 4 MB/s
    let test_data = {
        let mut test_data = [0_u8; TEST_WRITE_SIZE];
        for n in 0..TEST_WRITE_SIZE {
            test_data[n] = (n % 256) as u8;
        }
        test_data
    };

    // 3.7 MB/s
    //let test_data: heapless::Vec<u8, TEST_WRITE_SIZE> =
    //    (0..TEST_WRITE_SIZE).map(|x| (x % 256) as u8).collect();

    loop {
        let mut queue_length = 0;
        while let Some(message) = MESSAGE_QUEUE.dequeue() {
            match message {
                // - usb1 message handlers --

                // Usb1 received USB bus reset
                Message::UsbBusReset(1) => (),

                // Usb1 received setup packet
                Message::UsbReceiveSetupPacket(1, setup_packet) => {
                    test_command = TestCommand::Stop;
                    usb1.handle_setup_request(&setup_packet)
                        .map_err(|_| GreatError::BadMessage)?;
                }

                // TODO Usb1 received zero byte packet on endpoint 0x00 ???
                Message::UsbReceiveData(1, 0x00, bytes_read, buffer) => {
                    info!("received {} bytes on endpoint 0x00", bytes_read);
                }

                // Usb1 received bulk test data on endpoint 0x01
                Message::UsbReceiveData(1, 0x01, bytes_read, buffer) => {
                    info!("received bulk data from host: {} bytes", bytes_read);
                }

                // Usb1 received command data on endpoint 0x02
                Message::UsbReceiveData(1, 0x02, bytes_read, buffer) => {
                    match (bytes_read, buffer[0].into()) {
                        (1, TestCommand::In) => {
                            info!("starting test: IN");
                            test_stats.reset();
                            test_command = TestCommand::In;
                        }
                        (1, TestCommand::Out) => {
                            info!("starting test: OUT");
                            test_stats.reset();
                            test_command = TestCommand::Out;
                        }
                        (1, _) => {
                            info!("stopping test");
                            info!("  max write time: {}", test_stats.max_write_time);
                            info!("  min write time: {}", test_stats.min_write_time);
                            info!("  max flush time: {}", test_stats.max_flush_time);
                            info!("  min flush time: {}", test_stats.min_flush_time);
                            info!("  write count: {}", test_stats.write_count);
                            info!("  reset count: {}", test_stats.reset_count);
                            test_command = TestCommand::Stop;
                        }
                        _ => {
                            error!(
                                "received invalid command from host: {:x?}",
                                &buffer[0..bytes_read]
                            );
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

        // perform tests
        match test_command {
            TestCommand::In => test_in_speed(leds, &usb1.hal_driver, &test_data, &mut test_stats),
            TestCommand::Out => test_out_speed(leds, &usb1.hal_driver, &mut test_stats),
            _ => (),
        }

        // queue diagnostics
        if queue_length > test_stats.max_queue_length {
            test_stats.max_queue_length = queue_length;
            debug!("max_queue_length: {}", test_stats.max_queue_length);
        }
    }
}

// - tests --------------------------------------------------------------------

/// Send test data to host as fast as possible
#[inline(always)]
fn test_in_speed(
    leds: &pac::LEDS,
    usb1: &hal::Usb1,
    test_data: &[u8; TEST_WRITE_SIZE],
    test_stats: &mut TestStats,
) {
    leds.output.write(|w| unsafe { w.output().bits(0b00_0001) });

    // Passing in a fixed size slice ref is 4MB/s vs 3.7MB/s
    #[inline(always)]
    fn test_write_slice(usb1: &hal::Usb1, endpoint: u8, data: &[u8; TEST_WRITE_SIZE]) -> bool {
        let mut did_reset = false;
        if usb1.ep_in.have.read().have().bit() {
            usb1.ep_in.reset.write(|w| w.reset().bit(true));
            did_reset = true;
        }
        // 4.00309544825905 MB/s
        for byte in data {
            usb1.ep_in.data.write(|w| unsafe { w.data().bits(*byte) });
        }
        // 5.063017280948139 MB/s - no memory access
        /*for n in 0..TEST_WRITE_SIZE {
            usb1.ep_in.data.write(|w| unsafe { w.data().bits((n % 256) as u8) });
        }*/
        usb1.ep_in
            .epno
            .write(|w| unsafe { w.epno().bits(endpoint & 0xf) });
        did_reset
    }

    // wait for fifo endpoint to be idle
    let (_, t_flush) = cynthion::profile!(
        let mut timeout = 100;
        while !usb1.ep_in.idle.read().idle().bit() && timeout > 0 {
            leds.output.write(|w| unsafe { w.output().bits(0b00_0010) });
            timeout -= 1;
        }
    );

    // write data to endpoint fifo
    let (did_reset, t_write) = cynthion::profile!(
        //usb1.write(0x1, test_data.into_iter().copied()); false // 6780 / 5653 ~3.99MB/s
        //usb1.write_ref(0x1, test_data.iter()); false // 5663 / 5652 - ~4.02MB/s
        test_write_slice(usb1, 0x1, test_data) // 56533 / 5652 - ~4.04MB/s
    );
    test_stats.write_count += 1;

    // update stats
    test_stats.update_in(t_write, t_flush, did_reset);

    leds.output.write(|w| unsafe { w.output().bits(0b00_0100) });
}

/// Receive test data from host as fast as possible
#[inline(always)]
fn test_out_speed(leds: &pac::LEDS, usb1: &hal::Usb1, test_stats: &mut TestStats) {
    leds.output.write(|w| unsafe { w.output().bits(0b00_0001) });
    leds.output.write(|w| unsafe { w.output().bits(0b00_0100) });
}

// - types --------------------------------------------------------------------

#[derive(PartialEq)]
#[repr(u8)]
enum TestCommand {
    Stop,
    In = 0x23,
    Out = 0x42,
}

impl From<u8> for TestCommand {
    fn from(value: u8) -> Self {
        match value {
            0x23 => TestCommand::In,
            0x42 => TestCommand::Out,
            _ => TestCommand::Stop,
        }
    }
}

struct TestStats {
    max_queue_length: usize,

    max_write_time: usize,
    min_write_time: usize,
    max_flush_time: usize,
    min_flush_time: usize,

    write_count: usize,
    reset_count: usize,
}

impl TestStats {
    const fn new() -> Self {
        Self {
            max_queue_length: 0,
            max_write_time: 0,
            min_write_time: usize::MAX,
            max_flush_time: 0,
            min_flush_time: usize::MAX,
            write_count: 0,
            reset_count: 0,
        }
    }

    fn reset(&mut self) {
        *self = Self::new();
    }

    #[inline(always)]
    fn update_in(&mut self, t_write: usize, t_flush: usize, did_reset: bool) {
        if t_write > self.max_write_time {
            self.max_write_time = t_write;
        }
        if t_write < self.min_write_time {
            self.min_write_time = t_write;
        }
        if t_flush > self.max_flush_time {
            self.max_flush_time = t_flush;
        }
        if t_flush < self.min_flush_time {
            self.min_flush_time = t_flush;
        }
        if did_reset {
            self.reset_count += 1;
        }
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
                endpoint_address: 0x02, // OUT - host commands
                attributes: 0x02,       // Bulk
                max_packet_size: 8,
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
                    endpoint_address: 0x02, // OUT - host commands
                    attributes: 0x02,       // Bulk
                    max_packet_size: 8,
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
