#![feature(error_in_core)]
#![cfg_attr(not(test), no_std)]

pub mod error;
pub mod smolusb;

pub use error::Result;
