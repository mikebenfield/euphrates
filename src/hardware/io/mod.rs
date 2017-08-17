
pub mod sms2;

use super::memory_map::*;

pub trait Io {
    type MemoryMap: MemoryMap + ?Sized;
    /// Yes, `self` does need to be mutable, because some components may change
    /// when read from; for instance, the VDP
    fn input(&mut self, address: u16) -> u8;
    fn output(&mut self, address: u16, x: u8);
    fn mem(&self) -> &Self::MemoryMap;
    fn mem_mut(&mut self) -> &mut Self::MemoryMap;
}

#[derive(Copy)]
pub struct SimpleIo {
    pub mem: [u8; 0x10000],
}

impl Clone for SimpleIo {
    fn clone(&self) -> SimpleIo {
        *self
    }
}
impl Default for SimpleIo {
    fn default() -> SimpleIo {
        SimpleIo {
            mem: [0; 0x10000],
        }
    }
}

impl Io for SimpleIo {
    type MemoryMap = [u8; 0x10000];
    fn input(&mut self, _: u16) -> u8 { 0 }
    fn output(&mut self, _: u16, _: u8) {}
    fn mem(&self) -> &[u8; 0x10000] {
        &self.mem
    }
    fn mem_mut(&mut self) -> &mut [u8; 0x10000] {
        &mut self.mem
    }
}
