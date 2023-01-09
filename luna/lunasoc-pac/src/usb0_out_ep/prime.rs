#[doc = "Register `prime` writer"]
pub struct W(crate::W<PRIME_SPEC>);
impl core::ops::Deref for W {
    type Target = crate::W<PRIME_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl core::ops::DerefMut for W {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl From<crate::W<PRIME_SPEC>> for W {
    #[inline(always)]
    fn from(writer: crate::W<PRIME_SPEC>) -> Self {
        W(writer)
    }
}
#[doc = "Field `prime` writer - usb0_out_ep prime register field"]
pub type PRIME_W<'a, const O: u8> = crate::BitWriter<'a, u32, PRIME_SPEC, bool, O>;
impl W {
    #[doc = "Bit 0 - usb0_out_ep prime register field"]
    #[inline(always)]
    #[must_use]
    pub fn prime(&mut self) -> PRIME_W<0> {
        PRIME_W::new(self)
    }
    #[doc = "Writes raw bits to the register."]
    #[inline(always)]
    pub unsafe fn bits(&mut self, bits: u32) -> &mut Self {
        self.0.bits(bits);
        self
    }
}
#[doc = "usb0_out_ep prime register\n\nThis register you can [`write_with_zero`](crate::generic::Reg::write_with_zero), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [prime](index.html) module"]
pub struct PRIME_SPEC;
impl crate::RegisterSpec for PRIME_SPEC {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [prime::W](W) writer structure"]
impl crate::Writable for PRIME_SPEC {
    type Writer = W;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: Self::Ux = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: Self::Ux = 0;
}
#[doc = "`reset()` method sets prime to value 0"]
impl crate::Resettable for PRIME_SPEC {
    const RESET_VALUE: Self::Ux = 0;
}
