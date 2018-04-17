pub trait Sn76489Hardware {
    fn write(&mut self, data: u8);
}

pub trait Sn76489HardwareImpler<S>
where
    S: ?Sized,
{
    fn write(&mut S, data: u8);
}

pub trait Sn76489HardwareImpl {
    type Impler: Sn76489HardwareImpler<Self>;
}

impl<S> Sn76489Hardware for S
where
    S: Sn76489HardwareImpl,
{
    #[inline]
    fn write(&mut self, data: u8) {
        <<S as Sn76489HardwareImpl>::Impler as Sn76489HardwareImpler<Self>>::write(self, data)
    }
}
