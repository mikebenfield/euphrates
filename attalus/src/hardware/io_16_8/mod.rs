pub mod sms2;

/// A machine that has an IO system with 16 bit addresses and 8 bit data.
pub trait Io16_8 {
    fn input(&mut self, address: u16) -> u8;
    fn output(&mut self, address: u16, value: u8);
}

pub trait Io16_8Impler<S>
where
    S: ?Sized,
{
    fn input(s: &mut S, address: u16) -> u8;
    fn output(s: &mut S, address: u16, value: u8);
}

pub trait Io16_8Impl {
    type Impler: Io16_8Impler<Self>;
}

impl<S> Io16_8 for S
where
    S: Io16_8Impl + ?Sized,
{
    #[inline]
    fn input(&mut self, address: u16) -> u8 {
        S::Impler::input(self, address)
    }

    #[inline]
    fn output(&mut self, address: u16, value: u8) {
        S::Impler::output(self, address, value)
    }
}


#[derive(Clone, Copy, Default, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SimpleIo;

impl SimpleIo {
    pub fn new() -> SimpleIo {
        SimpleIo
    }
}

impl<S> Io16_8Impler<S> for SimpleIo
where
    S: ?Sized,
{
    fn input(_t: &mut S, _address: u16) -> u8 {
        0
    }

    fn output(_t: &mut S, _address: u16, _value: u8) {}
}
