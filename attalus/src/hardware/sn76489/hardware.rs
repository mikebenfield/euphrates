pub trait Sn76489Internal {
    fn write(&mut self, data: u8);
}

pub trait Sn76489InternalImpler<S>
where
    S: ?Sized,
{
    fn write(&mut S, data: u8);
}

pub trait Sn76489InternalImpl {
    type Impler: Sn76489InternalImpler<Self>;
}

impl<S> Sn76489Internal for S
where
    S: Sn76489InternalImpl,
{
    #[inline]
    fn write(&mut self, data: u8) {
        <<S as Sn76489InternalImpl>::Impler as Sn76489InternalImpler<Self>>::write(self, data)
    }
}
