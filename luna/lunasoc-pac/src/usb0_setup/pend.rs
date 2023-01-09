#[doc = "Register `pend` reader"]
pub struct R(crate::R<PEND_SPEC>);
impl core::ops::Deref for R {
    type Target = crate::R<PEND_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<crate::R<PEND_SPEC>> for R {
    #[inline(always)]
    fn from(reader: crate::R<PEND_SPEC>) -> Self {
        R(reader)
    }
}
#[doc = "Field `pend` reader - usb0_setup pend register field"]
pub type PEND_R = crate::BitReader<bool>;
impl R {
    #[doc = "Bit 0 - usb0_setup pend register field"]
    #[inline(always)]
    pub fn pend(&self) -> PEND_R {
        PEND_R::new((self.bits & 1) != 0)
    }
}
#[doc = "usb0_setup pend register\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [pend](index.html) module"]
pub struct PEND_SPEC;
impl crate::RegisterSpec for PEND_SPEC {
    type Ux = u32;
}
#[doc = "`read()` method returns [pend::R](R) reader structure"]
impl crate::Readable for PEND_SPEC {
    type Reader = R;
}
#[doc = "`reset()` method sets pend to value 0"]
impl crate::Resettable for PEND_SPEC {
    const RESET_VALUE: Self::Ux = 0;
}
