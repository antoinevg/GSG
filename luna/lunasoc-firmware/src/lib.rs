#![no_std]

// - aliases ------------------------------------------------------------------
use lunasoc_pac as pac;


// - modules ------------------------------------------------------------------

// TODO move these into lunasoc-pac
pub mod csr;
pub mod minerva;
pub mod register {
    pub use crate::minerva;
}
