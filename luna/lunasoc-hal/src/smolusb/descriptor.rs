#![allow(dead_code, unused_imports, unused_variables, unused_mut)] // TODO

///! USB Descriptor types
use crate::smolusb::ErrorKind;

use core::iter;
use core::mem::size_of;
use core::slice;
use core::iter::Chain;
use core::marker::PhantomData;
use heapless::Vec;

// Serialization cases:
//
// 1. DONE Fixed size struct with primitive types and fixed byte order (LE)
// 2. DONE Case #1 plus a variable length array of unicode characters
// 3. Case #1 plus a variable length array of fixed size types (struct and enum)

/// DescriptorType
#[derive(Copy, Clone, Debug, PartialEq)]
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

/// USB device descriptor
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DeviceDescriptor {
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

impl DeviceDescriptor {
    pub const N: usize = 18; //size_of::<DeviceDescriptor>();

    pub const fn length(&self) -> u8 {
        Self::N as u8
    }

    pub const fn descriptor_type(&self) -> u8 {
        DescriptorType::Device as u8
    }
}

impl IntoIterator for DeviceDescriptor {
    type Item = u8;
    type IntoIter = core::array::IntoIter<u8, { DeviceDescriptor::N }>;

    fn into_iter(self) -> Self::IntoIter {
        const N: usize = DeviceDescriptor::N;

        let iter: core::array::IntoIter<u8, 0> = [].into_iter();
        let chain: core::iter::Chain<_, core::array::IntoIter<u8, 1>> = iter
            .chain(self.length().to_le_bytes())
            .chain(self.descriptor_type().to_le_bytes())
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

// - DeviceQualifierDescriptor ---------------------------------------------------------

/// USB device qualifier descriptor
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DeviceQualifierDescriptor {
    pub _length: u8,          // 10
    pub _descriptor_type: u8, // 6 = DeviceQualifier
    pub descriptor_version: u16,
    pub device_class: u8,
    pub device_subclass: u8,
    pub device_protocol: u8,
    pub max_packet_size: u8,
    pub num_configurations: u8,
    pub reserved: u8,
}

impl DeviceQualifierDescriptor {
    pub const N: usize = 10; //size_of::<DeviceQualifierDescriptor>();

    pub const fn length(&self) -> u8 {
        Self::N as u8
    }

    pub const fn descriptor_type(&self) -> u8 {
        DescriptorType::Device as u8
    }
}

impl IntoIterator for DeviceQualifierDescriptor {
    type Item = u8;
    type IntoIter = core::array::IntoIter<u8, { size_of::<DeviceQualifierDescriptor>() }>;

    fn into_iter(self) -> Self::IntoIter {
        const N: usize = DeviceQualifierDescriptor::N;

        let iter: core::array::IntoIter<u8, 0> = [].into_iter();
        let chain: core::iter::Chain<_, core::array::IntoIter<u8, 1>> = iter
            .chain(self.length().to_le_bytes())
            .chain(self.descriptor_type().to_le_bytes())
            .chain(self.descriptor_version.to_le_bytes())
            .chain(self.device_class.to_le_bytes())
            .chain(self.device_subclass.to_le_bytes())
            .chain(self.device_protocol.to_le_bytes())
            .chain(self.max_packet_size.to_le_bytes())
            .chain(self.num_configurations.to_le_bytes())
            .chain(self.reserved.to_le_bytes());

        let vec: Vec<u8, N> = chain.take(N).collect::<Vec<u8, N>>();
        let _vec_iter: <heapless::Vec<u8, N> as IntoIterator>::IntoIter = vec.clone().into_iter();
        let buffer: [u8; N] = vec.into_array().expect("unproven");
        let iter: core::array::IntoIter<u8, N> = buffer.into_iter();
        iter
    }
}

// - ConfigurationDescriptor --------------------------------------------------

/// USB configuration descriptor
pub struct ConfigurationDescriptor<'a> {
    pub _length: u8,          // 9
    pub descriptor_type: DescriptorType, // 2 = Configuration, 3 = OtherSpeedConfiguration TODO
    pub _total_length: u16,
    pub _num_interfaces: u8,
    pub configuration_value: u8,
    pub configuration_string_index: u8,
    pub attributes: u8,
    pub max_power: u8,
    pub interface_descriptors: &'a [&'a InterfaceDescriptor<'a>],
}

impl<'a> ConfigurationDescriptor<'a> {
    pub const N: usize = 9;

    pub const fn length(&self) -> u8 {
        Self::N as u8
    }

    pub const fn descriptor_type(&self) -> u8 {
        // TODO
        self.descriptor_type as u8
    }

    pub fn total_length(&self) -> u16 {
        let iter: slice::Iter<'a, &InterfaceDescriptor<'a>> = self.interface_descriptors.iter();
        let map: iter::Map<_, _> = iter.map(|x: &'a &InterfaceDescriptor| {
            x.length() as u16 + (x.num_endpoints() as u16 * EndpointDescriptor::N as u16)
        });
        let reduce: u16 = map.reduce(|a, x| a + x).expect("interface descriptor length overflow");
        Self::N as u16 + reduce
    }

    pub fn num_interfaces(&self) -> u8 {
        self.interface_descriptors.len() as u8
    }

    pub fn iter(&self) -> ConfigurationDescriptorIterator {
        ConfigurationDescriptorIterator::new(self)
    }
}

pub struct ConfigurationDescriptorIterator<'a> {
    descriptor: &'a ConfigurationDescriptor<'a>,
    state: IteratorState,
}

