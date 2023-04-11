#[doc = r"Register block"]
#[repr(C)]
pub struct RegisterBlock {
    #[doc = "0x00 - gpioa mode register"]
    pub mode: MODE,
    _reserved1: [u8; 0x04],
    #[doc = "0x08 - gpioa odr register"]
    pub odr: ODR,
    _reserved2: [u8; 0x04],
    #[doc = "0x10 - gpioa idr register"]
    pub idr: IDR,
}
#[doc = "mode (rw) register accessor: an alias for `Reg<MODE_SPEC>`"]
pub type MODE = crate::Reg<mode::MODE_SPEC>;
#[doc = "gpioa mode register"]
pub mod mode;
#[doc = "odr (w) register accessor: an alias for `Reg<ODR_SPEC>`"]
pub type ODR = crate::Reg<odr::ODR_SPEC>;
#[doc = "gpioa odr register"]
pub mod odr;
#[doc = "idr (r) register accessor: an alias for `Reg<IDR_SPEC>`"]
pub type IDR = crate::Reg<idr::IDR_SPEC>;
#[doc = "gpioa idr register"]
pub mod idr;
