#![doc = "Peripheral access API for SOC microcontrollers (generated using svd2rust v0.28.0 ( ))\n\nYou can find an overview of the generated API [here].\n\nAPI features to be included in the [next]
svd2rust release can be generated by cloning the svd2rust [repository], checking out the above commit, and running `cargo doc --open`.\n\n[here]: https://docs.rs/svd2rust/0.28.0/svd2rust/#peripheral-api\n[next]: https://github.com/rust-embedded/svd2rust/blob/master/CHANGELOG.md#unreleased\n[repository]: https://github.com/rust-embedded/svd2rust"]
use core::marker::PhantomData;
use core::ops::Deref;
#[allow(unused_imports)]
use generic::*;
#[doc = r"Common register and bit access and modify traits"]
pub mod generic;
#[cfg(feature = "rt")]
extern "C" {
    fn TIMER();
    fn UART();
    fn USB0();
    fn USB0_EP_CONTROL();
    fn USB0_EP_IN();
    fn USB0_EP_OUT();
}
#[doc(hidden)]
pub union Vector {
    pub _handler: unsafe extern "C" fn(),
    pub _reserved: usize,
}
#[cfg(feature = "rt")]
#[doc(hidden)]
#[no_mangle]
pub static __EXTERNAL_INTERRUPTS: [Vector; 6] = [
    Vector { _handler: TIMER },
    Vector { _handler: UART },
    Vector { _handler: USB0 },
    Vector {
        _handler: USB0_EP_CONTROL,
    },
    Vector {
        _handler: USB0_EP_IN,
    },
    Vector {
        _handler: USB0_EP_OUT,
    },
];
#[doc(hidden)]
pub mod interrupt;
pub use self::interrupt::Interrupt;
#[doc = "TIMER"]
pub struct TIMER {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for TIMER {}
impl TIMER {
    #[doc = r"Pointer to the register block"]
    pub const PTR: *const timer::RegisterBlock = 0x8000_1000 as *const _;
    #[doc = r"Return the pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const timer::RegisterBlock {
        Self::PTR
    }
}
impl Deref for TIMER {
    type Target = timer::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::PTR }
    }
}
impl core::fmt::Debug for TIMER {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("TIMER").finish()
    }
}
#[doc = "TIMER"]
pub mod timer;
#[doc = "UART"]
pub struct UART {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for UART {}
impl UART {
    #[doc = r"Pointer to the register block"]
    pub const PTR: *const uart::RegisterBlock = 0x8000_0000 as *const _;
    #[doc = r"Return the pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const uart::RegisterBlock {
        Self::PTR
    }
}
impl Deref for UART {
    type Target = uart::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::PTR }
    }
}
impl core::fmt::Debug for UART {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("UART").finish()
    }
}
#[doc = "UART"]
pub mod uart;
#[doc = "USB0"]
pub struct USB0 {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for USB0 {}
impl USB0 {
    #[doc = r"Pointer to the register block"]
    pub const PTR: *const usb0::RegisterBlock = 0x8000_2000 as *const _;
    #[doc = r"Return the pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const usb0::RegisterBlock {
        Self::PTR
    }
}
impl Deref for USB0 {
    type Target = usb0::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::PTR }
    }
}
impl core::fmt::Debug for USB0 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("USB0").finish()
    }
}
#[doc = "USB0"]
pub mod usb0;
#[doc = "USB0_EP_CONTROL"]
pub struct USB0_EP_CONTROL {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for USB0_EP_CONTROL {}
impl USB0_EP_CONTROL {
    #[doc = r"Pointer to the register block"]
    pub const PTR: *const usb0_ep_control::RegisterBlock = 0x8000_2040 as *const _;
    #[doc = r"Return the pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const usb0_ep_control::RegisterBlock {
        Self::PTR
    }
}
impl Deref for USB0_EP_CONTROL {
    type Target = usb0_ep_control::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::PTR }
    }
}
impl core::fmt::Debug for USB0_EP_CONTROL {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("USB0_EP_CONTROL").finish()
    }
}
#[doc = "USB0_EP_CONTROL"]
pub mod usb0_ep_control;
#[doc = "USB0_EP_IN"]
pub struct USB0_EP_IN {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for USB0_EP_IN {}
impl USB0_EP_IN {
    #[doc = r"Pointer to the register block"]
    pub const PTR: *const usb0_ep_in::RegisterBlock = 0x8000_2080 as *const _;
    #[doc = r"Return the pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const usb0_ep_in::RegisterBlock {
        Self::PTR
    }
}
impl Deref for USB0_EP_IN {
    type Target = usb0_ep_in::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::PTR }
    }
}
impl core::fmt::Debug for USB0_EP_IN {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("USB0_EP_IN").finish()
    }
}
#[doc = "USB0_EP_IN"]
pub mod usb0_ep_in;
#[doc = "USB0_EP_OUT"]
pub struct USB0_EP_OUT {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for USB0_EP_OUT {}
impl USB0_EP_OUT {
    #[doc = r"Pointer to the register block"]
    pub const PTR: *const usb0_ep_out::RegisterBlock = 0x8000_2100 as *const _;
    #[doc = r"Return the pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const usb0_ep_out::RegisterBlock {
        Self::PTR
    }
}
impl Deref for USB0_EP_OUT {
    type Target = usb0_ep_out::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::PTR }
    }
}
impl core::fmt::Debug for USB0_EP_OUT {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("USB0_EP_OUT").finish()
    }
}
#[doc = "USB0_EP_OUT"]
pub mod usb0_ep_out;
#[doc = "LEDS"]
pub struct LEDS {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for LEDS {}
impl LEDS {
    #[doc = r"Pointer to the register block"]
    pub const PTR: *const leds::RegisterBlock = 0x8000_2180 as *const _;
    #[doc = r"Return the pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const leds::RegisterBlock {
        Self::PTR
    }
}
impl Deref for LEDS {
    type Target = leds::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::PTR }
    }
}
impl core::fmt::Debug for LEDS {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("LEDS").finish()
    }
}
#[doc = "LEDS"]
pub mod leds;
#[no_mangle]
static mut DEVICE_PERIPHERALS: bool = false;
#[doc = r" All the peripherals."]
#[allow(non_snake_case)]
pub struct Peripherals {
    #[doc = "TIMER"]
    pub TIMER: TIMER,
    #[doc = "UART"]
    pub UART: UART,
    #[doc = "USB0"]
    pub USB0: USB0,
    #[doc = "USB0_EP_CONTROL"]
    pub USB0_EP_CONTROL: USB0_EP_CONTROL,
    #[doc = "USB0_EP_IN"]
    pub USB0_EP_IN: USB0_EP_IN,
    #[doc = "USB0_EP_OUT"]
    pub USB0_EP_OUT: USB0_EP_OUT,
    #[doc = "LEDS"]
    pub LEDS: LEDS,
}
impl Peripherals {
    #[doc = r" Returns all the peripherals *once*."]
    #[cfg(feature = "critical-section")]
    #[inline]
    pub fn take() -> Option<Self> {
        critical_section::with(|_| {
            if unsafe { DEVICE_PERIPHERALS } {
                return None;
            }
            Some(unsafe { Peripherals::steal() })
        })
    }
    #[doc = r" Unchecked version of `Peripherals::take`."]
    #[doc = r""]
    #[doc = r" # Safety"]
    #[doc = r""]
    #[doc = r" Each of the returned peripherals must be used at most once."]
    #[inline]
    pub unsafe fn steal() -> Self {
        DEVICE_PERIPHERALS = true;
        Peripherals {
            TIMER: TIMER {
                _marker: PhantomData,
            },
            UART: UART {
                _marker: PhantomData,
            },
            USB0: USB0 {
                _marker: PhantomData,
            },
            USB0_EP_CONTROL: USB0_EP_CONTROL {
                _marker: PhantomData,
            },
            USB0_EP_IN: USB0_EP_IN {
                _marker: PhantomData,
            },
            USB0_EP_OUT: USB0_EP_OUT {
                _marker: PhantomData,
            },
            LEDS: LEDS {
                _marker: PhantomData,
            },
        }
    }
}
