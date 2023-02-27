#![feature(error_in_core)]
#![feature(panic_info_message)]
#![cfg_attr(not(test), no_std)]

pub mod error;
pub use error::Result;

pub mod gcp;
//pub mod smolusb;
