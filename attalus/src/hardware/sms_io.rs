//! The IO system of the Sega Master System.

use impler::{Cref, Mref, Ref};

use super::io16::Io16;
use super::sms_player_input::SmsPlayerInput;
use super::sms_vdp::SmsVdpInterface;
use super::sn76489::Sn76489Interface;

/// An Impler for `Io16`.
///
/// If `T` implements
///
/// * `SmsPlayerInput`,
/// * `SmsVdpInterface`, and
/// * `Sn74689Interface`,
///
/// then `SmsIo16Impler<T>` implements `Io16`.
pub struct SmsIo16Impler<T: ?Sized>(Ref<T>);

impl<T: ?Sized> SmsIo16Impler<T> {
    pub fn new<'a>(t: &'a T) -> Cref<'a, Self> {
        Cref::Own(SmsIo16Impler(unsafe { Ref::new(t) }))
    }

    pub fn new_mut<'a>(t: &'a mut T) -> Mref<'a, Self> {
        Mref::Own(SmsIo16Impler(unsafe { Ref::new_mut(t) }))
    }
}

impl<T> Io16 for SmsIo16Impler<T>
where
    T: SmsPlayerInput + SmsVdpInterface + Sn76489Interface + ?Sized,
{
    fn input(&mut self, address: u16) -> u8 {
        let masked = (address & 0b11000001) as u8;
        let value = match masked {
            0b00000000 => {
                // This is what the SMS 2 does. In the original SMS, reads
                // give the last byte of the instruction which read the
                // port. I'm not implementing that for now or hopefully
                // ever.
                0xFF
            }
            0b00000001 => {
                // ditto
                0xFF
            }
            0b01000000 => {
                // V counter
                self.0.mut_0().read_v()
            }
            0b01000001 => {
                // H counter
                self.0.mut_0().read_h()
            }
            0b10000000 => {
                // VDP data
                self.0.mut_0().read_data()
            }
            0b10000001 => {
                // VDP control
                self.0.mut_0().read_control()
            }
            0b11000000 => {
                // IO port A/B register
                self.0._0().joypad_a()
            }
            0b11000001 => {
                // IO port B register
                self.0._0().joypad_b()
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
                 self.0.mut_0().write(value),
            0b01000001 =>
                // SN76489 write
                 self.0.mut_0().write(value),
            0b10000000 =>
                // VDP data port write
                 self.0.mut_0().write_data(value),
            0b10000001 =>
                // VDP control port write
                 self.0.mut_0().write_control(value),
            _ => {}
        }
    }
}