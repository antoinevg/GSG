#![allow(dead_code, unused_imports, unused_variables, unused_mut)] // TODO

use crate::smolusb::traits::{AsByteSliceIterator, GetTotalLength, SetTotalLength};
use crate::smolusb::ErrorKind;

use heapless::Vec;
use zerocopy::{AsBytes, FromBytes};

use core::iter;
use core::iter::Chain;
use core::marker::PhantomData;
use core::mem::size_of;
use core::slice;

///! USB Descriptor types

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
#[derive(AsBytes, FromBytes)]
#[repr(C, packed)]
pub struct DeviceDescriptor {
    pub _length: u8,
    pub _descriptor_type: u8,
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

impl AsByteSliceIterator for DeviceDescriptor {}

impl DeviceDescriptor {
    pub const fn new() -> Self {
        Self {
            _length: 18,
            _descriptor_type: DescriptorType::Device as u8,
            descriptor_version: 0,
            device_class: 0,
            device_subclass: 0,
            device_protocol: 0,
            max_packet_size: 0,
            vendor_id: 0,
            product_id: 0,
            device_version_number: 0,
            manufacturer_string_index: 0,
            product_string_index: 0,
            serial_string_index: 0,
            num_configurations: 0,
        }
    }
}

impl Default for DeviceDescriptor {
    fn default() -> Self {
        Self::new()
    }
}

// - DeviceQualifierDescriptor ---------------------------------------------------------

/// USB device qualifier descriptor
#[derive(AsBytes, FromBytes)]
#[repr(C, packed)]
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

impl AsByteSliceIterator for DeviceQualifierDescriptor {}

impl DeviceQualifierDescriptor {

    pub const fn new() -> Self {
        Self {
            _length: 10,
            _descriptor_type: DescriptorType::DeviceQualifier as u8,
            descriptor_version: 0,
            device_class: 0,
            device_subclass: 0,
            device_protocol: 0,
            max_packet_size: 0,
            num_configurations: 0,
            reserved: 0,
        }
    }
}

impl Default for DeviceQualifierDescriptor {
    fn default() -> Self {
        Self::new()
    }
}


// - ConfigurationDescriptor --------------------------------------------------

/// USB configuration descriptor
pub struct ConfigurationDescriptor<'a> {
    pub _length: u8,                     // 9
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
        let reduce: u16 = map
            .reduce(|a, x| a + x)
            .expect("interface descriptor length overflow");
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
#[derive(AsBytes, FromBytes)]
#[repr(C, packed)]
pub struct EndpointDescriptor {
    pub _length: u8,          // 7
    pub _descriptor_type: u8, // 5 = Endpoint
    pub endpoint_address: u8,
    pub attributes: u8,
    pub max_packet_size: u16,
    pub interval: u8,
}

impl AsByteSliceIterator for EndpointDescriptor {}

impl EndpointDescriptor {
    pub const fn new() -> Self {
        Self {
            _length: 7,
            _descriptor_type: DescriptorType::Endpoint as u8,
            endpoint_address: 0,
            attributes: 0,
            max_packet_size: 0,
            interval: 0,
        }
    }
}

impl Default for EndpointDescriptor {
    fn default() -> Self {
        Self::new()
    }
}

/////////////////////////////////
// lose this
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
/////////////////////////////////

///////////////////////////
// lose this
enum IteratorState {
    Header,
    Body(usize),
}



// - LanguageId ---------------------------------------------------------------

/// Language ID
#[derive(AsBytes, Copy, Clone, Debug)]
#[repr(u16)]
pub enum LanguageId {
    EnglishUnitedStates = 0x0409,
    EnglishUnitedKingdom = 0x0809,
    EnglishCanadian = 0x1009,
    EnglishSouthAfrica = 0x1c09,
}

impl AsByteSliceIterator for LanguageId {}


// - StringDescriptorZero -----------------------------------------------------

pub struct StringDescriptorZero<'a> {
    head: StringDescriptorZeroHead,
    tail: &'a [LanguageId],
}

