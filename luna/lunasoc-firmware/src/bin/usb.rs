#![feature(error_in_core)]
#![allow(dead_code, unused_variables)] // TODO
#![no_std]
#![no_main]

use panic_halt as _;
use riscv_rt::entry;

use firmware::hal;
use firmware::pac;
use lunasoc_firmware as firmware;

use core::fmt::Write;
use hal::Serial;

use hal::hal::delay::DelayUs;
use hal::Timer;

use firmware::usb::{UsbControlRequest, UsbControlRequestType, UsbInterface0, UsbSetupRequest};
use firmware::SYSTEM_CLOCK_FREQUENCY;
use firmware::{Error, Result};

static mut SERIAL: Serial = unsafe { Serial::summon() };

#[entry]
fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();

    let leds = &peripherals.LEDS;
    let mut timer = Timer::new(peripherals.TIMER, SYSTEM_CLOCK_FREQUENCY);

    let usb0 = UsbInterface0 {
        usb: peripherals.USB0,
        setup: peripherals.USB0_SETUP,
        ep0_in: peripherals.USB0_EP0_IN,
        ep0_out: peripherals.USB0_EP0_OUT,
    };

    leds.output.write(|w| unsafe { w.output().bits(0xff) });
    timer.delay_ms(2000).unwrap();

    //writeln!(unsafe { &mut SERIAL }, "Connecting USB device...").unwrap();
    usb0.usb.connect.write(|w| w.connect().bit(true));
    //writeln!(unsafe { &mut SERIAL }, "Connected.").unwrap();

    loop {
        leds.output.write(|w| unsafe { w.output().bits(0x00) });
        // read setup request
        let setup_request = match read_setup_request(&usb0) {
            Ok(setup_request) => setup_request,
            Err(e) => {
                writeln!(unsafe { &mut SERIAL }, "  Error {:?}", e).unwrap();
                loop {}
            }
        };

        // handle setup request
        match handle_setup_request(&usb0, &setup_request) {
            Ok(()) => {
                writeln!(unsafe { &mut SERIAL }, "  Ok: {:?}", setup_request).unwrap();
            }
            Err(e) => {
                writeln!(
                    unsafe { &mut SERIAL },
                    "  Error {:?}: {:?}",
                    e,
                    setup_request
                )
                    .unwrap();
                loop {}
            }
        };
        leds.output.write(|w| unsafe { w.output().bits(0xff) });
    }
}

// - read_setup_request -------------------------------------------------------

fn read_setup_request(usb0: &UsbInterface0) -> Result<UsbSetupRequest> {
    writeln!(unsafe { &mut SERIAL }, "\n=> read_setup_request()").unwrap();

    let mut buf = [0_u8; 8];
    for i in 0..7 {
        // block until setup data is available
        while !usb0.setup.have.read().have().bit() {}

        // read next byte
        buf[i] = usb0.setup.data.read().data().bits();
    }

    // Deserialize into a UsbSetupRequest in the most cursed manner available to us.
    let setup_request = unsafe { core::mem::transmute::<[u8; 8], UsbSetupRequest>(buf) };

    Ok(setup_request)
}

// - handle_setup_request -----------------------------------------------------

fn handle_setup_request(usb0: &UsbInterface0, setup_request: &UsbSetupRequest) -> Result<()> {
    writeln!(unsafe { &mut SERIAL }, "=> handle_setup_request()",).unwrap();

    // Extract the type (e.g. standard/class/vendor) from our SETUP request.
    let bits: u8 = (setup_request.request_type >> 5) & 0b0000_0011;
    let request_type = UsbControlRequestType::try_from(bits)?;

    // TODO: Get rid of this once we move to be fully compatible with ValentyUSB.
    //usb0.ep0_in.pid.write(|w| w.pid().bit(true));

    // If this isn't a standard request, stall it.
    if request_type != UsbControlRequestType::Standard {
        handle_unhandled_request(usb0, setup_request)?;
        return Err(Error::RequestTypeIsNotStandard);
    }

    // TODO if this fails it should stall
    let request = match UsbControlRequest::try_from(setup_request.request) {
        Ok(request) => request,
        Err(e) => {
            writeln!(unsafe { &mut SERIAL }, "OOPSIE",).unwrap();
            handle_unhandled_request(usb0, setup_request)?;
            return Err(e);
        }
    };

    match request {
        UsbControlRequest::SetAddress => handle_set_address(usb0, setup_request),
        UsbControlRequest::GetDescriptor => handle_get_descriptor(usb0, setup_request),
        UsbControlRequest::SetConfiguration => handle_set_configuration(usb0, setup_request),
        _ => {
            handle_unhandled_request(usb0, setup_request)?;
            Err(Error::RequestIsUnknown)
        }
    }
}

