pub trait Z80Irq {
    fn requesting_mi(&self) -> Option<u8>;
    fn requesting_nmi(&mut self) -> bool;
}

pub trait Z80IrqImpl {
    type Impler: Z80Irq + ?Sized;

    fn close<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Self::Impler) -> T;

    fn close_mut<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self::Impler) -> T;
}

impl<T> Z80Irq for T
where
    T: Z80IrqImpl + ?Sized,
{
    #[inline]
    fn requesting_mi(&self) -> Option<u8> {
        self.close(|z| z.requesting_mi())
    }

    #[inline]
    fn requesting_nmi(&mut self) -> bool {
        self.close_mut(|z| z.requesting_nmi())
    }
}
