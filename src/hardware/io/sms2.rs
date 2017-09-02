// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use ::log;

use super::*;
use ::sdl_wrap::event::HostIo;
use ::hardware::irq;
use ::hardware::vdp;
use ::hardware::memory_map::{MemoryMap};

pub struct Sms2Io<M: MemoryMap> {
    memory_control: u8,
    io_control: u8,
    mem: M,
    host_io: HostIo,
    pub vdp: vdp::Vdp,
}

impl<M: MemoryMap> Sms2Io<M> {
    pub fn new(mm: M, host_io: HostIo) -> Sms2Io<M> {
        let mut vdp: vdp::Vdp = Default::default();
        vdp.version = vdp::Version::SMS2;
        Sms2Io {
            host_io: host_io,
            vdp: vdp,
            memory_control: 0,
            io_control: 0,
            mem: mm,
        }
    }
}

impl<M: MemoryMap> irq::Irq for Sms2Io<M> {
    fn requesting_mi(&self) -> bool {
        self.vdp.requesting_mi()
    }
    fn requesting_nmi(&self) -> bool {
        self.vdp.requesting_nmi()
    }
}

impl<M: MemoryMap> Io for Sms2Io<M> {
    type MemoryMap = M;

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
                    self.host_io.joypada()
                }
                0b11000001 => {
                    // IO port B register
                    self.host_io.joypadb()
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
        log_major!("Io: output to address {:0>4X}", address);
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

    fn mem(&self) -> &M {
        &self.mem
    }

    fn mem_mut(&mut self) -> &mut M {
        &mut self.mem
    }
}
