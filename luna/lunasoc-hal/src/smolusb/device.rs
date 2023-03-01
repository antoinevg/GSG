#![allow(dead_code, unused_imports, unused_variables)] // TODO

use crate::smolusb::control::{Feature, Recipient, Request, RequestType, SetupPacket};
use crate::smolusb::descriptor::*;
//use crate::smolusb::error::{ErrorKind};
use crate::smolusb::traits::AsByteSliceIterator;
use crate::smolusb::traits::{
    ControlRead, EndpointRead, EndpointWrite, EndpointWriteRef, UsbDriverOperations,
};

use libgreat::error::Result;
use log::{debug, error, info, trace, warn};

///! `smolusb` device implementation for Luna USB peripheral
///!
///! TODO probably not all of this should live in the smolusb crate,
///! it should rather be split into generic and
///! implementation-specific parts

/// USB Speed
///
/// Note: These match the gateware peripheral so the mapping isn't particularly meaningful in other contexts.
#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum Speed {
    Low = 2,        // 1.5 Mbps
    Full = 1,       //  12 Mbps
    High = 0,       // 480 Mbps
    SuperSpeed = 3, // 5/10 Gbps (includes SuperSpeed+)
}

impl From<u8> for Speed {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0 => Speed::High,
            1 => Speed::Full,
            2 => Speed::Low,
            3 => Speed::SuperSpeed,
            _ => unimplemented!(),
        }
    }
}

/// USB device state
#[derive(Debug, PartialEq)]
pub enum DeviceState {
    Reset,
    Address,
    Configured,
    Suspend,
}

