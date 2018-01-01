// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

//! # Sega Master System Memory Mappers
//! The SMS has two memory mappers in common use, the Sega memory mapper and the
//! CodeMasters mapper. Both swap out pages of cartridge ROM according to memory
//! writes in special locations.
//!
//! At the moment only the Sega Memory mapper is implemented (see
//! [`SegaMemoryMapperHardware`]), and it consists of Rust code manually
//! translating logical memory addresses to physical memory. Future plans include
//!
//! 1. Implement the CodeMasters memory mapper;
//! 2. Implement both memory mappers using native system calls to take advantage
//! of the native virtual memory system.
//!
//! [`SegaMemoryMapperHardware`]: implementation/struct.SegaMemoryMapperHardware.html

pub mod sega;
pub mod codemasters;

use std::convert::{AsMut, AsRef};

/// A machine that has a memory map with 16 bit addresses and 8 bit data.
pub trait T {
    fn read(&mut self, logical_address: u16) -> u8;
    fn write(&mut self, logical_address: u16, value: u8);
}

pub trait Impler<S>
where
    S: ?Sized,
{
    fn read(&mut S, logical_address: u16) -> u8;
    fn write(&mut S, logical_address: u16, value: u8);
}

pub trait Impl {
    type Impler: Impler<Self>;
}

impl<S> T for S
where
    S: Impl,
{
    #[inline]
    fn read(&mut self, logical_address: u16) -> u8 {
        <S::Impler as Impler<Self>>::read(self, logical_address)
    }

    #[inline]
    fn write(&mut self, logical_address: u16, value: u8) {
        <S::Impler as Impler<Self>>::write(self, logical_address, value)
    }
}

impl<S> Impler<S> for [u8; 0x10000]
where
    S: AsMut<[u8; 0x10000]> + AsRef<[u8; 0x10000]>,
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

impl T for [u8; 0x10000] {
    #[inline]
    fn read(&mut self, logical_address: u16) -> u8 {
        self[logical_address as usize]
    }

    #[inline]
    fn write(&mut self, logical_address: u16, value: u8) {
        self[logical_address as usize] = value
    }
}
