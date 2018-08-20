//! 16 bit memory maps.

/// A memory map with 16 bit addresses.
pub trait Memory16 {
    /// Read a byte.
    fn read(&mut self, logical_address: u16) -> u8;

    /// Write a byte.
    fn write(&mut self, logical_address: u16, value: u8);
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
