// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Sn76489 {
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

impl Default for Sn76489 {
    fn default() -> Sn76489 {
        Sn76489 {
            registers: [0, 0xF, 0, 0xF, 0, 0xF, 0, 0xF],
            latch: 0,
            linear_feedback: 0x8000,
            counters: [1, 1, 1, 1],
            polarity: [1, 1, 1, 1],
            cycles: 0,
        }
    }

}

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

impl Sn76489 {
    pub fn write(&mut self, x: u8) {
        if x & 0x80 != 0 {
            // latch
            self.latch = (x & 0x70) >> 4;
            let reg = &mut self.registers[self.latch as usize];
            *reg = (*reg & 0xFFF0) | (x as u16 & 0x0F);
        } else {
            // data
            let reg = &mut self.registers[self.latch as usize];
            if self.latch % 2 == 0 {
                // tone register
                *reg = (*reg & 0xFC0F) | ((x as u16 & 0x3F) << 4);
            } else {
                // volume
                *reg = (x as u16) & 0x0F;
            }
        }
        if self.latch == 6 {
            self.linear_feedback = 0x8000;
        }
    }

    pub fn queue<F>(
        &mut self,
        target_cycles: u64,
        out: &mut [i16],
        queue_f: F,
    )
    where F: Fn(&mut [i16]) {
        if self.cycles + 40000 >= target_cycles {
            return;
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
            convert_volume(self.registers[1]),
            convert_volume(self.registers[3]),
            convert_volume(self.registers[5]),
            convert_volume(self.registers[7]),
        ];
        let mut i: usize = 0;
        while i < out.len() {
            let tone0 = self.polarity[0] as i16 * amplitudes[0];
            let tone1 = self.polarity[1] as i16 * amplitudes[1];
            let tone2 = self.polarity[2] as i16 * amplitudes[2];
            let noise = self.polarity[3] as i16 * amplitudes[3];
            let sum = tone0 + tone1 + tone2 + noise;
            debug_assert!(out.len() <= u16::max_value() as usize);
            let count = min_nonzero!(
                (out.len() - i) as u16,
                self.counters[0],
                self.counters[1],
                self.counters[2],
                self.counters[3]
            );
            let last_idx = count as usize + i;
            for j in i .. last_idx as usize {
                out[j] = sum;
            }
            for j in 0 .. 3 {
                self.counters[j] -= count;
                let tone_reg = self.registers[2 * j];
                if tone_reg == 0 || tone_reg == 1 {
                    self.polarity[j] = 1;
                    self.counters[j] = 0x3FF;
                } else if self.counters[j] == 0 {
                    self.polarity[j] *= -1;
                    self.counters[j] = tone_reg;
                }
            }
            self.counters[3] -= count;
            if self.counters[3] == 0 {
                self.counters[3] = match 0x3 & self.registers[6] {
                    0 => 0x20,
                    1 => 0x40,
                    2 => 0x80,
                    _ => 2 * self.registers[4],
                };
                let bit0 = 1 & self.linear_feedback;
                let bit0_shifted = 1 << 15;
                self.polarity[3] = 2 * (bit0 as i8) - 1;
                if self.registers[6] & 4 != 0 {
                    // white noise
                    let bit3_shifted = (8 & self.linear_feedback) << 12;
                    let feed_bit = bit0_shifted ^ bit3_shifted;
                    self.linear_feedback = feed_bit | (self.linear_feedback >> 1);
                } else {
                    // "periodic noise"
                    self.linear_feedback = bit0_shifted | (self.linear_feedback >> 1);
                }
            }
            i = last_idx;
        }
        queue_f(out);
        self.cycles += out.len() as u64;
        self.queue(target_cycles, out, queue_f);
    }
}
