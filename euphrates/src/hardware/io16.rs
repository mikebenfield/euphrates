//! IO system with 16 bit addresses and 8 bit data.

use impler::Impl;

/// IO system with 16 bit addresses and 8 bit data.
pub trait Io16 {
    /// Receive an input byte at the given `address`.
    fn input(&mut self, address: u16) -> u8;

    /// Output a byte at the given `address`.
    fn output(&mut self, address: u16, value: u8);
}

pub struct Io16Impl;

impl<T> Io16 for T
where
    T: Impl<Io16Impl> + ?Sized,
    T::Impler: Io16,
{
    #[inline]
    fn input(&mut self, address: u16) -> u8 {
        self.make_mut().input(address)
    }

    #[inline]
    fn output(&mut self, address: u16, value: u8) {
        self.make_mut().output(address, value)
    }
}
