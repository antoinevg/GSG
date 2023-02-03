#![allow(dead_code, unused_imports, unused_variables, unused_mut)] // TODO

///! USB Descriptor types
use crate::smolusb::ErrorKind;

use core::mem::size_of;

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

// - DeviceDescriptor ---------------------------------------------------------

use core::iter::Chain;
use core::marker::PhantomData;
use heapless::Vec;

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
    pub manufacturer_string_index: u8,
    pub product_string_index: u8,
    pub serial_string_index: u8,
    pub num_configurations: u8,
}

impl IntoIterator for DeviceDescriptor {
    type Item = u8;
    type IntoIter = core::array::IntoIter<u8, 18>;

    fn into_iter(self) -> Self::IntoIter {
        const N: usize = size_of::<DeviceDescriptor>();

        let iter: core::array::IntoIter<u8, 0> = [].into_iter();
        let chain: core::iter::Chain<_, core::array::IntoIter<u8, 1>> = iter
            .chain(self.descriptor_length.to_le_bytes())
            .chain(self.descriptor_type.to_le_bytes())
            .chain(self.descriptor_version.to_le_bytes())
            .chain(self.device_class.to_le_bytes())
            .chain(self.device_subclass.to_le_bytes())
            .chain(self.device_protocol.to_le_bytes())
            .chain(self.max_packet_size.to_le_bytes())
            .chain(self.vendor_id.to_le_bytes())
            .chain(self.product_id.to_le_bytes())
            .chain(self.device_version_number.to_le_bytes())
            .chain(self.manufacturer_string_index.to_le_bytes())
            .chain(self.product_string_index.to_le_bytes())
            .chain(self.serial_string_index.to_le_bytes())
            .chain(self.num_configurations.to_le_bytes());

        let vec: Vec<u8, N> = chain.take(N).collect::<Vec<u8, N>>();
        let _vec_iter: <heapless::Vec<u8, N> as IntoIterator>::IntoIter = vec.clone().into_iter();
        let buffer: [u8; N] = vec.into_array().expect("unproven");
        let iter: core::array::IntoIter<u8, N> = buffer.into_iter();
        iter
    }
}

/*
type DeviceDescriptorBuffer = [u8; core::mem::size_of::<DeviceDescriptor>()];
impl From<DeviceDescriptor> for DeviceDescriptorBuffer {
    fn from(descriptor: DeviceDescriptor) -> Self {
        // cursed but proven - watch out for byte order!
        let _option_1: Self = unsafe { core::mem::transmute::<DeviceDescriptor, Self>(descriptor) };

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
*/

// - ConfigurationDescriptor --------------------------------------------------

/// USB configuration descriptor
pub struct ConfigurationDescriptor<'a> {
    pub length: u8,          // 9
    pub descriptor_type: u8, // 2 = Config
    pub total_length: u16,
    pub num_interfaces: u8,
    pub configuration_value: u8,
    pub configuration_string_index: u8,
    pub attributes: u8,
    pub max_power: u8,
    pub interface_descriptors: &'a [&'a InterfaceDescriptor<'a>],
}

// - InterfaceDescriptor ------------------------------------------------------

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
    pub interface_string_index: u8,
    pub endpoint_descriptors: &'a [&'a EndpointDescriptor],
}

// - EndpointDescriptor -------------------------------------------------------

/// USB endpoint descriptor
pub struct EndpointDescriptor {
    pub length: u8,          // 7
    pub descriptor_type: u8, // 5 = Endpoint
    pub endpoint_address: u8,
    pub attributes: u8,
    pub max_packet_size: u8,
    pub interval: u8,
}

// - StringDescriptorZero -----------------------------------------------------

/// Language ID
#[derive(Clone, Copy)]
#[repr(u16)]
pub enum LanguageId {
    EnglishUnitedStates = 0x0409,
}

/// USB String descriptor for language ids
pub struct StringDescriptorZero<'a> {
    pub length: u8,
    pub descriptor_type: u8, // 3 = String
    pub language_ids: &'a [LanguageId],
}

/*impl<'a> IntoIterator for StringDescriptorZero<'a>
{
    type Item = <ByteIterator as Iterator>::Item;
    type IntoIter = ByteIterator;
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            foo: 42,
        }
    }
}*/

use core::iter;
use core::slice;

