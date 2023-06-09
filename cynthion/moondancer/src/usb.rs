#![allow(dead_code, unused_variables)] // TODO

use smolusb::descriptor::*;

pub mod vendor {
    #[repr(u8)]
    #[derive(Debug, PartialEq)]
    pub enum VendorRequest {
        // libgreat/firmware/platform/lpc43xx/include/drivers/usb/comms_backend.h
        //   11:  #define LIBGREAT_USB_COMMAND_REQUEST 0x65
        // libgreat/host/pygreat/comms_backends/usb.py
        //   30:  LIBGREAT_REQUEST_NUMBER = 0x65
        UsbCommandRequest = 0x65, // 101
        Unknown(u8),
    }

    impl From<u8> for VendorRequest {
        fn from(value: u8) -> Self {
            match value {
                0x65 => VendorRequest::UsbCommandRequest,
                _ => VendorRequest::Unknown(value),
            }
        }
    }

    #[repr(u16)]
    #[derive(Debug, PartialEq)]
    pub enum VendorRequestValue {
        Start = 0x0000,
        Cancel = 0xdead,
        Unknown(u16),
    }

    impl From<u16> for VendorRequestValue {
        fn from(value: u16) -> Self {
            match value {
                0x0000 => VendorRequestValue::Start,
                0xdead => VendorRequestValue::Cancel,
                _ => VendorRequestValue::Unknown(value),
            }
        }
    }
}

pub static DEVICE_DESCRIPTOR: DeviceDescriptor = DeviceDescriptor {
    descriptor_version: 0x0200,
    device_class: 0x00,    // Composite
    device_subclass: 0x00, // Composite
    device_protocol: 0x00, // Composite
    max_packet_size: 64,
    vendor_id: 0x1d50,             // OpenMoko, Inc.
    product_id: 0x60e6,            // replacement for GoodFET/FaceDancer - GreatFet
    device_version_number: 0x0040, // Cynthion r04
    manufacturer_string_index: 1,
    product_string_index: 2,
    serial_string_index: 3,
    num_configurations: 1,
    ..DeviceDescriptor::new()
};

pub static DEVICE_QUALIFIER_DESCRIPTOR: DeviceQualifierDescriptor = DeviceQualifierDescriptor {
    descriptor_version: 0x0200,
    device_class: 0x00,    // Composite
    device_subclass: 0x00, // Composite
    device_protocol: 0x00, // Composite
    max_packet_size: 64,
    num_configurations: 1,
    ..DeviceQualifierDescriptor::new()
};

pub static CONFIGURATION_DESCRIPTOR_0: ConfigurationDescriptor = ConfigurationDescriptor::new(
    ConfigurationDescriptorHeader {
        descriptor_type: DescriptorType::Configuration as u8,
        configuration_value: 1,
        configuration_string_index: 1,
        attributes: 0x80, // 0b1000_0000 = bus-powered
        max_power: 250,   // 250 * 2 mA = 500 mA ?
        ..ConfigurationDescriptorHeader::new()
    },
    &[
        InterfaceDescriptor::new(
            InterfaceDescriptorHeader {
                interface_number: 0,
                alternate_setting: 0,
                interface_class: 0xff,    // Vendor-specific
                interface_subclass: 0xff, // Vendor-specific
                interface_protocol: 0xff, // Vendor-specific
                interface_string_index: 2,
                ..InterfaceDescriptorHeader::new()
            },
            &[],
        ),
        InterfaceDescriptor::new(
            InterfaceDescriptorHeader {
                interface_number: 1,
                alternate_setting: 0,
                interface_class: 0xff,    // Vendor-specific
                interface_subclass: 0xff, // Vendor-specific
                interface_protocol: 0xff, // Vendor-specific
                interface_string_index: 2,
                ..InterfaceDescriptorHeader::new()
            },
            &[
                EndpointDescriptor {
                    endpoint_address: 0x81, // IN
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
            ],
        ),
    ],
);

pub static OTHER_SPEED_CONFIGURATION_DESCRIPTOR_0: ConfigurationDescriptor =
    ConfigurationDescriptor::new(
        ConfigurationDescriptorHeader {
            descriptor_type: DescriptorType::OtherSpeedConfiguration as u8,
            configuration_value: 1,
            configuration_string_index: 1,
            attributes: 0x80, // 0b1000_0000 = bus-powered
            max_power: 250,   // 250 * 2 mA = 500 mA ?
            ..ConfigurationDescriptorHeader::new()
        },
        &[
            InterfaceDescriptor::new(
                InterfaceDescriptorHeader {
                    interface_number: 0,
                    alternate_setting: 0,
                    interface_class: 0xff,    // Vendor-specific
                    interface_subclass: 0xff, // Vendor-specific
                    interface_protocol: 0xff, // Vendor-specific
                    interface_string_index: 2,
                    ..InterfaceDescriptorHeader::new()
                },
                &[],
            ),
            InterfaceDescriptor::new(
                InterfaceDescriptorHeader {
                    interface_number: 1,
                    alternate_setting: 0,
                    interface_class: 0xff,    // Vendor-specific
                    interface_subclass: 0xff, // Vendor-specific
                    interface_protocol: 0xff, // Vendor-specific
                    interface_string_index: 2,
                    ..InterfaceDescriptorHeader::new()
                },
                &[
                    EndpointDescriptor {
                        endpoint_address: 0x81, // IN
                        attributes: 0x02,       // Bulk
                        max_packet_size: 64,
                        interval: 0,
                        ..EndpointDescriptor::new()
                    },
                    EndpointDescriptor {
                        endpoint_address: 0x02, // OUT
                        attributes: 0x02,       // Bulk
                        max_packet_size: 64,
                        interval: 0,
                        ..EndpointDescriptor::new()
                    },
                ],
            ),
        ],
    );

pub static USB_STRING_DESCRIPTOR_0: StringDescriptorZero =
    StringDescriptorZero::new(&[LanguageId::EnglishUnitedStates]);

pub static USB_STRING_DESCRIPTOR_1: StringDescriptor = StringDescriptor::new("Great Scott Gadgets");
pub static USB_STRING_DESCRIPTOR_2: StringDescriptor = StringDescriptor::new("Moondancer");
pub static USB_STRING_DESCRIPTOR_3: StringDescriptor = StringDescriptor::new("040");

pub static USB_STRING_DESCRIPTORS: &[&StringDescriptor] = &[
    &USB_STRING_DESCRIPTOR_1,
    &USB_STRING_DESCRIPTOR_2,
    &USB_STRING_DESCRIPTOR_3,
];
