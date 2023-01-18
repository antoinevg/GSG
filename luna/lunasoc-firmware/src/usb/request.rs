///! The Setup Packet
///!
///! see: https://www.beyondlogic.org/usbnutshell/usb6.shtml

use crate::Error;


// - UsbSetupRequest ----------------------------------------------------------


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SetupPacket {
    // 0..4 Recipient: 0=Device, 1=Interface, 2=Endpoint, 3=Other, 4-31=Reserved
    // 5..6 Type: 0=Standard, 1=Class, 2=Vendor, 3=Reserved
    // 7    Data Phase Transfer Direction: 0=Host to Device, 1=Device to Host
    pub request_type: u8,
    pub request: u8,
    pub value: u16,
    pub index: u16,
    pub length: u16,
}


// - request_type -------------------------------------------------------------

#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum Recipient {
    Device = 0,
    Interface = 1,
    Endpoint = 2,
    Other = 3,
    Reserved = 4,
}

impl TryFrom<u8> for Recipient {
    type Error = crate::Error;

    fn try_from(value: u8) -> core::result::Result<Self, Self::Error> {
        let result = match value {
            0 => Recipient::Device,
            1 => Recipient::Interface,
            2 => Recipient::Endpoint,
            3 => Recipient::Other,
            4..=31 => Recipient::Reserved,
            _ => return Err(Error::FailedConversion),
        };
        Ok(result)
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

impl TryFrom<u8> for RequestType {
    type Error = crate::Error;

    fn try_from(value: u8) -> core::result::Result<Self, Self::Error> {
        let result = match value {
            0 => RequestType::Standard,
            1 => RequestType::Class,
            2 => RequestType::Vendor,
            3 => RequestType::Reserved,
            _ => return Err(Error::FailedConversion),
        };
        Ok(result)
    }
}

/// 0x80
pub const MASK_DIRECTION_IN: u8 = 0b1000_0000;

#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum Direction {
    HostToDevice = 0,
    DeviceToHost = 1,
}

impl TryFrom<u8> for Direction {
    type Error = crate::Error;

    fn try_from(value: u8) -> core::result::Result<Self, Self::Error> {
        let result = match value {
            0 => Direction::HostToDevice,
            1 => Direction::DeviceToHost,
            _ => return Err(Error::FailedConversion),
        };
        Ok(result)
    }
}

// - request ------------------------------------------------------------------

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
}

impl TryFrom<u8> for Request {
    type Error = crate::Error;

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
            _ => return Err(Error::FailedConversion),
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

impl TryFrom<u8> for Feature {
    type Error = crate::Error;

    fn try_from(value: u8) -> core::result::Result<Self, Self::Error> {
        let result = match value {
            0 => Feature::EndpointHalt,
            1 => Feature::DeviceRemoteWakeup,
            _ => return Err(Error::FailedConversion),
        };
        Ok(result)
    }
}

#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum Descriptor {
    Device = 1,
    Configuration = 2,
    String = 3,
}

impl TryFrom<u8> for Descriptor {
    type Error = crate::Error;

    fn try_from(value: u8) -> core::result::Result<Self, Self::Error> {
        let result = match value {
            0 => Descriptor::Device,
            1 => Descriptor::Configuration,
            3 => Descriptor::String,
            _ => return Err(Error::FailedConversion),
        };
        Ok(result)
    }
}
