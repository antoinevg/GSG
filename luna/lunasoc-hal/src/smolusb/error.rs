/// [`smolusb`] Error type
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ErrorKind {
    FailedConversion,
}

// trait: core::error::Error
impl core::error::Error for ErrorKind {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        use ErrorKind::*;
        match self {
            FailedConversion => "Failed to convert packet value",
        }
    }
}

// trait:: core::fmt::Display
impl core::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self, f)
    }
}

// trait: core::convert::From<core::num::TryFromIntError>
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
