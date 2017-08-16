
use super::*;

impl MemoryMap for [u8; 0x10000] {
    fn read(&self, logical_address: u16) -> u8 {
        self[logical_address as usize]
    }
    fn write(&mut self, logical_address: u16, value: u8) {
        self[logical_address as usize] = value;
    }
}
