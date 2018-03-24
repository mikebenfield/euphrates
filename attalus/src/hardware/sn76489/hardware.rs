pub trait T {
    fn write(&mut self, data: u8);
}

pub trait Impler<S>
where
    S: ?Sized,
{
    fn write(&mut S, data: u8);
}

pub trait Impl {
    type Impler: Impler<Self>;
}

impl<S> T for S
where
    S: Impl
{
    #[inline]
    fn write(&mut self, data: u8) {
        <<S as Impl>::Impler as Impler<Self>>::write(self, data)
    }
}
