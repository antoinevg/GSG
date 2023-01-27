#[derive(Debug)]
pub enum Error {
    // TODO move these into a usb::Error
    FailedConversion,
    Timeout,
    Overflow,
    Underflow,
    Unknown,
}

/*impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self, f)
    }
}*/
//impl core::error::Error for Error {}

// Ugly little hack for now - https://stackoverflow.com/questions/48430836/
impl<E: core::fmt::Display> core::convert::From<E> for Error {
    fn from(_error: E) -> Self {
        Error::Unknown
    }
}

pub type Result<T> = core::result::Result<T, Error>;
