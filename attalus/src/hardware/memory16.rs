//! 16 bit memory maps.

use impler::Impl;

/// A memory map with 16 bit addresses.
pub trait Memory16 {
    /// Read a byte.
    fn read(&mut self, logical_address: u16) -> u8;

    /// Write a byte.
    fn write(&mut self, logical_address: u16, value: u8);
}

pub struct Memory16Impl;

impl<T> Memory16 for T
where
    T: Impl<Memory16Impl>,
    T::Impler: Memory16,
{
    #[inline(always)]
    fn read(&mut self, logical_address: u16) -> u8 {
        self.make_mut().read(logical_address)
    }

    #[inline(always)]
    fn write(&mut self, logical_address: u16, value: u8) {
        self.make_mut().write(logical_address, value)
    }
}

impl Memory16 for [u8; 0x10000] {
    #[inline(always)]
    fn read(&mut self, logical_address: u16) -> u8 {
        self[logical_address as usize]
    }

    #[inline(always)]
    fn write(&mut self, logical_address: u16, value: u8) {
        self[logical_address as usize] = value
    }
}
