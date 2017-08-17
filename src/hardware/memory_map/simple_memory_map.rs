// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify this file
// under the terms of the GNU General Public License, version 3, as published by
// the Free Software Foundation. You should have received a copy of the GNU
// General Public License along with Attalus. If not, see
// <http://www.gnu.org/licenses/>.

use super::*;

impl MemoryMap for [u8; 0x10000] {
    fn read(&self, logical_address: u16) -> u8 {
        self[logical_address as usize]
    }
    fn write(&mut self, logical_address: u16, value: u8) {
        self[logical_address as usize] = value;
    }
}
