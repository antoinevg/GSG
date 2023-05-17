#![allow(dead_code, unused_imports, unused_mut, unused_variables)] // TODO
#![no_std]
#![no_main]

//use riscv_atomic_emulation_trap as _;

use cynthion::{hal, pac, Message};

use hal::{smolusb, Serial};

use smolusb::control::{Direction, SetupPacket};
use smolusb::descriptor::*;
use smolusb::device::{DeviceState, Speed, UsbDevice};
use smolusb::traits::{
    ControlRead, EndpointRead, EndpointWrite, EndpointWriteRef, UnsafeUsbDriverOperations,
    UsbDriverOperations,
};

use libgreat::{GreatError, GreatResult};

use bbqueue::{BBBuffer, Producer};
use heapless::mpmc::MpMcQueue as Queue;
use log::{debug, error, info};

use riscv_rt::entry;

// - configuration ------------------------------------------------------------

const TEST_READ_SIZE: usize = 512;
const TEST_WRITE_SIZE: usize = 512;

// - global static state ------------------------------------------------------

static MESSAGE_QUEUE: Queue<Message, 32> = Queue::new();

// without: 38484
// 1 + 4 + 512 = ~ 517 bytes
struct ReceivePacket {
    pub port: cynthion::UsbInterface,
    pub endpoint: u8,
    pub bytes_read: usize,
    pub buffer: [u8; cynthion::EP_MAX_RECEIVE_LENGTH],
}

//  2: 38484 - 36988 = 1496 - 1034 = 462
//  4: 38484 - 35936 = 2548 - 2068 = 480
// 16: 38484 - 29644 = 8840 - 8272 = 568
static RECEIVE_PACKET_QUEUE: Queue<ReceivePacket, 64> = Queue::new();

// ~ 8K - 25244
//const USB_RECEIVE_BUFFER_SIZE: usize = cynthion::EP_MAX_ENDPOINTS * cynthion::EP_MAX_RECEIVE_LENGTH;
//static USB_RECEIVE_BUFFER: BBBuffer<USB_RECEIVE_BUFFER_SIZE> = BBBuffer::new();
//static mut USB_RECEIVE_BUFFER_PRODUCER: Option<Producer<USB_RECEIVE_BUFFER_SIZE>> = None;

// - MachineExternal interrupt handler ----------------------------------------