fn handle_set_address(usb0: &UsbInterface0, setup_request: &UsbSetupRequest) -> Result<()> {
    writeln!(unsafe { &mut SERIAL }, "  -> handle_set_address()").unwrap();

    usb0.ack_status_stage(setup_request);

    // TODO: UsbSetupRequest.value is u16 but register expects u8 - is this correct?
    let address: u8 = setup_request.value.try_into()?;

    usb0.setup
        .address
        .write(|w| unsafe { w.address().bits(address) });

    Ok(())
}

fn handle_get_descriptor(usb0: &UsbInterface0, setup_request: &UsbSetupRequest) -> Result<()> {
    writeln!(unsafe { &mut SERIAL }, "  -> handle_get_descriptor()").unwrap();

    let [descriptor_number, descriptor_type] = setup_request.value.to_le_bytes();
    let descriptor_type = UsbControlRequest::try_from(descriptor_type)?;

    match (descriptor_number, descriptor_type) {
        (_, UsbControlRequest::DescriptorDevice) => {
            usb0.send_control_response(setup_request, USB_DEVICE_DESCRIPTOR)
        }
        (0, UsbControlRequest::DescriptorConfiguration) => {
            usb0.send_control_response(setup_request, USB_CONFIG_DESCRIPTOR)
        }
        (0, UsbControlRequest::DescriptorString) => {
            usb0.send_control_response(setup_request, USB_STRING0_DESCRIPTOR)
        }
        (1, UsbControlRequest::DescriptorString) => {
            usb0.send_control_response(setup_request, USB_STRING1_DESCRIPTOR)
        }
        (2, UsbControlRequest::DescriptorString) => {
            usb0.send_control_response(setup_request, USB_STRING2_DESCRIPTOR)
        }
        _ => {
            handle_unhandled_request(usb0, setup_request)?;
            return Err(Error::DescriptorIsUnknown);
        }
    }

    Ok(())
}

fn handle_set_configuration(usb0: &UsbInterface0, setup_request: &UsbSetupRequest) -> Result<()> {
    writeln!(unsafe { &mut SERIAL }, "  -> handle_set_configuration()").unwrap();

    let configuration = setup_request.value;
    if configuration > 1 {
        handle_unhandled_request(usb0, setup_request)?;
        return Err(Error::ConfigurationIsUnknown);
    }

    usb0.ack_status_stage(setup_request);

    Ok(())
}

fn handle_unhandled_request(usb0: &UsbInterface0, setup_request: &UsbSetupRequest) -> Result<()> {
    writeln!(unsafe { &mut SERIAL }, "  -> handle_unhandled_request()",).unwrap();

    usb0.stall_request();

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

// - helpers ------------------------------------------------------------------

fn delay_ms(sys_clk: u32, ms: u32) {
    let cycles: u32 = sys_clk / 1_000 * ms;

    let peripherals = unsafe { pac::Peripherals::steal() };
    let timer = &peripherals.TIMER;

    timer.en.write(|w| w.en().bit(true));
    timer.reload.write(|w| unsafe { w.reload().bits(cycles) });

    while timer.ctr.read().ctr().bits() > 0 {
        unsafe {
            riscv::asm::nop();
        }
    }

    timer.en.write(|w| w.en().bit(false));
    timer.reload.write(|w| unsafe { w.reload().bits(0) });
}

fn uart_tx(string: &str) {
    let peripherals = unsafe { pac::Peripherals::steal() };
    let uart = &peripherals.UART;

    for c in string.chars() {
        while uart.tx_rdy.read().tx_rdy().bit() == false {
            unsafe {
                riscv::asm::nop();
            }
        }
        uart.tx_data.write(|w| unsafe { w.tx_data().bits(c as u8) })
    }
}
