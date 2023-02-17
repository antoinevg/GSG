///! Types for working with the SETUP packet.
use crate::smolusb::error::ErrorKind;

// - SetupPacket --------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SetupPacket {
    // 0..4 Recipient: 0=Device, 1=Interface, 2=Endpoint, 3=Other, 4-31=Reserved
    // 5..6 Type: 0=Standard, 1=Class, 2=Vendor, 3=Reserved
    // 7    Data Phase Transfer Direction: 0=Host to Device, 1=Device to Host
    pub request_type: u8,
    // values 0..=9 are standard, others are class or vendor
    pub request: u8,
    pub value: u16,
    pub index: u16,
    pub length: u16,
}

impl TryFrom<[u8; 8]> for SetupPacket {
    type Error = &'static dyn core::error::Error;

    fn try_from(buffer: [u8; 8]) -> core::result::Result<Self, Self::Error> {
        // Deserialize into a SetupRequest in the most cursed manner available to us
        // TODO parse properly
        Ok(unsafe { core::mem::transmute::<[u8; 8], SetupPacket>(buffer) })
    }
}

impl SetupPacket {
    pub fn request_type(&self) -> RequestType {
        RequestType::from(self.request_type)
    }

    pub fn recipient(&self) -> Recipient {
        Recipient::from(self.request_type)
    }

    pub fn direction(&self) -> Direction {
        Direction::from(self.request_type)
    }

    pub fn request(&self) -> core::result::Result<Request, ErrorKind> {
        Request::try_from(self.request)
    }
}

// - SetupPacket.request_type -------------------------------------------------

#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum Recipient {
    Device = 0,
    Interface = 1,
    Endpoint = 2,
    Other = 3,
    Reserved = 4,
}

impl From<u8> for Recipient {
    fn from(value: u8) -> Self {
        match value & 0b0001_1111 {
            0 => Recipient::Device,
            1 => Recipient::Interface,
            2 => Recipient::Endpoint,
            3 => Recipient::Other,
            4..=u8::MAX => Recipient::Reserved,
        }
    }
}

#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum RequestType {
    Standard = 0,
    Class = 1,
    Vendor = 2,
    Reserved = 3,
}

impl From<u8> for RequestType {
    fn from(value: u8) -> Self {
        match (value >> 5) & 0b0000_0011 {
            0 => RequestType::Standard,
            1 => RequestType::Class,
            2 => RequestType::Vendor,
            3..=u8::MAX => RequestType::Reserved,
        }
    }
}

/// USB traffic direction
#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum Direction {
    /// Host to device (OUT)
    HostToDevice = 0x00,
    /// Device to host (IN)
    DeviceToHost = 0x80, // 0b1000_0000,
}

impl From<u8> for Direction {
    fn from(value: u8) -> Self {
        match (value & 0b1000_0000) == 0 {
            true => Direction::HostToDevice,
            false => Direction::DeviceToHost,
        }
    }
}

// - SetupPacket.request ------------------------------------------------------

#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum Request {
    GetStatus = 0,
    ClearFeature = 1,
    SetFeature = 3,
    SetAddress = 5,
    GetDescriptor = 6,
    SetDescriptor = 7,
    GetConfiguration = 8,
    SetConfiguration = 9,
    GetInterface = 10,
    SetInterface = 11,
    SynchronizeFrame = 12,
    ClassOrVendor(u8),
}

impl TryFrom<u8> for Request {
    type Error = ErrorKind;

    fn try_from(value: u8) -> core::result::Result<Self, Self::Error> {
        let result = match value {
            0 => Request::GetStatus,
            1 => Request::ClearFeature,
            3 => Request::SetFeature,
            5 => Request::SetAddress,
            6 => Request::GetDescriptor,
            7 => Request::SetDescriptor,
            8 => Request::GetConfiguration,
            9 => Request::SetConfiguration,
            10 => Request::GetInterface,
            11 => Request::SetInterface,
            12 => Request::SynchronizeFrame,
            // TODO should we check that request_type is class or vendor?
            13..=u8::MAX => Request::ClassOrVendor(value),
            _ => return Err(ErrorKind::FailedConversion),
        };
        Ok(result)
    }
}

#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum Feature {
    EndpointHalt = 0,
    DeviceRemoteWakeup = 1,
}

impl TryFrom<u16> for Feature {
    type Error = ErrorKind;

    fn try_from(value: u16) -> core::result::Result<Self, Self::Error> {
        let result = match value {
            0 => Feature::EndpointHalt,
            1 => Feature::DeviceRemoteWakeup,
            _ => return Err(ErrorKind::FailedConversion),
        };
        Ok(result)
    }
}
