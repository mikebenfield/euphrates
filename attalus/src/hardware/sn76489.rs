//! The SN76489 is the sound chip in the Sega Master System and Sega Game Gear.

use failure::Error;

use host_multimedia::SimpleAudio;
use impler::{Cref, Impl, Mref, Ref};

/// The hardware interface for the SN76489 sound chip.
pub trait Sn76489Interface {
    fn write(&mut self, data: u8);
}

pub struct Sn76489InterfaceImpl;

impl<T> Sn76489Interface for T
where
    T: Impl<Sn76489InterfaceImpl> + ?Sized,
    T::Impler: Sn76489Interface,
{
    #[inline]
    fn write(&mut self, data: u8) {
        self.make_mut().write(data)
    }
}

pub trait Sn76489Audio {
    fn queue(&mut self, target_cycles: u64) -> Result<(), Error>;
    fn hold(&mut self);
}

pub struct Sn76489AudioImpl;

impl<T: ?Sized> Sn76489Audio for T
where
    T: Impl<Sn76489AudioImpl>,
    T::Impler: Sn76489Audio,
{
    #[inline]
    fn queue(&mut self, target_cycles: u64) -> Result<(), Error> {
        self.make_mut().queue(target_cycles)
    }

    #[inline]
    fn hold(&mut self) {
        self.make_mut().hold()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Sn76489State {
    // registers for the 4 different channels, in this order:
    // [channel 0 tone], [channel 0 volume], [channel 1 tone], [channel 1 volume],
    // etc.
    // We only need 10 bits for tone and 4 for volume, but if we just make
    // everything a u16 we can avoid some branches
    pub registers: [u16; 8],
    pub latch: u8,
    pub linear_feedback: u16,
    pub counters: [u16; 4],
    pub polarity: [i8; 4],
    pub cycles: u64,
}

impl Default for Sn76489State {
    fn default() -> Self {
        Sn76489State {
            registers: [0, 0xF, 0, 0xF, 0, 0xF, 0, 0xF],
            latch: 0,
            linear_feedback: 0x8000,
            counters: [1, 1, 1, 1],
            polarity: [1, 1, 1, 1],
            cycles: 0,
        }
    }
}

impl Sn76489Interface for Sn76489State {
    fn write(&mut self, data: u8) {
        if data & 0x80 != 0 {
            // latch
            self.latch = (data & 0x70) >> 4;
            let reg = &mut self.registers[self.latch as usize];
            *reg = (*reg & 0xFFF0) | (data as u16 & 0x0F);
        } else {
            // data
            let reg = &mut self.registers[self.latch as usize];
            if self.latch % 2 == 0 {
                // tone register
                *reg = (*reg & 0xFC0F) | ((data as u16 & 0x3F) << 4);
            } else {
                // volume
                *reg = (data as u16) & 0x0F;
            }
        }
        if self.latch == 6 {
            self.linear_feedback = 0x8000;
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

pub struct SimpleSn76489AudioImpler<T: ?Sized>(Ref<T>);

impl<T: ?Sized> SimpleSn76489AudioImpler<T> {
    #[inline(always)]
    pub fn new<'a>(t: &'a T) -> Cref<'a, Self> {
        Cref::Own(SimpleSn76489AudioImpler(unsafe { Ref::new(t) }))
    }

    #[inline(always)]
    pub fn new_mut<'a>(t: &'a mut T) -> Mref<'a, Self> {
        Mref::Own(SimpleSn76489AudioImpler(unsafe { Ref::new_mut(t) }))
    }
}

impl<T> Sn76489Audio for SimpleSn76489AudioImpler<T>
where
    T: AsRef<Sn76489State> + AsMut<Sn76489State> + SimpleAudio + ?Sized,
{
    fn queue(&mut self, target_cycles: u64) -> Result<(), Error> {
        if self.0._0().as_ref().cycles >= target_cycles {
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
            convert_volume(self.0._0().as_ref().registers[1]),
            convert_volume(self.0._0().as_ref().registers[3]),
            convert_volume(self.0._0().as_ref().registers[5]),
            convert_volume(self.0._0().as_ref().registers[7]),
        ];

        {
            let s = self.0.mut_0();
            let mut i: usize = 0;
            while i < s.buffer_len() {
                let tone0 = s.as_ref().polarity[0] as i16 * amplitudes[0];
                let tone1 = s.as_ref().polarity[1] as i16 * amplitudes[1];
                let tone2 = s.as_ref().polarity[2] as i16 * amplitudes[2];
                let noise = s.as_ref().polarity[3] as i16 * amplitudes[3];
                let sum = tone0 + tone1 + tone2 + noise;
                debug_assert!(s.buffer_len() <= u16::max_value() as usize);
                let count = min_nonzero!(
                    (s.buffer_len() - i) as u16,
                    s.as_ref().counters[0],
                    s.as_ref().counters[1],
                    s.as_ref().counters[2],
                    s.as_ref().counters[3]
                );
                let last_idx = count as usize + i;
                for j in i..last_idx as usize {
                    s.buffer_set(j, sum);
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
                        _ => 2 * s.as_mut().registers[4],
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
                        s.as_mut().linear_feedback =
                            bit0_shifted | (s.as_ref().linear_feedback >> 1);
                    }
                }
                i = last_idx;
            }

            s.as_mut().cycles += s.buffer_len() as u64;

            s.queue_buffer()?;
        }

        self.queue(target_cycles)
    }

    fn hold(&mut self) {
        unimplemented!();
    }
}

/// A `Sn76489Impler` that doesn't actually do anything.
///
/// If you don't need sound and want to save a bit of time and memory, use this.
/// (Or just write the empty implementations yourself.)
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct FakeSn76489;

impl Sn76489Interface for FakeSn76489 {
    #[inline]
    fn write(&mut self, _data: u8) {}
}

impl Sn76489Audio for FakeSn76489 {
    #[inline]
    fn queue(&mut self, _target_cycles: u64) -> Result<(), Error> {
        Ok(())
    }

    #[inline]
    fn hold(&mut self) {}
}