impl<'a> StringDescriptorZero<'a> {
    pub const fn new(tail: &'a [LanguageId]) -> Self {
        let head_length = size_of::<StringDescriptorZeroHead>();
        let tail_length = tail.len() * size_of::<LanguageId>();
        Self {
            head: StringDescriptorZeroHead {
                _length: (head_length + tail_length) as u8,
                _descriptor_type: DescriptorType::String as u8,
            },
            tail,
        }
    }

    pub fn iter(&'a self) -> CompositeIterator<'a, StringDescriptorZeroHead, LanguageId> {
        let iter  = CompositeIterator::new(&self.head, self.tail);
        iter
    }
}


#[derive(AsBytes, FromBytes, Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct StringDescriptorZeroHead {
    pub _length: u8,
    pub _descriptor_type: u8, // 3 = String
}

impl StringDescriptorZeroHead {
    pub const fn new() -> Self {
        Self {
            _length: 0,
            _descriptor_type: DescriptorType::String as u8,
        }
    }
}

impl AsByteSliceIterator for StringDescriptorZeroHead {}

/*
impl SetTotalLength for StringDescriptorZeroHead {
    fn set_total_length(&mut self, total_length: usize) {
        self._length = total_length as u8;
    }
}

// pretty much useless if it can't be const
impl GetTotalLength for StringDescriptorZeroHead {
    fn total_length(&self, tail_count: usize) -> usize {
        let head_length = size_of::<StringDescriptorZeroHead>();
        let tail_length = tail_count * size_of::<LanguageId>();
        head_length + tail_length
    }
}
*/


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


// - CompositeIterator --------------------------------------------------------


type HeadIterator<'a> = slice::Iter<'a, u8>;
type TailIterator<'a, T> = iter::FlatMap<
    slice::Iter<'a, T>,
    slice::Iter<'a, u8>,
    &'a dyn Fn(&'a T) -> slice::Iter<'a, u8>,
>;
type CompositeChain<'a, T> = iter::Chain<slice::Iter<'a, u8>, TailIterator<'a, T>>;

pub struct CompositeIterator<'a, H, T> {
    chain: CompositeChain<'a, T>,
    _marker: PhantomData<H>,
}

impl<'a, H, T> CompositeIterator<'a, H, T>
where
    H: AsByteSliceIterator + SetTotalLength + 'a,
    T: AsByteSliceIterator + 'a,
{
    pub fn new_mut(head: &'a mut H, tail: &'a [T]) -> Self {
        // this works but it ain't great
        let head_length = size_of::<H>();
        let tail_length = tail.len() * size_of::<T>();
        head.set_total_length(head_length + tail_length);

        let head_iter: HeadIterator<'a> = head.as_iter();
        let tail_iter: TailIterator<'a, T> =
            tail.iter().flat_map(&|x: &'a T| x.as_iter());
        let chain: CompositeChain<'a, T> = head_iter.chain(tail_iter);

        Self {
            chain,
            _marker: PhantomData,
        }
    }
}

impl<'a, H, T> CompositeIterator<'a, H, T>
where
    H: AsByteSliceIterator + 'a,
    T: AsByteSliceIterator + 'a,
{

    pub fn new(head: &'a H, tail: &'a [T]) -> Self {
        let head_iter: HeadIterator<'a> = head.as_iter();
        let tail_iter: TailIterator<'a, T> =
            tail.iter().flat_map(&|x: &'a T| x.as_iter());
        let chain: CompositeChain<'a, T> = head_iter.chain(tail_iter);
        Self {
            chain,
            _marker: PhantomData,
        }
    }

}

impl<'a, H, T> Iterator for CompositeIterator<'a, H, T> {
    type Item = &'a u8;
    fn next(&mut self) -> Option<Self::Item> {
        self.chain.next()
    }
}
