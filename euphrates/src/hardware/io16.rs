//! IO system with 16 bit addresses and 8 bit data.

/// IO system with 16 bit addresses and 8 bit data.
pub trait Io16 {
    /// Receive an input byte at the given `address`.
    fn input(&mut self, address: u16) -> u8;

    /// Output a byte at the given `address`.
    fn output(&mut self, address: u16, value: u8);
}
