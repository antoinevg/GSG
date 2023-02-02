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
    pub descriptor_length: u8, // 18
    pub descriptor_type: u8,   // 1 = Device
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
type DeviceDescriptorBuffer = [u8; core::mem::size_of::<DeviceDescriptor>()];

impl From<DeviceDescriptor> for DeviceDescriptorBuffer {
    fn from(descriptor: DeviceDescriptor) -> Self {
        // cursed but proven - watch out for byte order!
        let _option_1: Self = unsafe {
            core::mem::transmute::<DeviceDescriptor, Self>(
                descriptor,
            )
        };

        // pedantic but proven
        let _option_2: Self = [
            descriptor.descriptor_length,
            descriptor.descriptor_type,
            descriptor.descriptor_version.to_le_bytes()[0],
            descriptor.descriptor_version.to_le_bytes()[1],
            descriptor.device_class,
            descriptor.device_subclass,
            descriptor.device_protocol,
            descriptor.max_packet_size,
            descriptor.vendor_id.to_le_bytes()[0],
            descriptor.vendor_id.to_le_bytes()[1],
            descriptor.product_id.to_le_bytes()[0],
            descriptor.product_id.to_le_bytes()[1],
            descriptor.device_version_number.to_le_bytes()[0],
            descriptor.device_version_number.to_le_bytes()[1],
            descriptor.manufacturer_index,
            descriptor.product_index,
            descriptor.serial_index,
            descriptor.num_configurations,
        ];

        _option_2
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
pub struct StringDescriptor {
    string: &'static str,
}

impl StringDescriptor {
    pub const fn new(string: &'static str) -> Self {
        // TODO USB string descriptors can be a maximum of 126 characters
        /*let string = match self.string.char_indices().nth(126) {
            None => string,
            Some((idx, _)) => &string[..idx],
        };*/
        Self {
            string,
        }
    }
}

impl StringDescriptor {
    /// Descriptor length
    pub fn length(&self) -> u8 {
        // TODO USB string descriptors can be a maximum of 126 characters
        2 + (self.string.encode_utf16().count() * 2) as u8
    }

    /// Descriptor type
    pub fn descriptor_type(&self) -> DescriptorType {
        DescriptorType::String
    }

    /// Returns an iterator to the descriptor
    ///
    /// Note that this
    pub fn iter(&self) -> StringDescriptorIterator {
        StringDescriptorIterator::new(self)
    }
}


// - StringDescriptorIterator

enum IteratorState {
    Length,
    DescriptorType,
    String,
}

pub struct StringDescriptorIterator<'a> {
    descriptor: &'a StringDescriptor,
    string_iter: Utf16ByteIterator<'a>,
    state: IteratorState,
}

impl<'a> StringDescriptorIterator<'a> {
    fn new(descriptor: &'a StringDescriptor) -> Self {
        Self {
            descriptor,
            string_iter: Utf16ByteIterator::new(descriptor.string.encode_utf16()),
            state: IteratorState::Length,
        }
    }
}

impl<'a> Iterator for StringDescriptorIterator<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            IteratorState::Length => {
                self.state = IteratorState::DescriptorType;
                Some(self.descriptor.length())
            },
            IteratorState::DescriptorType => {
                self.state = IteratorState::String;
                Some(self.descriptor.descriptor_type() as u8)
            }
            IteratorState::String => {
                // TODO USB string descriptors can be a maximum of 126 characters
                match self.string_iter.next() {
                    Some(byte) => {
                        self.state = IteratorState::String;
                        Some(byte)
                    }
                    None => {
                        self.string_iter = Utf16ByteIterator::new(self.descriptor.string.encode_utf16());
                        self.state = IteratorState::Length;
                        None
                    }
                }
            }
        }
    }
}

#[allow(dead_code)]
fn test() {
    let descriptor = StringDescriptor::new("TRI-FIFO Example");
    for byte in descriptor.iter() {
        let _byte: u8 = byte;
    }
}

// - Utf16ByteIterator

#[derive(Clone)]
pub struct Utf16ByteIterator <'a> {
    encode_utf16: core::str::EncodeUtf16<'a>,
    byte: Option<u8>,
}

impl<'a> Utf16ByteIterator <'a> {
    pub fn new(encode_utf16: core::str::EncodeUtf16<'a>) -> Self {
        Self {
            encode_utf16,
            byte: None,
        }
    }
}

impl<'a> Iterator for Utf16ByteIterator<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        match self.byte {
            Some(_) => self.byte.take(),
            None => {
                match self.encode_utf16.next() {
                    Some(unicode_char) => {
                        let bytes: [u8; 2] = unicode_char.to_le_bytes();
                        self.byte = Some(bytes[1]);
                        Some(bytes[0])
                    }
                    None => {
                        None
                    }
                }
            }
        }
    }
}