/// A USB device
///
/// `UsbDevice` implements the control portion of the USB
/// specification and consists of:
///
///     * a hal driver
///     * a device descriptor
///     * a configuration descriptor
///     * a set of string descriptors
///
pub struct UsbDevice<'a, D> {
    pub hal_driver: &'a D,
    device_descriptor: &'a DeviceDescriptor,
    //configuration_descriptor: &'a ConfigurationDescriptor<'a>,
    configuration_descriptor: ConfigurationDescriptor<'a>,
    string_descriptor_zero: &'a StringDescriptorZero<'a>,
    string_descriptors: &'a [&'a StringDescriptor<'a>],
    // TODO DeviceQualifierDescriptor
    // TODO OtherSpeedConfiguration
    pub state: DeviceState,
    pub reset_count: usize,
    pub feature_remote_wakeup: bool,

    pub cb_class_request:
        Option<fn(device: &UsbDevice<'a, D>, setup_packet: &SetupPacket, request: u8)>,
    pub cb_vendor_request:
        Option<fn(device: &UsbDevice<'a, D>, setup_packet: &SetupPacket, request: u8)>,
    pub cb_string_request:
        Option<fn(device: &UsbDevice<'a, D>, setup_packet: &SetupPacket, index: u8)>,
}

impl<'a, D> UsbDevice<'a, D>
where
    D: ControlRead + EndpointRead + EndpointWrite + EndpointWriteRef + UsbDriverOperations,
{
    pub fn new(
        hal_driver: &'a D,
        device_descriptor: &'a DeviceDescriptor,
        configuration_descriptor: &'a ConfigurationDescriptor<'a>,
        string_descriptor_zero: &'a StringDescriptorZero<'a>,
        string_descriptors: &'a [&'a StringDescriptor<'a>],
    ) -> Self {
        // Calculate and update descriptor length fields
        // TODO this ain't great but it will do for now
        let mut configuration_descriptor = configuration_descriptor.clone();
        let total_length = configuration_descriptor.set_total_length();
        debug!(
            "  ConfigurationDescriptor total length: {} bytes",
            total_length
        );

        Self {
            hal_driver,
            device_descriptor,
            configuration_descriptor,
            string_descriptor_zero,
            string_descriptors,
            state: DeviceState::Reset,
            reset_count: 0,
            feature_remote_wakeup: false,

            cb_class_request: None,
            cb_vendor_request: None,
            cb_string_request: None,
        }
    }
}

// Device functions
impl<'a, D> UsbDevice<'a, D>
where
    D: ControlRead + EndpointRead + EndpointWrite + EndpointWriteRef + UsbDriverOperations,
{
    pub fn connect(&mut self) -> Speed {
        self.hal_driver.connect().into()
    }

    pub fn reset(&mut self) -> Speed {
        let speed = self.hal_driver.reset().into();
        self.reset_count += 1;
        self.state = DeviceState::Reset;
        speed
    }
}

// Handle SETUP packet
impl<'a, D> UsbDevice<'a, D>
where
    D: ControlRead + EndpointRead + EndpointWrite + EndpointWriteRef + UsbDriverOperations,
{
    // TODO consider a `state: &dyn Any` variable for callbacks
    pub fn handle_setup_request(&mut self, setup_packet: &SetupPacket) -> Result<()> {
        debug!("# handle_setup_request()",);

        let request_type = setup_packet.request_type();
        let request = setup_packet.request();

        debug!(
            "  SETUP {:?} {:?} {:?} {:?} value:{} index:{} length:{}",
            setup_packet.recipient(),
            setup_packet.direction(),
            request_type,
            request,
            setup_packet.value,
            setup_packet.index,
            setup_packet.length
        );

        match (&request_type, &request) {
            (RequestType::Standard, Request::SetAddress) => {
                self.handle_set_address(setup_packet)?;
            }
            (RequestType::Standard, Request::GetDescriptor) => {
                self.handle_get_descriptor(setup_packet)?;
            }
            (RequestType::Standard, Request::SetConfiguration) => {
                self.handle_set_configuration(setup_packet)?;
            }
            (RequestType::Standard, Request::GetConfiguration) => {
                self.handle_get_configuration(setup_packet)?;
            }
            (RequestType::Standard, Request::ClearFeature) => {
                self.handle_clear_feature(setup_packet)?;
            }
            (RequestType::Standard, Request::SetFeature) => {
                self.handle_set_feature(setup_packet)?;
            }
            (RequestType::Class, Request::ClassOrVendor(request)) => {
                if let Some(cb) = self.cb_class_request {
                    cb(self, setup_packet, *request);
                } else {
                    warn!(
                        "   stall: unhandled class request {:?} {:?}",
                        request_type, request
                    );
                    self.hal_driver.stall_request();
                }
            }
            (RequestType::Vendor, Request::ClassOrVendor(request)) => {
                if let Some(cb) = self.cb_vendor_request {
                    cb(self, setup_packet, *request);
                } else {
                    warn!(
                        "   stall: unhandled class request {:?} {:?}",
                        request_type, request
                    );
                    self.hal_driver.stall_request();
                }
            }
            _ => {
                warn!(
                    "   stall: unhandled request {:?} {:?}",
                    request_type, request
                );
                self.hal_driver.stall_request();
            }
        }

        Ok(())
    }

    fn handle_set_address(&mut self, setup_packet: &SetupPacket) -> Result<()> {
        self.hal_driver.ack_status_stage(setup_packet);

        let address: u8 = (setup_packet.value & 0x7f) as u8;
        self.hal_driver.set_address(address);
        self.state = DeviceState::Address;

        debug!("  -> handle_set_address({})", address);

        Ok(())
    }

    fn handle_get_descriptor(&self, setup_packet: &SetupPacket) -> Result<()> {
        // extract the descriptor type and number from our SETUP request
        let [descriptor_number, descriptor_type_bits] = setup_packet.value.to_le_bytes();
        let descriptor_type = match DescriptorType::try_from(descriptor_type_bits) {
            Ok(descriptor_type) => descriptor_type,
            Err(e) => {
                warn!(
                    "   stall: invalid descriptor type: {} {}",
                    descriptor_type_bits, descriptor_number
                );
                self.hal_driver.stall_request();
                return Ok(());
            }
        };

        // if the host is requesting less than the maximum amount of data,
        // only respond with the amount requested
        let requested_length = setup_packet.length as usize;

        match (&descriptor_type, descriptor_number) {
            (DescriptorType::Device, 0) => self
                .hal_driver
                .write_ref(0, self.device_descriptor.as_iter().take(requested_length)),
            (DescriptorType::Configuration, 0) => self.hal_driver.write_ref(
                0,
                self.configuration_descriptor.iter().take(requested_length),
            ),
            // TODO
            //(DescriptorType::DeviceQualifier, 0) => {
            //    self.hal_driver.ep_in_write(0, self.device_qualifier_descriptor.into_iter().take(requested_length))
            //}
            //(DescriptorType::OtherSpeedConfiguration, 0) => {
            //    self.hal_driver.ep_in_write(0, self.other_speed_config_descriptor.iter().take(requested_length))
            //}
            (DescriptorType::String, 0) => self
                .hal_driver
                .write_ref(0, self.string_descriptor_zero.iter().take(requested_length)),
            (DescriptorType::String, index) => {
                let offset_index: usize = (index - 1).into();

                if offset_index > self.string_descriptors.len() {
                    if let Some(cb) = self.cb_string_request {
                        cb(self, setup_packet, index);
                    } else {
                        warn!("   stall: unknown string descriptor {}", index);
                        self.hal_driver.stall_request();
                    }
                    return Ok(());
                }

                self.hal_driver.write(
                    0,
                    self.string_descriptors[offset_index]
                        .iter()
                        .take(requested_length),
                )
            }
            _ => {
                warn!(
                    "   stall: unhandled descriptor {:?}, {}",
                    descriptor_type, descriptor_number
                );
                self.hal_driver.stall_request();
                return Ok(());
            }
        }

        self.hal_driver.ack_status_stage(setup_packet);

        debug!(
            "  -> handle_get_descriptor({:?}({}), {}, {})",
            descriptor_type, descriptor_type_bits, descriptor_number, requested_length
        );

        Ok(())
    }

    fn handle_set_configuration(&mut self, setup_packet: &SetupPacket) -> Result<()> {
        self.hal_driver.ack_status_stage(setup_packet);

        debug!("  -> handle_set_configuration()");

        let configuration = setup_packet.value;
        if configuration > 1 {
            warn!("   stall: unknown configuration {}", configuration);
            self.hal_driver.stall_request();
            return Ok(());
        }
        self.state = DeviceState::Configured;

        Ok(())
    }

    fn handle_get_configuration(&self, setup_packet: &SetupPacket) -> Result<()> {
        debug!("  -> handle_get_configuration()");

        let requested_length = setup_packet.length as usize;

        self.hal_driver
            .write(0, [1].into_iter().take(requested_length));
        self.hal_driver.ack_status_stage(setup_packet);

        Ok(())
    }

    fn handle_clear_feature(&mut self, setup_packet: &SetupPacket) -> Result<()> {
        debug!("  -> handle_clear_feature()");

        // parse request
        let recipient = setup_packet.recipient();
        let feature_bits = setup_packet.value;
        let feature = match Feature::try_from(feature_bits) {
            Ok(feature) => feature,
            Err(e) => {
                warn!("   stall: invalid clear feature type: {}", feature_bits);
                self.hal_driver.stall_request();
                return Ok(());
            }
        };

        match (&recipient, &feature) {
            (Recipient::Device, Feature::DeviceRemoteWakeup) => {
                self.feature_remote_wakeup = false;
            }
            (Recipient::Endpoint, Feature::EndpointHalt) => {
                let endpoint = setup_packet.index as u8;
                self.hal_driver.stall_endpoint(endpoint, false);
                //debug!("  clear stall: 0x{:x}", endpoint);

                // queue a little test data on interrupt endpoint
                if endpoint == 0x82 {
                    let endpoint = endpoint - 0x80;
                    //self.hal_driver.ack(endpoint, packet);
                    /*const SIZE: usize = 8;
                    let data: heapless::Vec<u8, SIZE> =
                        (0..(SIZE as u8)).collect::<heapless::Vec<u8, SIZE>>().try_into().unwrap();
                    let bytes_written = data.len();
                    self.hal_driver.write(endpoint, data.into_iter());
                    info!(
                        "Sent {} bytes to interrupt endpoint: {}",
                        bytes_written, endpoint
                    );*/
                    //self.hal_driver.write(endpoint, [].into_iter());
                }
            }
            _ => {
                warn!(
                    "   stall: unhandled clear feature {:?}, {:?}",
                    recipient, feature
                );
                self.hal_driver.stall_request();
                return Ok(());
            }
        };

        Ok(())
    }

    fn handle_set_feature(&mut self, setup_packet: &SetupPacket) -> Result<()> {
        debug!("  -> handle_set_feature()");

        // parse request
        let recipient = setup_packet.recipient();
        let feature_bits = setup_packet.value;
        let feature = match Feature::try_from(feature_bits) {
            Ok(feature) => feature,
            Err(e) => {
                warn!("   stall: invalid set feature type: {}", feature_bits);
                self.hal_driver.stall_request();
                return Ok(());
            }
        };

        match (&recipient, &feature) {
            (Recipient::Device, Feature::DeviceRemoteWakeup) => {
                self.feature_remote_wakeup = true;
            }
            (Recipient::Endpoint, Feature::EndpointHalt) => {
                let endpoint = setup_packet.index as u8;
                self.hal_driver.stall_endpoint(endpoint, true);
                debug!("  set stall: 0x{:x}", endpoint);
            }
            _ => {
                warn!(
                    "   stall: unhandled set feature {:?}, {:?}",
                    recipient, feature
                );
                self.hal_driver.stall_request();
                return Ok(());
            }
        };

        Ok(())
    }
}

// TODO I'm not convinced about any of this
impl<'a, D> UsbDevice<'a, D>
where
    D: ControlRead + EndpointRead + EndpointWrite + EndpointWriteRef,
{
    pub fn _handle_interrupt_ep_control(hal_driver: &D) -> Result<SetupPacket> {
        let mut buffer = [0_u8; 8];
        hal_driver.read_control(&mut buffer);
        SetupPacket::try_from(buffer)
    }

    pub fn _handle_interrupt_ep_out(
        &self,
        hal_driver: &D,
        endpoint: u8,
        buffer: &mut [u8],
    ) -> usize {
        hal_driver.read(endpoint, buffer)
    }
}

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
