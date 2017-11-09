// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use ::message::{Receiver, Sender};

use super::*;
use ::hardware::irq;
use ::hardware::vdp;
use ::hardware::sn76489;
use ::hardware::memory_map::MemoryMap;

pub struct Sms2Io<M> {
    memory_control: u8,
    io_control: u8,
    mem: M,
    id: u32,
    joypad_a: u8,
    joypad_b: u8,
    pub sn76489: sn76489::Sn76489,
    pub vdp: vdp::Vdp,
}

impl<M> Sms2Io<M> {
    pub fn new(mm: M) -> Sms2Io<M> {
        let mut vdp: vdp::Vdp = Default::default();
        let sn76489: sn76489::Sn76489 = Default::default();
        Sms2Io {
            sn76489: sn76489,
            vdp: vdp,
            memory_control: 0,
            io_control: 0,
            mem: mm,
            id: 0,
            joypad_a: 0xFF,
            joypad_b: 0xFF,
        }
    }

    pub fn joypad_a(&self) -> u8 { self.joypad_a }
    pub fn joypad_b(&self) -> u8 { self.joypad_b }
    pub fn set_joypad_a(&mut self, x: u8) { self.joypad_a = x; }
    pub fn set_joypad_b(&mut self, x: u8) { self.joypad_b = x; }
}


impl<M> irq::Irq for Sms2Io<M> {
    fn requesting_mi(&self) -> Option<u8> {
        self.vdp.requesting_mi()
    }
    fn requesting_nmi(&self) -> bool {
        self.vdp.requesting_nmi()
    }
    fn clear_nmi(&self) {
        self.vdp.clear_nmi();
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Matchable)]
pub enum Sms2IoMessage {
    Input {
        address: u16,
        value: u8,
    },

    Output {
        address: u16,
        value: u8,
    },

    BogusOutput {
        address: u16,
        value: u8,
    }
}

impl<M> Sender for Sms2Io<M> {
    type Message = Sms2IoMessage;

    fn id(&self) -> u32 { self.id }
    fn set_id(&mut self, id: u32) { self.id = id; }
}

impl<M, R> Io<R> for Sms2Io<M>
where
    M: MemoryMap,
    R: Receiver<Sms2IoMessage> + Receiver<vdp::VdpMessage>,
{
    type MemoryMap = M;

    fn input(&mut self, receiver: &mut R, address: u16) -> u8 {
        let masked = (address & 0b11000001) as u8;
        let value =
            match masked {
                0b00000000 => {
                    // XXX - This is what the SMS 2 does. In the original SMS, reads
                    // are supposed to return the last byte of the instruction which
                    // read the port. I'm not implementing that for now.
                    0xFF
                }
                0b00000001 => {
                    // XXX - ditto
                    0xFF
                }
                0b01000000 => {
                    // V counter
                    self.vdp.read_v(receiver)
                }
                0b01000001 => {
                    // H counter
                    self.vdp.read_h(receiver)
                }
                0b10000000 => {
                    // VDP data
                    self.vdp.read_data(receiver)
                }
                0b10000001 => {
                    // VDP control
                    self.vdp.read_control(receiver)
                }
                0b11000000 => {
                    // IO port A/B register
                    self.joypad_a()
                }
                0b11000001 => {
                    // IO port B register
                    self.joypad_b()
                }
                _ => {
                    panic!("Missing IO address in input");
                }
            };

        receiver.receive(
            self.id(),
            Sms2IoMessage::Input {
                address: address,
                value: value,
            }
        );

        value
    }

    fn output(&mut self, receiver: &mut R, address: u16, x: u8) {
        receiver.receive(
            self.id(),
            Sms2IoMessage::Output {
                address: address,
                value: x,
            }
        );

        let masked = (address & 0b11000001) as u8;

        match masked {
            0b00000000 => {
                self.memory_control = x;
            }
            0b00000001 => {
                self.io_control = x;
            }
            0b01000000 => {
                self.sn76489.write(x);
            }
            0b01000001 => {
                self.sn76489.write(x);
            }
            0b10000000 => {
                self.vdp.write_data(receiver, x);
            }
            0b10000001 => {
                self.vdp.write_control(receiver, x);
            }
            _ => {
                // writes to the remaining addresses have no effect
                receiver.receive(
                    self.id(),
                    Sms2IoMessage::BogusOutput {
                        address: address,
                        value: x,
                    }
                );
            }
        }
    }

    fn mem(&self) -> &M {
        &self.mem
    }

    fn mem_mut(&mut self) -> &mut M {
        &mut self.mem
    }
}
