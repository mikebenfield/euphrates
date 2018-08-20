//! The IO system of the Sega Master System.

use std::cell::RefCell;
use std::rc::Rc;

use super::io16::Io16;
use super::sms_player_input::SmsPlayerInput;
use super::sms_vdp::{SmsVdpInterface, SmsVdpInternal};
use super::sn76489::Sn76489Interface;

pub struct SmsIo16Impler<'a, V: 'a + ?Sized, S: 'a + ?Sized> {
    pub vdp: Rc<RefCell<&'a mut V>>,
    pub sn76489: &'a mut S,
    pub player_input: SmsPlayerInput,
}

impl<'a, V: 'a, S: 'a> Io16 for SmsIo16Impler<'a, V, S>
where
    V: SmsVdpInterface + SmsVdpInternal + ?Sized,
    S: Sn76489Interface + ?Sized,
{
    fn input(&mut self, address: u16) -> u8 {
        use hardware::sms_vdp::Kind;

        let masked = (address & 0b11000001) as u8;
        let value = match masked {
            0b00000000 => {
                match (self.vdp.borrow().kind(), self.player_input.pause()) {
                    (Kind::Gg, true) => 0,
                    (Kind::Gg, false) => 0x80,
                    // This is what the SMS 2 does. In the original SMS, reads
                    // give the last byte of the instruction which read the
                    // port. I'm not implementing that for now or hopefully
                    // ever.
                    _ => 0xFF,
                }
            }
            0b00000001 => {
                // ditto
                0xFF
            }
            0b01000000 => {
                // V counter
                self.vdp.borrow_mut().read_v()
            }
            0b01000001 => {
                // H counter
                self.vdp.borrow_mut().read_h()
            }
            0b10000000 => {
                // VDP data
                self.vdp.borrow_mut().read_data()
            }
            0b10000001 => {
                // VDP control
                self.vdp.borrow_mut().read_control()
            }
            0b11000000 => {
                // IO port A/B register
                self.player_input.joypad_a()
            }
            0b11000001 => {
                // IO port B register
                self.player_input.joypad_b()
            }
            _ => {
                unreachable!("Missing IO address in input");
            }
        };

        value
    }

    fn output(&mut self, address: u16, value: u8) {
        let masked = (address & 0b11000001) as u8;

        match masked {
            0b00000000 => {
                // This is supposed to write to the IO system's memory control.
                // It doesn't seem necessary to emulate this.
            }
            0b00000001 => {
                // This is supposed to write to the IO system's IO control. It
                // doesn't seem necessary to emulate this.
            }
            0b01000000 =>
                // SN76489 write
                self.sn76489.write(value),
            0b01000001 =>
                // SN76489 write
                self.sn76489.write(value),
            0b10000000 =>
                // VDP data port write
                self.vdp.borrow_mut().write_data(value),
            0b10000001 =>
                // VDP control port write
                self.vdp.borrow_mut().write_control(value),
            _ => {}
        }
    }
}
