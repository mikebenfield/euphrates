//! 16 bit memory maps.

/// A memory map with 16 bit addresses.
pub trait Memory16 {
    /// Read a byte.
    fn read(&mut self, logical_address: u16) -> u8;

    /// Write a byte.
    fn write(&mut self, logical_address: u16, value: u8);
}

/// For the Impler pattern for `Memory16`.
pub trait Memory16Impl {
    type Impler: Memory16;

    fn close<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Self::Impler) -> T;

    fn close_mut<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self::Impler) -> T;
}

impl<T> Memory16 for T
where
    T: Memory16Impl + ?Sized,
{
    #[inline]
    fn read(&mut self, logical_address: u16) -> u8 {
        self.close_mut(|z| z.read(logical_address))
    }

    #[inline]
    fn write(&mut self, logical_address: u16, value: u8) {
        self.close_mut(|z| z.write(logical_address, value))
    }
}

impl Memory16 for [u8; 0x10000] {
    #[inline]
    fn read(&mut self, logical_address: u16) -> u8 {
        self[logical_address as usize]
    }

    #[inline]
    fn write(&mut self, logical_address: u16, value: u8) {
        self[logical_address as usize] = value
    }
}
