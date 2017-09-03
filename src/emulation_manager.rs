// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use std;
use std::error::Error;

use sdl2;

use ::log;
use ::sdl_wrap::event::HostIo;
use ::hardware::memory_map::MemoryMap;
use ::hardware::io::sms2::Sms2Io;
use ::hardware::z80::*;
use ::hardware::vdp::*;

pub struct EmulationManager<M: MemoryMap>
{
    z80: Z80<Sms2Io<M>>,
}

const SYSTEM_FREQUENCY: u64 = 10738580;
const AUDIO_BUFFER_SIZE: usize = 0x800;

impl<M: MemoryMap> EmulationManager<M> {
    pub fn new(mm: M, host_io: HostIo) -> EmulationManager<M> {
        let io = Sms2Io::new(mm, host_io);
        EmulationManager {
            z80: Z80::new(io),
        }
    }

    // XXX return something other than a ScreenError
    pub fn main_loop<S>(
        &mut self,
        screen: &mut S,
        palette_screen: &mut S,
        sprite_screen: &mut S,
        tile_screen: &mut S,
        audio: sdl2::AudioSubsystem,
        n: u64
    ) -> Result<(), ScreenError>
    where
        S: Screen
    {
        use sdl_wrap;

        let result = audio.open_queue(
            None,
            &sdl2::audio::AudioSpecDesired {
                freq: Some((SYSTEM_FREQUENCY / 48) as i32),
                channels: Some(1),
                samples: Some(AUDIO_BUFFER_SIZE as u16),
            }
        );
        let audio_queue = match result {
            Err(s) => return Err(ScreenError(s)),
            Ok(a) => a,
        };
        audio_queue.resume();

        let mut audio_buffer = [0i16; AUDIO_BUFFER_SIZE];

        let system_time = std::time::SystemTime::now();

        for i in 0 .. n {
            log_major!("EM: loop {}", i);

            self.z80.io.vdp.draw_line(screen)?;

            self.z80.io.vdp.draw_palettes(palette_screen)?;

            self.z80.io.vdp.draw_sprites(sprite_screen)?;

            self.z80.io.vdp.draw_tiles(tile_screen)?;

            let vdp_cycles = self.z80.io.vdp.cycles;
            let z80_target_cycles = 2 * vdp_cycles / 3;
            Z80Interpreter {}.run(&mut self.z80, z80_target_cycles);

            let sound_target_cycles = z80_target_cycles / 16;

            if sdl_wrap::event::check_quit() {
                break;
            }

            if self.z80.io.vdp.read_v() == 0 {
                self.z80.io.sn76489.queue(
                    sound_target_cycles,
                    &mut audio_buffer,
                    |buf| {
                        audio_queue.queue(buf);
                    }
                );

                let total_duration = match system_time.elapsed() {
                    Err(e) => return Err(ScreenError(e.description().to_string())),
                    Ok(d) => d,
                };
                let desired_time_seconds = (3 * self.z80.cycles) / SYSTEM_FREQUENCY;
                let cycles_given_seconds = (desired_time_seconds * SYSTEM_FREQUENCY) / 3;
                let remaining_cycles = self.z80.cycles - cycles_given_seconds;
                let desired_time_nanos = (3000000000 * remaining_cycles) / SYSTEM_FREQUENCY;
                debug_assert!(desired_time_nanos < 1000000000);
                let desired_duration = std::time::Duration::new(
                    desired_time_seconds,
                    desired_time_nanos as u32
                );
                match desired_duration.checked_sub(total_duration) {
                    None => {}
                    Some(diff) => {
                        // println!("worked");
                        std::thread::sleep(diff);
                    }
                }
                // println!("old time {:?}, new time {:?}", total_duration, desired_duration);
                // std::process::exit(1);
                // std::thread::sleep(diff);
            }
        }
        Ok(())
    }
}
