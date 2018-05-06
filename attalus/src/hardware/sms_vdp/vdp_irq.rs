use impler::{ConstOrMut, Impler, ImplerImpl};

use super::*;

/// Is the VDP requesting an interrupt this rendering line?
pub trait SmsVdpIrq {
    fn get(&self) -> Option<u8>;
}

pub trait SmsVdpIrqImpl {
    type Impler: SmsVdpIrq + ?Sized;

    fn close<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Self::Impler) -> T;

    fn close_mut<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self::Impler) -> T;
}

impl<T> SmsVdpIrq for T
where
    T: SmsVdpIrqImpl,
{
    #[inline]
    fn get(&self) -> Option<u8> {
        self.close(|z| z.get())
    }
}

pub struct SmsVdpIrqImpler<T: ?Sized>(ConstOrMut<T>);

unsafe impl<T: ?Sized> ImplerImpl for SmsVdpIrqImpler<T> {
    type T = T;

    #[inline]
    unsafe fn new(c: ConstOrMut<Self::T>) -> Self {
        SmsVdpIrqImpler(c)
    }

    #[inline]
    fn get(&self) -> &ConstOrMut<Self::T> {
        &self.0
    }

    #[inline]
    fn get_mut(&mut self) -> &mut ConstOrMut<Self::T> {
        &mut self.0
    }
}

impl<T: ?Sized> SmsVdpIrq for SmsVdpIrqImpler<T>
where
    T: SmsVdpInternal,
{
    #[inline]
    fn get(&self) -> Option<u8> {
        self._0().requesting_mi()
    }
}
