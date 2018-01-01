// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use std::convert::{AsMut, AsRef};

use hardware::irq::Irq;
use hardware::sn76489;
use hardware::vdp;
use memo::{Inbox, Outbox};
use super::Impler;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Matchable)]
pub enum Memo {
    Input { address: u16, value: u8 },

    Output { address: u16, value: u8 },

    BogusOutput { address: u16, value: u8 },
}

/// The IO system in the Sega Master Sytem 2. It's almost identical to that in
/// the original Sega Master System, but a little simpler to implement.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct T {
    memory_control: u8,
    io_control: u8,
    id: u32,
    joypad_a: u8,
    joypad_b: u8,
    pause: bool,
}

impl Outbox for T {
    type Memo = Memo;

    #[inline]
    fn id(&self) -> u32 {
        self.id
    }

    #[inline]
    fn set_id(&mut self, id: u32) {
        self.id = id;
    }
}

impl Default for T {
    fn default() -> Self {
        T {
            memory_control: 0,
            io_control: 0,
            id: 0,
            joypad_a: 0xFF,
            joypad_b: 0xFF,
            pause: false,
        }
    }
}

impl T {
    pub fn new() -> T {
        Default::default()
    }

    #[inline]
    pub fn joypad_a(&self) -> u8 {
        self.joypad_a
    }

    #[inline]
    pub fn set_joypad_a(&mut self, x: u8) {
        self.joypad_a = x;
    }

    #[inline]
    pub fn joypad_b(&self) -> u8 {
        self.joypad_b
    }

    #[inline]
    pub fn set_joypad_b(&mut self, x: u8) {
        self.joypad_b = x;
    }

    #[inline]
    pub fn pause(&self) -> bool {
        self.pause
    }

    #[inline]
    pub fn set_pause(&mut self, x: bool) {
        self.pause = x;
    }
}

impl Irq for T {
    #[inline]
    fn requesting_nmi(&self) -> bool {
        self.pause
    }

    #[inline]
    fn clear_nmi(&mut self) {
        self.pause = false
    }
}

impl<S> Impler<S> for T
where
    S: AsMut<T>
        + AsRef<T>
        + vdp::Machine
        + sn76489::Machine
        + Inbox<Memo>
        + ?Sized,
{
    fn input(s: &mut S, address: u16) -> u8 {
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
                s.read_v()
            }
            0b01000001 => {
                // H counter
                s.read_h()
            }
            0b10000000 => {
                // VDP data
                s.read_data()
            }
            0b10000001 => {
                // VDP control
                s.read_control()
            }
            0b11000000 => {
                // IO port A/B register
                AsRef::<T>::as_ref(s).joypad_a()
            }
            0b11000001 => {
                // IO port B register
                AsRef::<T>::as_ref(s).joypad_b()
            }
            _ => {
                unreachable!("Missing IO address in input");
            }
        };

        let id = AsRef::<T>::as_ref(s).id();
        s.receive(
            id,
            Memo::Input {
                address: address,
                value: value,
            },
        );

        value
    }

    fn output(s: &mut S, address: u16, value: u8) {
        let id = AsRef::<T>::as_ref(s).id();

        s.receive(id, Memo::Output { address, value });

        let masked = (address & 0b11000001) as u8;

        match masked {
            0b00000000 => {
                AsMut::<T>::as_mut(s).memory_control = value;
            }
            0b00000001 => {
                AsMut::<T>::as_mut(s).io_control = value;
            }
            0b01000000 => s.write_sound(value),
            0b01000001 => s.write_sound(value),
            0b10000000 => {
                s.write_data(value);
            }
            0b10000001 => {
                s.write_control(value);
            }
            _ => {
                // writes to the remaining addresses have no effect
                s.receive(id, Memo::BogusOutput { address, value });
            }
        }
    }
}
