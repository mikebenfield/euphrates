// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use ::errors::*;
use ::has::Has;
use ::host_multimedia::SimpleAudio;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Component {
    // registers for the 4 different channels, in this order:
    // [channel 0 tone], [channel 0 volume], [channel 1 tone], [channel 1 volume],
    // etc.
    // We only need 10 bits for tone and 4 for volume, but if we just make
    // everything a u16 we can avoid some branches
    registers: [u16; 8],
    latch: u8,
    linear_feedback: u16,
    counters: [u16; 4],
    polarity: [i8; 4],
    pub cycles: u64,
}

impl Default for Component {
    fn default() -> Self {
        Component {
            registers: [0, 0xF, 0, 0xF, 0, 0xF, 0, 0xF],
            latch: 0,
            linear_feedback: 0x8000,
            counters: [1, 1, 1, 1],
            polarity: [1, 1, 1, 1],
            cycles: 0,
        }
    }

}

impl<T> super::ComponentOf<T> for Component
where
    T: Has<Component>
{
    fn write_sound(t: &mut T, data: u8) {
        let sn = t.get_mut();
        if data & 0x80 != 0 {
            // latch
            sn.latch = (data & 0x70) >> 4;
            let reg = &mut sn.registers[sn.latch as usize];
            *reg = (*reg & 0xFFF0) | (data as u16 & 0x0F);
        } else {
            // data
            let reg = &mut sn.registers[sn.latch as usize];
            if sn.latch % 2 == 0 {
                // tone register
                *reg = (*reg & 0xFC0F) | ((data as u16 & 0x3F) << 4);
            } else {
                // volume
                *reg = (data as u16) & 0x0F;
            }
        }
        if sn.latch == 6 {
            sn.linear_feedback = 0x8000;
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Emulator;

macro_rules! min_nonzero {
    ($x: expr) => {
        $x
    };
    ($x: expr, $($xs: expr),*) => {
        {
            let prev_min = min_nonzero!($($xs),*);
            if prev_min == 0 || $x < prev_min {
                $x
            } else {
                prev_min
            }
        }
    };
}

impl<A> super::Emulator<A, Component> for Emulator
where
    A: SimpleAudio + ?Sized
{
    fn queue(
        &mut self,
        component: &mut Component,
        target_cycles: u64,
        audio: &mut A,
    ) -> Result<()> {
        if component.cycles >= target_cycles {
            return Ok(());
        }

        fn convert_volume(vol: u16) -> i16 {
            match vol {
                // The first three amplitudes should be as in comments, but it
                // seems in real hardware they are capped. The "correct" values
                // are certainly too loud.
                0 => 2010, // 8000
                1 => 2010, // 5048
                2 => 2010, // 3184
                3 => 2010,
                4 => 1268,
                5 => 800,
                6 => 505,
                7 => 318,
                8 => 201,
                9 => 127,
                10 => 80,
                11 => 50,
                12 => 32,
                13 => 20,
                14 => 13,
                15 => 0,
                _ => unreachable!(),
            }
        }

        let amplitudes: [i16; 4] = [
            convert_volume(component.registers[1]),
            convert_volume(component.registers[3]),
            convert_volume(component.registers[5]),
            convert_volume(component.registers[7]),
        ];

        {
            let buffer = audio.buffer()?;
            let mut i: usize = 0;
            while i < buffer.len() {
                let tone0 = component.polarity[0] as i16 * amplitudes[0];
                let tone1 = component.polarity[1] as i16 * amplitudes[1];
                let tone2 = component.polarity[2] as i16 * amplitudes[2];
                let noise = component.polarity[3] as i16 * amplitudes[3];
                let sum = tone0 + tone1 + tone2 + noise;
                debug_assert!(buffer.len() <= u16::max_value() as usize);
                let count = min_nonzero!(
                    (buffer.len() - i) as u16,
                    component.counters[0],
                    component.counters[1],
                    component.counters[2],
                    component.counters[3]
                );
                let last_idx = count as usize + i;
                for j in i .. last_idx as usize {
                    buffer[j] = sum;
                }
                for j in 0 .. 3 {
                    component.counters[j] -= count;
                    let tone_reg = component.registers[2 * j];
                    if tone_reg == 0 || tone_reg == 1 {
                        component.polarity[j] = 1;
                        component.counters[j] = 0x3FF;
                    } else if component.counters[j] == 0 {
                        component.polarity[j] *= -1;
                        component.counters[j] = tone_reg;
                    }
                }
                component.counters[3] -= count;
                if component.counters[3] == 0 {
                    component.counters[3] = match 0x3 & component.registers[6] {
                        0 => 0x20,
                        1 => 0x40,
                        2 => 0x80,
                        _ => 2 * component.registers[4],
                    };
                    let bit0 = 1 & component.linear_feedback;
                    let bit0_shifted = 1 << 15;
                    component.polarity[3] = 2 * (bit0 as i8) - 1;
                    if component.registers[6] & 4 != 0 {
                        // white noise
                        let bit3_shifted = (8 & component.linear_feedback) << 12;
                        let feed_bit = bit0_shifted ^ bit3_shifted;
                        component.linear_feedback = feed_bit | (component.linear_feedback >> 1);
                    } else {
                        // "periodic noise"
                        component.linear_feedback = bit0_shifted | (component.linear_feedback >> 1);
                    }
                }
                i = last_idx;
            }

            component.cycles += buffer.len() as u64;
        }

        audio.queue_buffer()?;

        self.queue(component, target_cycles, audio)
    }
}
