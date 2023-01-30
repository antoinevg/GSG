/// USB Error type
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive] // ... is a double-edged sword
pub enum ErrorKind {
    FailedConversion,
    Timeout,
    Overflow,
    Underflow,
    Unknown,
}

// trait: core::error::Error
impl core::error::Error for ErrorKind {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        use ErrorKind::*;
        match self {
            FailedConversion => "TODO FailedConversion",
            Timeout => "TODO Timeout",
            Overflow => "TODO Overflow",
            Underflow => "TODO Underflow",
            Unknown => "TODO Unknown",
        }
    }
}

// trait:: core::fmt::Display
impl core::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self, f)
    }
}

// trait: core::num::TryFromIntError
impl core::convert::From<core::num::TryFromIntError> for ErrorKind {
    fn from(_error: core::num::TryFromIntError) -> Self {
        ErrorKind::FailedConversion
    }
}

// trait: libgreat::error::Error
impl libgreat::error::Error for ErrorKind {
    type Error = ErrorKind; // TODO can we just say `Self`?
    fn kind(&self) -> Self::Error {
        *self
    }
}

/// USB Result<T>
pub type Result<T> = core::result::Result<T, ErrorKind>;
