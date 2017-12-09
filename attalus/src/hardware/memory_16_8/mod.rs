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
pub trait Machine {
    fn read(&mut self, logical_address: u16) -> u8;
    fn write(&mut self, logical_address: u16, value: u8);
}

pub trait ComponentOf<T>
where
    T: ?Sized,
{
    fn read(&mut T, logical_address: u16) -> u8;
    fn write(&mut T, logical_address: u16, value: u8);
}

pub trait MachineImpl {
    type C: ComponentOf<Self>;
}

impl<T> Machine for T
where
    T: MachineImpl,
{
    #[inline(always)]
    fn read(&mut self, logical_address: u16) -> u8 {
        <T::C as ComponentOf<Self>>::read(self, logical_address)
    }

    #[inline(always)]
    fn write(&mut self, logical_address: u16, value: u8) {
        <T::C as ComponentOf<Self>>::write(self, logical_address, value)
    }
}

impl<T> ComponentOf<T> for [u8; 0x10000]
where
    T: AsMut<[u8; 0x10000]> + AsRef<[u8; 0x10000]>,
{
    fn read(t: &mut T, logical_address: u16) -> u8 {
        t.as_ref()[logical_address as usize]
    }

    fn write(t: &mut T, logical_address: u16, value: u8) {
        t.as_mut()[logical_address as usize] = value
    }
}

impl Machine for [u8; 0x10000] {
    fn read(&mut self, logical_address: u16) -> u8 {
        self[logical_address as usize]
    }

    fn write(&mut self, logical_address: u16, value: u8) {
        self[logical_address as usize] = value
    }
}
