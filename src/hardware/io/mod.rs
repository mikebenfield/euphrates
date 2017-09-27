// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

pub mod sms2;

use ::message::{Receiver, Sender};
use super::memory_map::*;
use super::irq::*;

pub trait Io<R>: Irq + Sender {
    type MemoryMap: MemoryMap + ?Sized;
    /// Yes, `self` does need to be mutable, because some components may change
    /// when read from; for instance, the VDP
    fn input(&mut self, receiver: &mut R, address: u16) -> u8;
    fn output(&mut self, receiver: &mut R, address: u16, x: u8);
    fn mem(&self) -> &Self::MemoryMap;
    fn mem_mut(&mut self) -> &mut Self::MemoryMap;
}

#[derive(Clone, Copy, Default)]
pub struct SimpleIo {
    mem: SimpleMemoryMap,
    id: u32,
}

impl SimpleIo {
    pub fn new(mem: SimpleMemoryMap) -> SimpleIo {
        SimpleIo {
            mem: mem,
            id: 0,
        }
    }
}

impl Irq for SimpleIo {
    fn requesting_mi(&self) -> Option<u8> { None }
    fn requesting_nmi(&self) -> bool { false }
    fn clear_nmi(&self) {}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum SimpleIoMessage {
    Input {
        address: u16,
    },

    Output {
        address: u16,
        value: u8,
    },
}

impl Sender for SimpleIo {
    type Message = SimpleIoMessage;

    fn id(&self) -> u32 { self.id }
    fn set_id(&mut self, id: u32) { self.id = id; }
}

impl<R> Io<R> for SimpleIo
where
    R: Receiver<SimpleIoMessage>
{
    type MemoryMap = SimpleMemoryMap;

    fn input(&mut self, receiver: &mut R, address: u16) -> u8 {
        receiver.receive(
            self.id(),
            SimpleIoMessage::Input {
                address: address,
            }
        );
        0
    }

    fn output(&mut self, receiver: &mut R, address: u16, value: u8) {
        receiver.receive(
            self.id(),
            SimpleIoMessage::Output {
                address: address,
                value: value,
            }
        );
    }

    fn mem(&self) -> &SimpleMemoryMap {
        &self.mem
    }

    fn mem_mut(&mut self) -> &mut SimpleMemoryMap {
        &mut self.mem
    }
}
