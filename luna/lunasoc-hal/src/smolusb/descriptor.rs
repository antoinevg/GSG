///! USB Descriptor types
use crate::smolusb::ErrorKind;


/// DescriptorType
#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum DescriptorType {
    Device = 1,
    Configuration = 2,
    String = 3,
    Interface = 4,
    Endpoint = 5,
    DeviceQualifier = 6,
    OtherSpeedConfiguration = 7,
    InterfacePower = 8,
    OnTheGo = 9,
    Debug = 10,
    InterfaceAssociation = 11,
    Security = 12,
    Key = 13,
    EncryptionType = 14,
    BinaryDeviceObjectStore = 15,
    DeviceCapability = 16,
    WirelessEndpointCompanion = 17,
    SuperSpeedEndpointCompanion = 48,
}

impl TryFrom<u8> for DescriptorType {
    type Error = ErrorKind;

    fn try_from(value: u8) -> core::result::Result<Self, Self::Error> {
        let result = match value {
            1 => DescriptorType::Device,
            2 => DescriptorType::Configuration,
            3 => DescriptorType::String,
            4 => DescriptorType::Interface,
            5 => DescriptorType::Endpoint,
            6 => DescriptorType::DeviceQualifier,
            7 => DescriptorType::OtherSpeedConfiguration,
            8 => DescriptorType::InterfacePower,
            9 => DescriptorType::OnTheGo,
            10 => DescriptorType::Debug,
            11 => DescriptorType::InterfaceAssociation,
            12 => DescriptorType::Security,
            13 => DescriptorType::Key,
            14 => DescriptorType::EncryptionType,
            15 => DescriptorType::BinaryDeviceObjectStore,
            16 => DescriptorType::DeviceCapability,
            17 => DescriptorType::WirelessEndpointCompanion,
            48 => DescriptorType::SuperSpeedEndpointCompanion,
            _ => return Err(ErrorKind::FailedConversion),
        };
        Ok(result)
    }
}

/// Language ID
#[repr(u16)]
pub enum LanguageId {
    EnglishUnitedStates = 0x0409,
}

/// USB device descriptor
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DeviceDescriptor {
    pub descriptor_length: u8,          // 18
    pub descriptor_type: u8, // 1 = Device
    pub descriptor_version: u16,
    pub device_class: u8,
    pub device_subclass: u8,
    pub device_protocol: u8,
    pub max_packet_size: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub device_version_number: u16,
    pub manufacturer_index: u8,
    pub product_index: u8,
    pub serial_index: u8,
    pub num_configurations: u8,
}

impl From<DeviceDescriptor> for [u8; 18] {
    fn from(descriptor: DeviceDescriptor) -> [u8; 18] {
        let buffer = [
            descriptor.descriptor_length,
            descriptor.descriptor_type,
            descriptor.descriptor_version.to_le_bytes()[1], descriptor.descriptor_version.to_le_bytes()[0],
            descriptor.device_class,
            descriptor.device_subclass,
            descriptor.device_protocol,
            descriptor.max_packet_size,
            descriptor.vendor_id.to_le_bytes()[1], descriptor.vendor_id.to_le_bytes()[0],
            descriptor.product_id.to_le_bytes()[1], descriptor.product_id.to_le_bytes()[0],
            descriptor.device_version_number.to_le_bytes()[1], descriptor.device_version_number.to_le_bytes()[0],
            descriptor.manufacturer_index,
            descriptor.product_index,
            descriptor.serial_index,
            descriptor.num_configurations,
        ];

        buffer
    }
}


/// USB configuration descriptor
pub struct ConfigurationDescriptor<'a> {
    pub length: u8,          // 9
    pub descriptor_type: u8, // 2 = Config
    pub total_length: u16,
    pub num_interfaces: u8,
    pub configuration_value: u8,
    pub configuration_index: u8,
    pub attributes: u8,
    pub max_power: u8,
    pub interface_descriptors: &'a [InterfaceDescriptor<'a>],
}

/// USB interface descriptor
pub struct InterfaceDescriptor<'a> {
    pub length: u8,          // 9
    pub descriptor_type: u8, // 4 = Interface
    pub interface_number: u16,
    pub alternate_setting: u8,
    pub num_endpoints: u8,
    pub interface_class: u8,
    pub interface_subclass: u8,
    pub interface_protocol: u8,
    pub interface_index: u8,
    pub endpoint_descriptors: &'a [EndpointDescriptor],
}

/// USB endpoint descriptor
pub struct EndpointDescriptor {
    pub length: u8,          // 7
    pub descriptor_type: u8, // 5 = Endpoint
    pub endpoint_address: u8,
    pub attributes: u8,
    pub max_packet_size: u8,
    pub interval: u8,
}

/// USB String descriptor for language ids
pub struct StringDescriptorZero<'a> {
    pub lengh: u8,
    pub descriptor_type: u8, // 3 = String
    pub language_ids: &'a [LanguageId],
}

/// USB String Descriptor
pub struct StringDescriptor<'a> {
    pub length: u8,
    pub descriptor_type: u8, // 3 = String
    pub string: &'a str,
}

impl <'a> StringDescriptor<'a> {
    pub fn new(string: &'a str) -> Self {
        Self {
            length: string.len().try_into().unwrap_or(0xff), // TODO
            descriptor_type: DescriptorType::String as u8,
            string
        }
    }
}