impl<'a> ConfigurationDescriptorIterator<'a> {
    pub fn new(descriptor: &'a ConfigurationDescriptor) -> Self {
        Self {
            descriptor,
            state: IteratorState::Body(0),
        }
    }

    fn byte_at_offset(&self, index: usize) -> Option<u8> {
        let descriptor = self.descriptor;
        let mut chain: iter::Chain<
            _,
            iter::FlatMap<slice::Iter<&InterfaceDescriptor>, InterfaceDescriptorIterator, _>,
        > = []
            .into_iter()
            .chain(descriptor.length().to_le_bytes())
            .chain(descriptor.descriptor_type().to_le_bytes())
            .chain(descriptor.total_length().to_le_bytes())
            .chain(descriptor.num_interfaces().to_le_bytes())
            .chain(descriptor.configuration_value.to_le_bytes())
            .chain(descriptor.configuration_string_index.to_le_bytes())
            .chain(descriptor.attributes.to_le_bytes())
            .chain(descriptor.max_power.to_le_bytes())
            .chain(
                descriptor
                    .interface_descriptors
                    .iter()
                    .flat_map(|x| x.iter()),
            );
        chain.nth(index)
    }
}

impl<'a> Iterator for ConfigurationDescriptorIterator<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            IteratorState::Header => None,
            IteratorState::Body(index) => {
                self.state = IteratorState::Body(index + 1);
                self.byte_at_offset(index)
            }
        }
    }
}

// - InterfaceDescriptor ------------------------------------------------------

/// USB interface descriptor
pub struct InterfaceDescriptor<'a> {
    pub _length: u8,          // 9
    pub _descriptor_type: u8, // 4 = Interface
    pub interface_number: u8,
    pub alternate_setting: u8,
    pub _num_endpoints: u8,
    pub interface_class: u8,
    pub interface_subclass: u8,
    pub interface_protocol: u8,
    pub interface_string_index: u8,
    pub endpoint_descriptors: &'a [&'a EndpointDescriptor],
}

impl<'a> InterfaceDescriptor<'a> {
    pub const N: usize = 9;

    pub const fn length(&self) -> u8 {
        Self::N as u8
    }

    pub const fn descriptor_type(&self) -> u8 {
        DescriptorType::Interface as u8
    }

    pub fn num_endpoints(&self) -> u8 {
        self.endpoint_descriptors.len() as u8
    }

    pub fn iter(&'a self) -> InterfaceDescriptorIterator<'a> {
        InterfaceDescriptorIterator::new(self)
    }
}

pub struct InterfaceDescriptorIterator<'a> {
    descriptor: &'a InterfaceDescriptor<'a>,
    state: IteratorState,
}

impl<'a> InterfaceDescriptorIterator<'a> {
    pub fn new(descriptor: &'a InterfaceDescriptor) -> Self {
        Self {
            descriptor,
            state: IteratorState::Body(0),
        }
    }

    // TODO play with https://stackoverflow.com/questions/65739365/
    //                https://stackoverflow.com/questions/66076379/
    //                https://www.reddit.com/r/rust/comments/3q3edl/how_do_you_store_iterators/
    fn byte_at_offset(&self, index: usize) -> Option<u8> {
        let descriptor = self.descriptor;
        let mut chain: iter::Chain<
            _,
            iter::FlatMap<slice::Iter<&EndpointDescriptor>, EndpointDescriptorIterator, _>,
        > = []
            .into_iter()
            .chain(descriptor.length().to_le_bytes())
            .chain(descriptor.descriptor_type().to_le_bytes())
            .chain(descriptor.interface_number.to_le_bytes())
            .chain(descriptor.alternate_setting.to_le_bytes())
            .chain(descriptor.num_endpoints().to_le_bytes())
            .chain(descriptor.interface_class.to_le_bytes())
            .chain(descriptor.interface_subclass.to_le_bytes())
            .chain(descriptor.interface_protocol.to_le_bytes())
            .chain(descriptor.interface_string_index.to_le_bytes())
            .chain(
                descriptor
                    .endpoint_descriptors
                    .iter()
                    .flat_map(|x| x.iter()),
            );
        chain.nth(index)
    }
}

