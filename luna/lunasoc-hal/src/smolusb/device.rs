#![allow(dead_code, unused_imports, unused_variables)] // TODO

use crate::smolusb::control::{Request, RequestType, SetupPacket};
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
    hal_driver: &'a D,
    device_descriptor: &'a DeviceDescriptor,
    configuration_descriptor: &'a ConfigurationDescriptor<'a>,
    string_descriptor_zero: &'a StringDescriptorZero<'a>,
    string_descriptors: &'a [&'a StringDescriptor<'a>],
    // TODO DeviceQualifierDescriptor
    // TODO OtherSpeedConfiguration
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
        Self {
            hal_driver,
            device_descriptor,
            configuration_descriptor,
            string_descriptor_zero,
            string_descriptors,
        }
    }

    pub fn handle_setup_request(&self, setup_packet: &SetupPacket) -> Result<()> {
        debug!("# handle_setup_request()",);

        // if this isn't a standard request, stall it.
        if setup_packet.request_type() != RequestType::Standard {
            warn!(
                "   stall: unsupported request type {:?}",
                setup_packet.request_type
            );
            self.hal_driver.stall_request();
            return Ok(());
        }

        // extract the request
        let request = match setup_packet.request() {
            Ok(request) => request,
            Err(e) => {
                warn!(
                    "   stall: unsupported request {}: {:?}",
                    setup_packet.request, e
                );
                self.hal_driver.stall_request();
                return Ok(());
            }
        };

        debug!(
            "  dispatch: {:?} {:?} {:?} {}, {}",
            setup_packet.recipient(),
            setup_packet.direction(),
            request,
            setup_packet.value,
            setup_packet.length
        );

        match request {
            Request::SetAddress => self.handle_set_address(setup_packet),
            Request::GetDescriptor => self.handle_get_descriptor(setup_packet),
            Request::SetConfiguration => self.handle_set_configuration(setup_packet),
            Request::GetConfiguration => self.handle_get_configuration(setup_packet),
            Request::ClearFeature => self.handle_clear_feature(setup_packet),
            _ => {
                warn!("   stall: unhandled request {:?}", request);
                self.hal_driver.stall_request();
                Ok(())
            }
        }
    }

    fn handle_set_address(&self, packet: &SetupPacket) -> Result<()> {
        self.hal_driver.ack_status_stage(packet);

        let address: u8 = (packet.value & 0x7f) as u8;
        self.hal_driver.set_address(address);

        debug!("  -> handle_set_address({})", address);

        Ok(())
    }

    fn handle_get_descriptor(&self, packet: &SetupPacket) -> Result<()> {
        // extract the descriptor type and number from our SETUP request
        let [descriptor_number, descriptor_type_bits] = packet.value.to_le_bytes();
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
        let requested_length = packet.length as usize;

        match (&descriptor_type, descriptor_number) {
            (DescriptorType::Device, 0) => self
                .hal_driver
                .write_ref(0, self.device_descriptor.as_iter().take(requested_length)),
            (DescriptorType::Configuration, 0) => self.hal_driver.write(
                0,
                self.configuration_descriptor.iter().take(requested_length),
            ),
            //(DescriptorType::DeviceQualifier, 0) => {
            //    self.hal_driver.ep_in_write(0, self.device_qualifier_descriptor.into_iter().take(requested_length))
            //}
            //(DescriptorType::OtherSpeedConfiguration, 0) => {
            //    self.hal_driver.ep_in_write(0, self.other_speed_config_descriptor.iter().take(requested_length))
            //}
            (DescriptorType::String, 0) => self
                .hal_driver
                .write(0, self.string_descriptor_zero.iter().take(requested_length)),
            (DescriptorType::String, index) => {
                let index: usize = (index - 1).into();
                if index > self.string_descriptors.len() {
                    warn!("   stall: unknown string descriptor {}", index);
                    self.hal_driver.stall_request();
                    return Ok(());
                }
                self.hal_driver.write(
                    0,
                    self.string_descriptors[index].iter().take(requested_length),
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

        self.hal_driver.ack_status_stage(packet);

        debug!(
            "  -> handle_get_descriptor({:?}({}), {}, {})",
            descriptor_type, descriptor_type_bits, descriptor_number, requested_length
        );

        Ok(())
    }

    fn handle_set_configuration(&self, packet: &SetupPacket) -> Result<()> {
        self.hal_driver.ack_status_stage(packet);

        debug!("  -> handle_set_configuration()");

        let configuration = packet.value;
        if configuration > 1 {
            warn!("   stall: unknown configuration {}", configuration);
            self.hal_driver.stall_request();
            return Ok(());
        }

        Ok(())
    }

    fn handle_get_configuration(&self, packet: &SetupPacket) -> Result<()> {
        debug!("  -> handle_get_configuration()");

        let requested_length = packet.length as usize;

        self.hal_driver
            .write(0, [1].into_iter().take(requested_length));
        self.hal_driver.ack_status_stage(packet);

        Ok(())
    }

    fn handle_clear_feature(&self, packet: &SetupPacket) -> Result<()> {
        debug!("  -> handle_clear_feature()");

        // TODO

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
