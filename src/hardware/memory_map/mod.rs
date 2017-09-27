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

pub mod sega_memory_map;
pub mod codemasters_memory_map;
pub mod simple_memory_map;

use std;
use std::error::Error;
use std::fmt;

use ::message::{Receiver, Sender};
pub use self::sega_memory_map::*;
pub use self::codemasters_memory_map::*;
pub use self::simple_memory_map::*;

#[derive(Clone, Debug)]
pub struct MemoryMapError {
    msg: String
}

impl std::convert::From<std::io::Error> for MemoryMapError {
    fn from(err: std::io::Error) -> MemoryMapError {
        MemoryMapError {
            msg: format!("Error reading file: {}", err.description())
        }
    }
}

impl fmt::Display for MemoryMapError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for MemoryMapError {
    fn description(&self) -> &str {
        &self.msg
    }
}

pub trait MemoryMap: Sender {
    fn read<R>(&self, receiver: &mut R, logical_address: u16) -> u8
    where
        R: Receiver<<Self as Sender>::Message>;
    fn write<R>(&mut self, receiver: &mut R, logical_address: u16, value: u8)
    where
        R: Receiver<<Self as Sender>::Message>;
}

pub trait SliceMemoryMap: MemoryMap {
    fn slice(&self) -> &[u8];
    fn slice_mut(&mut self) -> &mut [u8];
}