impl<'a> Iterator for InterfaceDescriptorIterator<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            IteratorState::Header => None,
            IteratorState::Body(index) => {
                self.state = IteratorState::Body(index + 1);
                self.byte_at_offset(index)
            }
        }
    }
}

// - EndpointDescriptor -------------------------------------------------------

/// USB endpoint descriptor
pub struct EndpointDescriptor {
    pub _length: u8,          // 7
    pub _descriptor_type: u8, // 5 = Endpoint
    pub endpoint_address: u8,
    pub attributes: u8,
    pub max_packet_size: u16,
    pub interval: u8,
}

impl EndpointDescriptor {
    pub const N: usize = 7;

    pub const fn length(&self) -> u8 {
        Self::N as u8
    }

    pub const fn descriptor_type(&self) -> u8 {
        DescriptorType::Endpoint as u8
    }

    pub fn iter(&self) -> EndpointDescriptorIterator {
        EndpointDescriptorIterator::new(self)
    }
}

pub struct EndpointDescriptorIterator<'a> {
    descriptor: &'a EndpointDescriptor,
    iter: core::array::IntoIter<u8, { EndpointDescriptor::N }>,
}

impl<'a> EndpointDescriptorIterator<'a> {
    pub fn new(descriptor: &'a EndpointDescriptor) -> Self {
        let iter = [
            descriptor.length(),
            descriptor.descriptor_type(),
            descriptor.endpoint_address,
            descriptor.attributes,
            descriptor.max_packet_size.to_le_bytes()[0],
            descriptor.max_packet_size.to_le_bytes()[1],
            descriptor.interval,
        ]
        .into_iter();
        Self { descriptor, iter }
    }
}

impl<'a> Iterator for EndpointDescriptorIterator<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

// - StringDescriptorZero -----------------------------------------------------

/// Language ID
#[derive(Clone, Copy, Debug)]
#[repr(u16)]
pub enum LanguageId {
    EnglishUnitedStates = 0x0409,
    EnglishUnitedKingdom = 0x0809,
    EnglishCanadian = 0x1009,
    EnglishSouthAfrica = 0x1c09,
}

/// USB String descriptor for language ids
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct StringDescriptorZero<'a> {
    pub _length: u8,
    pub _descriptor_type: u8, // 3 = String
    pub language_ids: &'a [LanguageId],
}

impl<'a> StringDescriptorZero<'a> {
    pub const N: usize = 2;

    pub fn length(&self) -> u8 {
        (Self::N + (size_of::<LanguageId>() * self.language_ids.len())) as u8
    }

    pub const fn descriptor_type(&self) -> u8 {
        DescriptorType::String as u8
    }

    pub fn iter(&self) -> StringDescriptorZeroIterator {
        StringDescriptorZeroIterator::new(self)
    }
}

enum IteratorState {
    Header,
    Body(usize),
}

pub struct StringDescriptorZeroIterator<'a> {
    descriptor: &'a StringDescriptorZero<'a>,
    state: IteratorState,
    header_iter: core::array::IntoIter<u8, 2>,
}

impl<'a> StringDescriptorZeroIterator<'a> {
    pub fn new(descriptor: &'a StringDescriptorZero) -> Self {
        let header_iter = [descriptor.length(), descriptor.descriptor_type()].into_iter();
        Self {
            descriptor,
            state: IteratorState::Header,
            header_iter,
        }
    }
}

impl<'a> Iterator for StringDescriptorZeroIterator<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            IteratorState::Header => match self.header_iter.next() {
                Some(byte) => Some(byte),
                None => {
                    self.state = IteratorState::Body(0);
                    self.next()
                }
            },
            IteratorState::Body(index) => {
                match self
                    .descriptor
                    .language_ids
                    .iter()
                    .flat_map(|x: &LanguageId| (*x as u16).to_le_bytes().into_iter())
                    .nth(index)
                {
                    Some(byte) => {
                        self.state = IteratorState::Body(index + 1);
                        Some(byte)
                    }
                    None => {
                        // reset iterator
                        self.state = IteratorState::Header;
                        None
                    }
                }
            }
        }
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
    pub const N: usize = 2;

    /// Descriptor length
    pub fn length(&self) -> u8 {
        // TODO USB string descriptors can be a maximum of 126 characters
        (Self::N + (self.string.encode_utf16().count() * size_of::<u16>())) as u8
    }

    /// Descriptor type
    pub const fn descriptor_type(&self) -> u8 {
        DescriptorType::String as u8
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
    state: StringDescriptorIteratorState,
    string_iter: Utf16ByteIterator<'a>,
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
