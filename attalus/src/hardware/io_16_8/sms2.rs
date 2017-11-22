// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use super::*;
use ::memo::{Inbox, Outbox};
use ::has::Has;
use ::hardware::vdp;
use ::hardware::sn76489;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Matchable)]
pub enum Memo {
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

/// The IO system in the Sega Master Sytem 2. It's almost identical to that in
/// the original Sega Master System, but a little simpler to implement.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Component {
    memory_control: u8,
    io_control: u8,
    id: u32,
    joypad_a: u8,
    joypad_b: u8,
}

impl Outbox for Component {
    type Memo = Memo;

    fn id(&self) -> u32 { self.id }
    fn set_id(&mut self, id: u32) { self.id = id; }
}

impl Default for Component {
    fn default() -> Self {
        Component {
            memory_control: 0,
            io_control: 0,
            id: 0,
            joypad_a: 0xFF,
            joypad_b: 0xFF,
        }
    }
}

impl Component {
    pub fn new() -> Component {
        Default::default()
    }

    pub fn joypad_a(&self) -> u8 { self.joypad_a }
    pub fn joypad_b(&self) -> u8 { self.joypad_b }
    pub fn set_joypad_a(&mut self, x: u8) { self.joypad_a = x; }
    pub fn set_joypad_b(&mut self, x: u8) { self.joypad_b = x; }
}

impl<T> ComponentOf<T> for Component
where
    T: Has<Component> + vdp::Machine + sn76489::Machine + Inbox<Memo>
{
    fn input(t: &mut T, address: u16) -> u8 {
        let masked = (address & 0b11000001) as u8;
        let value =
            match masked {
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
                    t.read_v()
                }
                0b01000001 => {
                    // H counter
                    t.read_h()
                }
                0b10000000 => {
                    // VDP data
                    t.read_data()
                }
                0b10000001 => {
                    // VDP control
                    t.read_control()
                }
                0b11000000 => {
                    // IO port A/B register
                    Has::<Component>::get(t).joypad_a()
                }
                0b11000001 => {
                    // IO port B register
                    Has::<Component>::get(t).joypad_b()
                }
                _ => {
                    unreachable!("Missing IO address in input");
                }
            };

        let id = Has::<Component>::get(t).id();
        t.receive(
            id,
            Memo::Input {
                address: address,
                value: value,
            }
        );

        value
    }

    fn output(t: &mut T, address: u16, value: u8) {
        let id = Has::<Component>::get(t).id();

        t.receive(
            id,
            Memo::Output {
                address,
                value,
            }
        );

        let masked = (address & 0b11000001) as u8;

        match masked {
            0b00000000 => {
                Has::<Component>::get_mut(t).memory_control = value;
            }
            0b00000001 => {
                Has::<Component>::get_mut(t).io_control = value;
            }
            0b01000000 => {
                t.write_sound(value)
            }
            0b01000001 => {
                t.write_sound(value)
            }
            0b10000000 => {
                t.write_data(value);
            }
            0b10000001 => {
                t.write_control(value);
            }
            _ => {
                // writes to the remaining addresses have no effect
                t.receive(
                    id,
                    Memo::BogusOutput {
                        address,
                        value,
                    }
                );
            }
        }
    }
}
