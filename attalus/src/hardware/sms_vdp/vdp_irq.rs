use impler::{Cref, Impl, Mref, Ref};

use super::*;

/// Is the VDP requesting an interrupt this rendering line?
pub trait SmsVdpIrq {
    fn get(&self) -> Option<u8>;
}

pub struct SmsVdpIrqImpl;

impl<T> SmsVdpIrq for T
where
    T: Impl<SmsVdpIrqImpl>,
    T::Impler: SmsVdpIrq,
{
    #[inline]
    fn get(&self) -> Option<u8> {
        self.make().get()
    }
}

pub struct SmsVdpIrqImpler<T: ?Sized>(Ref<T>);

impl<T: ?Sized> SmsVdpIrqImpler<T> {
    #[inline(always)]
    pub fn new<'a>(t: &'a T) -> Cref<'a, Self> {
        Cref::Own(SmsVdpIrqImpler(unsafe { Ref::new(t) }))
    }

    #[inline(always)]
    pub fn new_mut<'a>(t: &'a mut T) -> Mref<'a, Self> {
        Mref::Own(SmsVdpIrqImpler(unsafe { Ref::new_mut(t) }))
    }
}

impl<T: ?Sized> SmsVdpIrq for SmsVdpIrqImpler<T>
where
    T: SmsVdpInternal,
{
    #[inline]
    fn get(&self) -> Option<u8> {
        self.0._0().requesting_mi()
    }
}
