use std::convert::{AsMut, AsRef};

use failure::Error;

use host_multimedia::SimpleAudio;

use super::{hardware, machine};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct T {
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

impl Default for T {
    fn default() -> Self {
        T {
            registers: [0, 0xF, 0, 0xF, 0, 0xF, 0, 0xF],
            latch: 0,
            linear_feedback: 0x8000,
            counters: [1, 1, 1, 1],
            polarity: [1, 1, 1, 1],
            cycles: 0,
        }
    }
}

impl<S> hardware::Impler<S> for T
where
    S: AsMut<T>,
{
    /// If bit 7 is 1, this is a latch byte. In this case,
    /// bits 0-3 are data to be written,
    /// bit 4 is type (0: tone, 1: volume)
    /// bits 5-6 are channel
    /// Write to the lowest 4 bits of the register (top bit is discarded
    /// for the noise regsiter)
    /// If bit 7 is 0, this is a data byte. In this case, we write up
    /// to 6 bits in the upper bits of the latched register.
    fn write(s: &mut S, data: u8) {
        let sn = s.as_mut();
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

impl<S> machine::Impler<S> for T
where
    S: SimpleAudio + AsRef<T> + AsMut<T> + ?Sized,
{
    fn queue(s: &mut S, target_cycles: u64) -> Result<(), Error> {
        if s.as_ref().cycles >= target_cycles {
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
            convert_volume(s.as_ref().registers[1]),
            convert_volume(s.as_ref().registers[3]),
            convert_volume(s.as_ref().registers[5]),
            convert_volume(s.as_ref().registers[7]),
        ];

        {
            let mut i: usize = 0;
            while i < s.buffer().len() {
                let tone0 = s.as_ref().polarity[0] as i16 * amplitudes[0];
                let tone1 = s.as_ref().polarity[1] as i16 * amplitudes[1];
                let tone2 = s.as_ref().polarity[2] as i16 * amplitudes[2];
                let noise = s.as_ref().polarity[3] as i16 * amplitudes[3];
                let sum = tone0 + tone1 + tone2 + noise;
                debug_assert!(s.buffer().len() <= u16::max_value() as usize);
                let count = min_nonzero!(
                    (s.buffer().len() - i) as u16,
                    s.as_ref().counters[0],
                    s.as_ref().counters[1],
                    s.as_ref().counters[2],
                    s.as_ref().counters[3]
                );
                let last_idx = count as usize + i;
                for j in i..last_idx as usize {
                    s.buffer()[j] = sum;
                }
                for j in 0..3 {
                    s.as_mut().counters[j] -= count;
                    let tone_reg = s.as_ref().registers[2 * j];
                    if tone_reg == 0 || tone_reg == 1 {
                        s.as_mut().polarity[j] = 1;
                        s.as_mut().counters[j] = 0x3FF;
                    } else if s.as_ref().counters[j] == 0 {
                        s.as_mut().polarity[j] *= -1;
                        s.as_mut().counters[j] = tone_reg;
                    }
                }
                s.as_mut().counters[3] -= count;
                if s.as_ref().counters[3] == 0 {
                    s.as_mut().counters[3] = match 0x3 & s.as_ref().registers[6] {
                        0 => 0x20,
                        1 => 0x40,
                        2 => 0x80,
                        _ => 2 * s.as_ref().registers[4],
                    };
                    let bit0 = 1 & s.as_ref().linear_feedback;
                    let bit0_shifted = 1 << 15;
                    s.as_mut().polarity[3] = 2 * (bit0 as i8) - 1;
                    if s.as_ref().registers[6] & 4 != 0 {
                        // white noise
                        let bit3_shifted = (8 & s.as_ref().linear_feedback) << 12;
                        let feed_bit = bit0_shifted ^ bit3_shifted;
                        s.as_mut().linear_feedback = feed_bit | (s.as_ref().linear_feedback >> 1);
                    } else {
                        // "periodic noise"
                        s.as_mut().linear_feedback = bit0_shifted |
                            (s.as_ref().linear_feedback >> 1);
                    }
                }
                i = last_idx;
            }

            s.as_mut().cycles += s.buffer().len() as u64;
        }

        s.queue_buffer()?;

        Self::queue(s, target_cycles)
    }
}