#[allow(non_snake_case)]
#[no_mangle]
fn MachineExternal() {
    use cynthion::UsbInterface::Aux;

    let usb1 = unsafe { hal::Usb1::summon() };
    let leds = unsafe { &pac::Peripherals::steal().LEDS };

    // - usb1 interrupts - "host_phy" / "aux_phy" --

    // USB1_EP_OUT UsbReceiveData
    if usb1.is_pending(pac::Interrupt::USB1_EP_OUT) {
        let endpoint = usb1.ep_out.data_ep.read().bits() as u8;

        // silently drain endpoint and discard packet
        if endpoint == 1 {
            //usb1.ep_out.reset.write(|w| w.reset().bit(true));
            while usb1.ep_out.have.read().have().bit() {
                let _b = usb1.ep_out.data.read().data().bits();
            }
            usb1.ep_out.epno.write(|w| unsafe { w.epno().bits(endpoint) });
            usb1.ep_out.prime.write(|w| w.prime().bit(true));
            usb1.ep_out.enable.write(|w| w.enable().bit(true));
            usb1.clear_pending(pac::Interrupt::USB1_EP_OUT);
            return;
        }

        let mut buffer = [0_u8; cynthion::EP_MAX_RECEIVE_LENGTH];
        let bytes_read = usb1.read(endpoint, &mut buffer);
        usb1.clear_pending(pac::Interrupt::USB1_EP_OUT);
        match RECEIVE_PACKET_QUEUE.enqueue(ReceivePacket {
            port: cynthion::UsbInterface::Aux,
            endpoint,
            bytes_read,
            buffer,
        }) {
            Ok(()) => {
                leds.output.write(|w| unsafe { w.output().bits(0b00_0001) });
            }
            Err(_) => {
                leds.output.write(|w| unsafe { w.output().bits(0b00_0010) });
                // stall endpoint ?
                // usb1.ep_out.epno.write(|w| unsafe { w.epno().bits(endpoint) });
                // usb1.ep_out.stall.write(|w| w.stall().bit(true));
                /*let message = Message::ErrorMessage("no space in received packet queue");
                match MESSAGE_QUEUE.enqueue(message) {
                    Ok(()) => (),
                    Err(_) => (),
                }*/
            }
        }

        return;
    }

    // USB1 UsbBusReset
    let message = if usb1.is_pending(pac::Interrupt::USB1) {
        usb1.clear_pending(pac::Interrupt::USB1);
        usb1.bus_reset();
        Message::UsbBusReset(Aux)

    // USB1_EP_CONTROL UsbReceiveSetupPacket
    } else if usb1.is_pending(pac::Interrupt::USB1_EP_CONTROL) {
        let mut setup_packet_buffer = [0_u8; 8];
        usb1.read_control(&mut setup_packet_buffer);
        usb1.clear_pending(pac::Interrupt::USB1_EP_CONTROL);

        match SetupPacket::try_from(setup_packet_buffer) {
            Ok(setup_packet) => Message::UsbReceiveSetupPacket(Aux, setup_packet),
            Err(e) => Message::ErrorMessage("USB1_EP_CONTROL failed to read setup packet"),
        }

    // USB1_EP_OUT UsbReceiveData
    /*} else if usb1.is_pending(pac::Interrupt::USB1_EP_OUT) {
    let endpoint = usb1.ep_out.data_ep.read().bits() as u8;
    if let Some(producer) = unsafe { USB_RECEIVE_BUFFER_PRODUCER.as_mut() } {
        match producer.grant_exact(cynthion::EP_MAX_RECEIVE_LENGTH) {
            Ok(mut grant) => {
                let bytes_read = usb1.read(endpoint, grant.buf());
                usb1.clear_pending(pac::Interrupt::USB1_EP_OUT);
                grant.commit(cynthion::EP_MAX_RECEIVE_LENGTH);
                Message::UsbReceiveData(Aux, endpoint, bytes_read)
            }
            Err(e) => {
                Message::ErrorMessage("no space in bbqueue")
            }
        }
    } else {
        Message::ErrorMessage("no bbqueue")
    }*/

    // USB1_EP_IN UsbTransferComplete
    } else if usb1.is_pending(pac::Interrupt::USB1_EP_IN) {
        usb1.clear_pending(pac::Interrupt::USB1_EP_IN);
        let endpoint = usb1.ep_in.epno.read().bits() as u8;

        // TODO something a little bit safer would be nice
        unsafe {
            usb1.clear_tx_ack_active();
        }

        Message::UsbTransferComplete(Aux, endpoint)

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
    /*let (producer, mut consumer) = USB_RECEIVE_BUFFER.try_split().unwrap();
    //let producer: bbqueue::Producer<USB_RECEIVE_BUFFER_SIZE> = producer;
    unsafe {
        USB_RECEIVE_BUFFER_PRODUCER = Some(producer);
    }*/

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

    let mut counter = 0;

    loop {
        let mut queue_length = 0;

        while let Some(receive_packet) = RECEIVE_PACKET_QUEUE.dequeue() {
            match receive_packet {
                ReceivePacket {
                    port: cynthion::UsbInterface::Aux,
                    endpoint: 0,
                    bytes_read,
                    buffer,
                } => {
                    info!("received {} bytes on endpoint 0x00", bytes_read);
                }
                ReceivePacket {
                    port: cynthion::UsbInterface::Aux,
                    endpoint: 1,
                    bytes_read,
                    buffer,
                } => {
                    /*if counter % 100 == 0 {
                        info!("received bulk data from host: {} bytes", bytes_read);
                        /*if bytes_read > 8 {
                            info!(
                                "{:?} .. {:?}",
                                &buffer[0..8],
                                &buffer[(bytes_read - 8)..bytes_read]
                            );
                        }*/
                    }
                    counter += 1;*/
                }
                ReceivePacket {
                    port: cynthion::UsbInterface::Aux,
                    endpoint: 2,
                    bytes_read,
                    buffer,
                } => {
                    info!("received command data from host: {} bytes", bytes_read);
                    let command = buffer[0].into();
                    match (bytes_read, &command) {
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
                        (1, command) => {
                            info!("stopping test: {:?}", command);
                            info!("  max write time: {}", test_stats.max_write_time);
                            info!("  min write time: {}", test_stats.min_write_time);
                            info!("  max flush time: {}", test_stats.max_flush_time);
                            info!("  min flush time: {}", test_stats.min_flush_time);
                            info!("  write count: {}", test_stats.write_count);
                            info!("  reset count: {}", test_stats.reset_count);
                            test_command = TestCommand::Stop;
                        }
                        (bytes_read, _) => {
                            error!(
                                "received invalid command from host: {:?} (read {} bytes)",
                                command,
                                bytes_read,
                            );
                        }
                    }
                }
                ReceivePacket { port, endpoint, bytes_read, buffer } => {
                    log::warn!("received unknown packet on {:?} endpoint: {}", port, endpoint);
                }
            }
        }

        if let Some(message) = MESSAGE_QUEUE.dequeue() {
            use cynthion::{Message::*, UsbInterface::Aux};

            match message {
                // - usb1 message handlers --

                // Usb1 received USB bus reset
                UsbBusReset(Aux) => (),

                // Usb1 received setup packet
                UsbReceiveSetupPacket(Aux, setup_packet) => {
                    test_command = TestCommand::Stop;
                    usb1.handle_setup_request(&setup_packet)
                        .map_err(|_| GreatError::BadMessage)?;
                }

                // TODO Usb1 received zero byte packet on endpoint 0x00 ???
                UsbReceiveData(Aux, 0x00, bytes_read) => {
                    info!("received {} bytes on endpoint 0x00", bytes_read);
                    /*match consumer.read() {
                        Ok(bbbuffer) => {
                            if bytes_read > 0 {
                                info!(
                                    "{:?} .. {:?}",
                                    &bbbuffer[0..8],
                                    &bbbuffer[(bytes_read - 8)..]
                                );
                            }
                            bbbuffer.release(cynthion::EP_MAX_RECEIVE_LENGTH);
                        }
                        Err(e) => {
                            error!("no bbqueue: {:?}", e);
                        }
                    }*/
                }

                // Usb1 received bulk test data on endpoint 0x01
                UsbReceiveData(Aux, 0x01, bytes_read) => {
                    counter += 1;
                    //if counter % 1000 == 0 {
                    info!("received bulk data from host: {} bytes", bytes_read);
                    //}

                    /*match consumer.read() {
                        Ok(bbbuffer) => {
                            info!(
                                "{:?} .. {:?}",
                                &bbbuffer[0..8],
                                &bbbuffer[(bytes_read - 8)..]
                            );
                            bbbuffer.release(cynthion::EP_MAX_RECEIVE_LENGTH);
                        }
                        Err(e) => {
                            error!("no bbqueue: {:?}", e);
                        }
                    }*/
                }

                // Usb1 received command data on endpoint 0x02
                UsbReceiveData(Aux, 0x02, bytes_read) => {
                    info!("received command data from host: {} bytes", bytes_read);
                    /*let command = match consumer.read() {
                        Ok(bbbuffer) => {
                            let bytes_read = cynthion::EP_MAX_RECEIVE_LENGTH;
                            info!(
                                "{:?} .. {:?}",
                                &bbbuffer[0..8],
                                &bbbuffer[(bytes_read - 8)..]
                            );
                            let command = bbbuffer[0].into();
                            bbbuffer.release(cynthion::EP_MAX_RECEIVE_LENGTH);
                            command
                        }
                        Err(e) => {
                            error!("no bbqueue: {:?}", e);
                            TestCommand::Error
                        }
                    };

                    match (bytes_read, &command) {
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
                        (1, command) => {
                            info!("stopping test: {:?}", command);
                            info!("  max write time: {}", test_stats.max_write_time);
                            info!("  min write time: {}", test_stats.min_write_time);
                            info!("  max flush time: {}", test_stats.max_flush_time);
                            info!("  min flush time: {}", test_stats.min_flush_time);
                            info!("  write count: {}", test_stats.write_count);
                            info!("  reset count: {}", test_stats.reset_count);
                            test_command = TestCommand::Stop;
                        }
                        (bytes_read, _) => {
                            error!(
                                "received invalid command from host: {:?} (read {} bytes)",
                                command,
                                bytes_read,
                            );
                        }
                    }*/
                }

                // Usb1 transfer complete
                UsbTransferComplete(Aux, endpoint) => {
                    //leds.output.write(|w| unsafe { w.output().bits(0b10_0000) });
                }

                // Error Message
                ErrorMessage(message) => {
                    //error!("MachineExternal Error - {}", message);
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
            TestCommand::Out => (), //test_out_speed(leds, &usb1.hal_driver, &mut test_stats),
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
    // Passing in a fixed size slice ref is 4MB/s vs 3.7MB/s
    #[inline(always)]
    fn test_write_slice(usb1: &hal::Usb1, endpoint: u8, data: &[u8; TEST_WRITE_SIZE]) -> bool {
        let mut did_reset = false;
        if usb1.ep_in.have.read().have().bit() {
            usb1.ep_in.reset.write(|w| w.reset().bit(true));
            did_reset = true;
        }
        // 5.005340640788884 MB/s
        for byte in data {
            usb1.ep_in.data.write(|w| unsafe { w.data().bits(*byte) });
        }
        // 5.507828119928235 MB/s - no memory access
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
}

/// Receive test data from host as fast as possible
#[inline(always)]
fn test_out_speed(leds: &pac::LEDS, usb1: &hal::Usb1, test_stats: &mut TestStats) {
}

// - types --------------------------------------------------------------------

#[derive(Debug, PartialEq)]
#[repr(u8)]
enum TestCommand {
    Stop,
    In = 0x23,
    Out = 0x42,
    Error = 0xff,
}

impl From<u8> for TestCommand {
    fn from(value: u8) -> Self {
        match value {
            0x23 => TestCommand::In,
            0x42 => TestCommand::Out,
            0xff => TestCommand::Error,
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
