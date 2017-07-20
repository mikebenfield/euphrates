use log::Log;

use super::vdp::Vdp;

#[derive(Clone, Copy, Default)]
pub struct IoHardware {
    memory_control: u8,
    io_control: u8,
    ab: u8,
    b_misc: u8,
}

pub trait Io: Log + Vdp {
    fn get_io_hardware(&self) -> &IoHardware;

    fn get_mut_io_hardware(&mut self) -> &mut IoHardware;

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
                    self.read_v()
                }
                0b01000001 => {
                    // H counter
                    self.read_h()
                }
                0b10000000 => {
                    // VDP data
                    self.read_data()
                }
                0b10000001 => {
                    // VDP control
                    self.read_control()
                }
                0b11000000 => {
                    // IO port A/B register
                    self.get_io_hardware().ab
                }
                0b11000001 => {
                    self.get_io_hardware().b_misc
                }
                _ => {
                    panic!("Missing IO address in input");
                }
            };

        log_minor!(self, "Io: input: address {:0>4X}, value 0x{:0>2X}", address, value);

        value
    }

    fn output(&mut self, address: u16, x: u8) {
        let masked = (address & 0b11000001) as u8;
        match masked {
            0b00000000 => {
                log_major!(self, "Io: output memory control: {:0>2X}", x);
                self.get_mut_io_hardware().memory_control = x;
            }
            0b00000001 => {
                log_major!(self, "Io: output io control: {:0>2X}", x);
                self.get_mut_io_hardware().io_control = x;
            }
            0b01000000 => {
                // SN76489 PSG - XXX not implemented
                log_major!(self, "Io: Attempted to output to SN76489 PSG");
            }
            0b01000001 => {
                // SN76489 PSG - XXX not implemented
                log_major!(self, "Io: Attempted to output to SN76489 PSG");
            }
            0b10000000 => {
                // VDP data
                self.write_data(x);
            }
            0b10000001 => {
                // VDP control
                self.write_control(x);
            }
            _ => {
                // writes to the remaining addresses have no effect
                log_major!(self, "Io: Attempted to output to bogus address");
            }
        }
        log_minor!(self, "Io: output: address {:0>4X}, value 0x{:0>2X}", address, x);
    }
}
