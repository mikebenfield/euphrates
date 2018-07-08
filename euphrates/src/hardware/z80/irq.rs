use impler::Impl;

pub trait Z80Irq {
    fn requesting_mi(&self) -> Option<u8>;
    fn requesting_nmi(&self) -> bool;
    fn take_nmi(&mut self);
}

pub struct Z80IrqImpl;

impl<T> Z80Irq for T
where
    T: Impl<Z80IrqImpl> + ?Sized,
    T::Impler: Z80Irq,
{
    #[inline(always)]
    fn requesting_mi(&self) -> Option<u8> {
        self.make().requesting_mi()
    }

    #[inline(always)]
    fn requesting_nmi(&self) -> bool {
        self.make().requesting_nmi()
    }

    #[inline(always)]
    fn take_nmi(&mut self) {
        self.make_mut().take_nmi()
    }

}
