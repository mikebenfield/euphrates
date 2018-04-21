//! 16 bit memory maps.
//!
//! The SMS has two memory mappers in common use, the Sega memory mapper and the
//! CodeMasters mapper. Both swap out pages of cartridge ROM according to memory
//! writes in special locations.
//!
//! At the moment only the Sega Memory mapper works, and it consists of Rust
//! code manually translating logical memory addresses to physical memory.
//! Future plans include
//!
//! 1. Fix the CodeMasters memory mapper;
//! 2. Implement both memory mappers using native system calls to take advantage
//! of the native virtual memory system.

pub mod sega;
pub mod codemasters;

use std::convert::{AsMut, AsRef};

/// A machine that has a memory map with 16 bit addresses.
pub trait Memory16 {
    fn read(&mut self, logical_address: u16) -> u8;
    fn write(&mut self, logical_address: u16, value: u8);
}

pub trait Memory16Impler<S>
where
    S: ?Sized,
{
    fn read(&mut S, logical_address: u16) -> u8;
    fn write(&mut S, logical_address: u16, value: u8);
}

pub trait Memory16Impl {
    type Impler: Memory16Impler<Self>;
}

impl<S> Memory16 for S
where
    S: Memory16Impl + ?Sized,
{
    #[inline]
    fn read(&mut self, logical_address: u16) -> u8 {
        S::Impler::read(self, logical_address)
    }

    #[inline]
    fn write(&mut self, logical_address: u16, value: u8) {
        S::Impler::write(self, logical_address, value)
    }
}

impl<S> Memory16Impler<S> for [u8; 0x10000]
where
    S: AsMut<[u8; 0x10000]> + AsRef<[u8; 0x10000]> + ?Sized,
{
    #[inline]
    fn read(s: &mut S, logical_address: u16) -> u8 {
        s.as_ref()[logical_address as usize]
    }

    #[inline]
    fn write(s: &mut S, logical_address: u16, value: u8) {
        s.as_mut()[logical_address as usize] = value
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
