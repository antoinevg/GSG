/**
 * Rust error-handling continues to be somewhat of a chore in no_std.
 *
 * Some light reading:
 *
 *   * https://doc.rust-lang.org/rust-by-example/error/multiple_error_types.html
 *   * https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html
 *   * https://stevedonovan.github.io/rust-gentle-intro/6-error-handling.html
 *
 * Useful documentation:
 *
 *   * https://doc.rust-lang.org/beta/core/error/trait.Error.html
 *
 */


/// The libgreat Error trait
pub trait Error: core::fmt::Debug {
    type Error: Error;
    fn kind(&self) -> Self::Error; // TODO hrmm... can we just return Self ?
}

/// Defines an error type, to be used by any other traits.
pub trait ErrorType {
    /// Error type
    type Error: Error;
}

impl<T: ErrorType> ErrorType for &mut T {
    type Error = T::Error;
}

/// Result<T>
pub type Result<T> = core::result::Result<T, &'static (dyn core::error::Error + 'static)>;

#[cfg(test)]
mod tests {
    use super::*;

    // - fixtures -------------------------------------------------------------

    /// A Custom ErrorKind
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    #[non_exhaustive] // ... is a double-edged sword
    pub enum CustomErrorKind {
        One,
        Two,
        Unknown,
    }

    // trait: libgreat::Error
    impl Error for CustomErrorKind {
        type Error = CustomErrorKind;
        fn kind(&self) -> Self::Error {
            *self
        }
    }

    // trait: core::error::Error
    impl core::error::Error for CustomErrorKind {
        #[allow(deprecated)]
        fn description(&self) -> &str {
            use CustomErrorKind::*;
            match self {
                One => "This is a One error",
                Two => "This is a Two error",
                Unknown => "This is an Unknown error",
            }
        }
    }

    // trait: core::fmt::Display
    impl core::fmt::Display for CustomErrorKind {
        #[allow(deprecated)]
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            // Or: core::fmt::Debug::fmt(&self, f)
            use core::error::Error;
            write!(f, "{}", self.description())
        }
    }

    // trait: core::num::TryFromIntError
    impl core::convert::From<core::num::TryFromIntError> for CustomErrorKind {
        fn from(_error: core::num::TryFromIntError) -> Self {
            CustomErrorKind::One
        }
    }

    /// returns a `CustomErrorKind`
    fn result_custom(n: u32) -> core::result::Result<u32, CustomErrorKind> {
        if n % 32 == 0 {
            Ok(n)
        } else {
            Err(CustomErrorKind::Two)
        }
    }

    /// returns a `&'static (dyn core::error::Error + 'static)`
    fn result_core_error(n: u32) -> core::result::Result<u32, &'static (dyn core::error::Error)> {
        if n % 32 == 0 {
            Ok(n)
        } else {
            Err(&CustomErrorKind::Unknown)
        }
    }

    // - tests ----------------------------------------------------------------

    #[test]
    fn test_everything() {
        match result_custom(31) {
            Ok(_n) => (),
            Err(CustomErrorKind::Unknown) => {
                println!("Unknown Error");
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }

        match result_core_error(31) {
            Ok(_n) => (),
            Err(e) => {
                println!("Error: {}", e);
                if e.is::<CustomErrorKind>() {
                    println!("  ... which is a custom error");
                }
            }
        }

        match result_custom(31) {
            Ok(_n) => (),
            Err(error) => match error.kind() {
                CustomErrorKind::Unknown => {
                    println!("Unknown Error");
                }
                _ => {
                    println!("Error: {}", error);
                }
            },
        }
    }
}
