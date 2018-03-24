pub mod sms2;

/// A machine that has an IO system with 16 bit addresses and 8 bit data.
pub trait T {
    fn input(&mut self, address: u16) -> u8;
    fn output(&mut self, address: u16, value: u8);
}

/// A component providing IO services to `T`.
pub trait Impler<S>
where
    S: ?Sized,
{
    fn input(s: &mut S, address: u16) -> u8;
    fn output(s: &mut S, address: u16, value: u8);
}

pub trait Impl {
    type Impler: Impler<Self>;
}

impl<S> T for S
where
    S: Impl + ?Sized,
{
    #[inline]
    fn input(&mut self, address: u16) -> u8 {
        <S::Impler as Impler<Self>>::input(self, address)
    }

    #[inline]
    fn output(&mut self, address: u16, value: u8) {
        <S::Impler as Impler<Self>>::output(self, address, value)
    }
}


#[derive(Clone, Copy, Default, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SimpleIo;

impl SimpleIo {
    pub fn new() -> SimpleIo {
        SimpleIo
    }
}

impl<S> Impler<S> for SimpleIo
where
    S: ?Sized,
{
    fn input(_t: &mut S, _address: u16) -> u8 {
        0
    }

    fn output(_t: &mut S, _address: u16, _value: u8) {}
}
