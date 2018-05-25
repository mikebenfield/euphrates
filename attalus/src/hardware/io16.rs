//! IO system with 16 bit addresses and 8 bit data.

/// IO system with 16 bit addresses and 8 bit data.
pub trait Io16 {
    /// Receive an input byte at the given `address`.
    fn input(&mut self, address: u16) -> u8;

    /// Output a byte at the given `address`.
    fn output(&mut self, address: u16, value: u8);
}

/// For the Impler Pattern for `Io16`.
pub trait Io16Impl {
    type Impler: Io16;

    fn close<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Self::Impler) -> T;

    fn close_mut<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self::Impler) -> T;
}

impl<T> Io16 for T
where
    T: Io16Impl + ?Sized,
{
    #[inline]
    fn input(&mut self, address: u16) -> u8 {
        self.close_mut(|z| z.input(address))
    }

    #[inline]
    fn output(&mut self, address: u16, value: u8) {
        self.close_mut(|z| z.output(address, value))
    }
}
