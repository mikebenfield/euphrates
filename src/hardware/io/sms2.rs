// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify this file
// under the terms of the GNU General Public License, version 3, as published by
// the Free Software Foundation. You should have received a copy of the GNU
// General Public License along with Attalus. If not, see
// <http://www.gnu.org/licenses/>.

use ::log;

use super::*;
use ::hardware::vdp;
use ::hardware::memory_map::sega_memory_map::*;

pub struct Sms2Io {
    memory_control: u8,
    io_control: u8,
    ab: u8,
    b_misc: u8,
    mem: SegaMemoryMap,
    pub vdp: vdp::Vdp,
}

impl Sms2Io {
    pub fn new(smm: SegaMemoryMap) -> Sms2Io {
        Sms2Io {
            vdp: Default::default(),
            memory_control: 0,
            io_control: 0,
            ab: 0,
            b_misc: 0,
            mem: smm,

        }
    }
}

impl Io for Sms2Io {
    type MemoryMap = SegaMemoryMap;

    fn input(&mut self, address: u16) -> u8 {
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
                    self.vdp.read_v()
                }
                0b01000001 => {
                    // H counter
                    self.vdp.read_h()
                }
                0b10000000 => {
                    // VDP data
                    self.vdp.read_data()
                }
                0b10000001 => {
                    // VDP control
                    self.vdp.read_control()
                }
                0b11000000 => {
                    // IO port A/B register
                    self.ab
                }
                0b11000001 => {
                    self.b_misc
                }
                _ => {
                    panic!("Missing IO address in input");
                }
            };

        log_minor!("Io: input: address {:0>4X}, value 0x{:0>2X}", address, value);

        value
    }

    fn output(&mut self, address: u16, x: u8) {
        let masked = (address & 0b11000001) as u8;
        match masked {
            0b00000000 => {
                log_major!("Io: output memory control: {:0>2X}", x);
                self.memory_control = x;
            }
            0b00000001 => {
                log_major!("Io: output io control: {:0>2X}", x);
                self.io_control = x;
            }
            0b01000000 => {
                // SN76489 PSG - XXX not implemented
                log_major!("Io: Attempted to output to SN76489 PSG");
            }
            0b01000001 => {
                // SN76489 PSG - XXX not implemented
                log_major!("Io: Attempted to output to SN76489 PSG");
            }
            0b10000000 => {
                // VDP data
                self.vdp.write_data(x);
            }
            0b10000001 => {
                // VDP control
                self.vdp.write_control(x);
            }
            _ => {
                // writes to the remaining addresses have no effect
                log_major!("Io: Attempted to output to bogus address");
            }
        }
        log_minor!("Io: output: address {:0>4X}, value 0x{:0>2X}", address, x);
    }

    fn mem(&self) -> &SegaMemoryMap {
        &self.mem
    }

    fn mem_mut(&mut self) -> &mut SegaMemoryMap {
        &mut self.mem
    }
}
