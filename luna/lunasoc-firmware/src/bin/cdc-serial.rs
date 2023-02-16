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

use smolusb::class::cdc;
use smolusb::control::SetupPacket;
use smolusb::descriptor::{
    ConfigurationDescriptor, DescriptorType, DeviceDescriptor, DeviceQualifierDescriptor,
    EndpointDescriptor, InterfaceDescriptor, LanguageId, StringDescriptor, StringDescriptorZero,
};
use smolusb::device::{DeviceState, Speed, UsbDevice};
use smolusb::traits::{
    ControlRead, EndpointRead, EndpointWrite, EndpointWriteRef, UsbDriverOperations,
};

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
        &cdc::DEVICE_DESCRIPTOR,
        &cdc::CONFIGURATION_DESCRIPTOR_0,
        &cdc::USB_STRING_DESCRIPTOR_0,
        &cdc::USB_STRING_DESCRIPTORS,
    );
    let mut counter: usize = 1;
    let mut start_polling = false;

    loop {
        // send some data occasionally
        if start_polling && usb0_device.state == DeviceState::Configured && counter % 100_000 == 0 {
            // bulk out
            let endpoint = 2;
            let text = firmware::log::format!("Counter: {}\r\n", counter);
            let data: &[u8] = text.as_bytes();
            let bytes_written = data.len();
            usb0.write_ref(endpoint, data.into_iter());
            info!("Sent {} bytes to endpoint: {}", bytes_written, endpoint);
        }
        counter += 1;

        if let Some(message) = MESSAGE_QUEUE.dequeue() {
            match message {
                Message::ReceivedSetupPacket(packet) => {
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

                Message::ReceivedData(endpoint, bytes_read, buffer) => {
                    if endpoint != 0 {
                        debug!(
                            "Received {} bytes on endpoint: {} - {:?}\n",
                            bytes_read, endpoint, buffer
                        );
                        start_polling = true;
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
