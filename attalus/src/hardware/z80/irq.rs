use impler::Impl;

pub trait Z80Irq {
    fn requesting_mi(&self) -> Option<u8>;
    fn requesting_nmi(&mut self) -> bool;
}

pub struct Z80IrqImpl;

impl<T> Z80Irq for T
where
    T: Impl<Z80IrqImpl> + ?Sized,
    T::Impler: Z80Irq,
{
    #[inline]
    fn requesting_mi(&self) -> Option<u8> {
        self.make().requesting_mi()
    }

    #[inline]
    fn requesting_nmi(&mut self) -> bool {
        self.make_mut().requesting_nmi()
    }
}
