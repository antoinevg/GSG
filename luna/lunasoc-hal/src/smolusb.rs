//! Simple peripheral-level USB stack

pub mod class;
pub mod control;
pub mod descriptor;
pub mod device;
pub mod error;
pub mod traits;
pub use error::ErrorKind;