impl<'a> IntoIterator for StringDescriptorZero<'a> {
    type Item = &'a u8;
    type IntoIter = slice::Iter<'a, u8>;
    fn into_iter(self) -> Self::IntoIter {
        let slice: &[u8] = &[self.length, self.descriptor_type];
        let mut iter: slice::Iter<u8> = slice.into_iter();

        let iter_ids: slice::Iter<LanguageId> = self.language_ids.into_iter();

        // LanguageId -> u16
        let map_ids: iter::Map<slice::Iter<LanguageId>, _> =
            iter_ids.clone().map(|x: &LanguageId| *x as u16);

        // LanguageId -> [u8; 2]
        let map_ids: iter::FlatMap<slice::Iter<LanguageId>, [u8; 2], _> =
            iter_ids.clone().flat_map(|x: &LanguageId| {
                let id = *x as u16;
                id.to_le_bytes()
            });

        // LanguageId -> core::array::IntoIter<u8, 2>
        let map_ids: iter::FlatMap<slice::Iter<LanguageId>, core::array::IntoIter<u8, 2>, _> =
            iter_ids.clone().flat_map(|x: &LanguageId| {
                let id = *x as u16;
                id.to_le_bytes().into_iter()
            });

        let f: &dyn FnMut(&LanguageId) -> [u8; 2] = &|x: &LanguageId| {
            let id = *x as u16;
            id.to_le_bytes()
        };

        unimplemented!();
    }
}

/// ByteIterator
pub struct ByteIterator {
    foo: u8,
}

impl Iterator for ByteIterator {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

// - StringDescriptor ---------------------------------------------------------

/// USB String Descriptor
pub struct StringDescriptor<'a> {
    _length: u8,
    _descriptor_type: u8,
    string: &'a str,
}

impl<'a> StringDescriptor<'a> {
    pub const fn new(string: &'a str) -> Self {
        // TODO USB string descriptors can be a maximum of 126 characters
        /*let string = match self.string.char_indices().nth(126) {
            None => string,
            Some((idx, _)) => &string[..idx],
        };*/
        Self {
            _length: 2,
            _descriptor_type: DescriptorType::String as u8,
            string,
        }
    }
}

impl<'a> StringDescriptor<'a> {
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
    pub fn iter(&self) -> StringDescriptorIterator {
        StringDescriptorIterator::new(self)
    }
}

enum StringDescriptorIteratorState {
    Length,
    DescriptorType,
    String,
}

pub struct StringDescriptorIterator<'a> {
    descriptor: &'a StringDescriptor<'a>,
    string_iter: Utf16ByteIterator<'a>,
    state: StringDescriptorIteratorState,
}

impl<'a> StringDescriptorIterator<'a> {
    fn new(descriptor: &'a StringDescriptor) -> Self {
        Self {
            descriptor,
            string_iter: Utf16ByteIterator::new(descriptor.string.encode_utf16()),
            state: StringDescriptorIteratorState::Length,
        }
    }
}

impl<'a> Iterator for StringDescriptorIterator<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            StringDescriptorIteratorState::Length => {
                self.state = StringDescriptorIteratorState::DescriptorType;
                Some(self.descriptor.length())
            }
            StringDescriptorIteratorState::DescriptorType => {
                self.state = StringDescriptorIteratorState::String;
                Some(self.descriptor.descriptor_type() as u8)
            }
            StringDescriptorIteratorState::String => {
                // TODO USB string descriptors can be a maximum of 126 characters
                match self.string_iter.next() {
                    Some(byte) => {
                        self.state = StringDescriptorIteratorState::String;
                        Some(byte)
                    }
                    None => {
                        self.string_iter =
                            Utf16ByteIterator::new(self.descriptor.string.encode_utf16());
                        self.state = StringDescriptorIteratorState::Length;
                        None
                    }
                }
            }
        }
    }
}

#[allow(dead_code)]
fn static_test_string_descriptor() {
    let descriptor = StringDescriptor::new("TRI-FIFO Example");
    for byte in descriptor.iter() {
        let _byte: u8 = byte;
    }
}

// - Utf16ByteIterator --------------------------------------------------------

#[derive(Clone)]
pub struct Utf16ByteIterator<'a> {
    encode_utf16: core::str::EncodeUtf16<'a>,
    byte: Option<u8>,
}

impl<'a> Utf16ByteIterator<'a> {
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
            None => match self.encode_utf16.next() {
                Some(unicode_char) => {
                    let bytes: [u8; 2] = unicode_char.to_le_bytes();
                    self.byte = Some(bytes[1]);
                    Some(bytes[0])
                }
                None => None,
            },
        }
    }
}
