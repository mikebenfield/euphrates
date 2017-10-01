// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use ::message::{Receiver, Sender};

use super::*;

#[derive(Copy)]
pub struct SimpleMemoryMap {
    contents: [u8; 0x10000],
    id: u32,
}

impl Clone for SimpleMemoryMap {
    fn clone(&self) -> Self {
        *self
    }
}

impl Default for SimpleMemoryMap {
    fn default() -> Self {
        SimpleMemoryMap {
            contents: [0u8; 0x10000],
            id: 0,
        }
    }
}

impl SimpleMemoryMap {
    pub fn new(contents: [u8; 0x10000]) -> SimpleMemoryMap {
        SimpleMemoryMap {
            contents: contents,
            id: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum SimpleMemoryMapMessage {
    Read {
        address: u16,
        value: u8,
    },

    Write {
        address: u16,
        value: u8,
    }
}

impl Sender for SimpleMemoryMap {
    type Message = SimpleMemoryMapMessage;

    fn id(&self) -> u32 { self.id }
    fn set_id(&mut self, id: u32) { self.id = id }
}

impl MemoryMap for SimpleMemoryMap {
    fn read<R>(&self, receiver: &mut R, logical_address: u16) -> u8
    where
        R: Receiver<SimpleMemoryMapMessage>
    {
        let value = self.contents[logical_address as usize];
        receiver.receive(
            self.id(),
            SimpleMemoryMapMessage::Read {
                address: logical_address,
                value: value,
            }
        );
        value
    }

    fn write<R>(&mut self, receiver: &mut R, logical_address: u16, value: u8)
    where
        R: Receiver<SimpleMemoryMapMessage>
    {
        receiver.receive(
            self.id(),
            SimpleMemoryMapMessage::Write {
                address: logical_address,
                value: value,
            }
        );
        self.contents[logical_address as usize] = value;
    }
}

impl SliceMemoryMap for SimpleMemoryMap {
    fn slice(&self) -> &[u8] {
        &self.contents
    }

    fn slice_mut(&mut self) -> &mut [u8] {
        &mut self.contents
    }
}
