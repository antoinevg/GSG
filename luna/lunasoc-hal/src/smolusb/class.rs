//! USB device and interface classes

pub mod cdc {
    use crate::smolusb::descriptor::*;

    pub mod ch34x {
        #[derive(Debug, PartialEq)]
        #[repr(u8)]
        pub enum VendorRequest {
            WriteType = 0x40,  //  64
            ReadType = 0xc0,   // 192
            Read = 0x95,       // 149
            Write = 0x9a,      // 154
            SerialInit = 0xa1, // 161
            ModemOut = 0xa4,   // 164
            Version = 0x5f,    //  95
        }
    }

    pub static DEVICE_DESCRIPTOR: DeviceDescriptor = DeviceDescriptor {
        descriptor_version: 0x0200,
        device_class: 0xff,    // Vendor-specific
        device_subclass: 0x00, // Vendor-specific
        device_protocol: 0x00,
        max_packet_size: 8,
        vendor_id: 0x1a86,
        product_id: 0x7523,
        device_version_number: 0x0264,
        manufacturer_string_index: 1,
        product_string_index: 2,
        serial_string_index: 3,
        num_configurations: 1,
        ..DeviceDescriptor::new()
    };

    pub static CONFIGURATION_DESCRIPTOR_0: ConfigurationDescriptor = ConfigurationDescriptor {
        _length: 0,
        descriptor_type: DescriptorType::Configuration, // TODO
        _total_length: 0,
        _num_interfaces: 0,
        configuration_value: 1,
        configuration_string_index: 1,
        attributes: 0x80, // 0b1000_0000 = bus-powered
        max_power: 50,    // 50 * 2 mA = 100 mA
        interface_descriptors: &[&INTERFACE_DESCRIPTOR_0],
    };

    pub static INTERFACE_DESCRIPTOR_0: InterfaceDescriptor = InterfaceDescriptor {
        _length: 0,
        _descriptor_type: DescriptorType::Interface as u8,
        interface_number: 0,
        alternate_setting: 0,
        _num_endpoints: 0,
        interface_class: 0xff,    // Vendor-specific
        interface_subclass: 0x01, // Vendor-specific
        interface_protocol: 0x02, // CDC
        interface_string_index: 2,
        endpoint_descriptors: &[
            &ENDPOINT_DESCRIPTOR_82,
            &ENDPOINT_DESCRIPTOR_02,
            &ENDPOINT_DESCRIPTOR_81,
        ],
    };

    pub static ENDPOINT_DESCRIPTOR_82: EndpointDescriptor = EndpointDescriptor {
        _length: 7,
        _descriptor_type: DescriptorType::Endpoint as u8,
        endpoint_address: 0x82, // IN
        attributes: 0x02,       // Bulk
        max_packet_size: 512,   // technically 32
        interval: 0,
    };

    pub static ENDPOINT_DESCRIPTOR_02: EndpointDescriptor = EndpointDescriptor {
        _length: 7,
        _descriptor_type: DescriptorType::Endpoint as u8,
        endpoint_address: 0x02, // OUT
        attributes: 0x02,       // Bulk
        max_packet_size: 512,   // technically 32
        interval: 0,
    };

    pub static ENDPOINT_DESCRIPTOR_81: EndpointDescriptor = EndpointDescriptor {
        _length: 7,
        _descriptor_type: DescriptorType::Endpoint as u8,
        endpoint_address: 0x81, // IN
        attributes: 0x03,       // Interrupt
        max_packet_size: 8,
        interval: 1, // 1ms
    };

    pub static USB_STRING_DESCRIPTOR_0: StringDescriptorZero = StringDescriptorZero {
        _length: 10,
        _descriptor_type: DescriptorType::String as u8,
        language_ids: &[LanguageId::EnglishUnitedStates],
    };

    pub static USB_STRING_DESCRIPTOR_1: StringDescriptor =
        StringDescriptor::new("Great Scott Gadgets");
    pub static USB_STRING_DESCRIPTOR_2: StringDescriptor =
        StringDescriptor::new("USB Serial Emulation");
    pub static USB_STRING_DESCRIPTOR_3: StringDescriptor = StringDescriptor::new("v1.0");
    pub static USB_STRING_DESCRIPTORS: &[&StringDescriptor] = &[
        &USB_STRING_DESCRIPTOR_1,
        &USB_STRING_DESCRIPTOR_2,
        &USB_STRING_DESCRIPTOR_3,
    ];
}
