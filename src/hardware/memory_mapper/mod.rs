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

pub mod implementation;

use std;
use std::error::Error;
use std::fmt;

use log::Log;

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

/// Users of memory mappers will access them through this trait. This is largely
/// so that users won't have to pass around a Log with every memory read and
/// write.
pub trait MemoryMapper: Log {
    fn read(&mut self, i: u16) -> u8;
    fn write(&mut self, i: u16, v: u8);
    fn check_ok(&self) -> Result<(), MemoryMapError>;
}
