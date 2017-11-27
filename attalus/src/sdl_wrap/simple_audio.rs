// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use std;

use sdl2::{self, AudioSubsystem};
use sdl2::audio::AudioQueue;

use ::errors::{CommonKind, SimpleKind, Error};
use ::host_multimedia::{SimpleAudio, Result};

pub const DEFAULT_BUFFER_SIZE: u16 = 0x800;

pub const DEFAULT_FREQUENCY: u32 = 1000000;

pub struct Audio {
    buffer: Box<[i16]>,
    queue: AudioQueue<i16>,
    audio_subsystem: AudioSubsystem,
}

impl Audio {
    pub fn new(sdl: &sdl2::Sdl) -> std::result::Result<Audio, Error<SimpleKind>> {
        let audio_subsystem = sdl.audio().map_err(|s|
            SimpleKind(format!("Unable to create SDL audio subsystem: {}", s))
        )?;

        let queue = audio_subsystem.open_queue(
            None,
            &sdl2::audio::AudioSpecDesired {
                freq: Some(DEFAULT_FREQUENCY as i32),
                channels: Some(1),
                samples: Some(DEFAULT_BUFFER_SIZE as u16),
            },
        ).map_err(|s|
            SimpleKind(format!("Unable to create SDL audio queue: {}", s))
        )?;

        Ok(
            Audio {
                buffer: vec![0i16; DEFAULT_BUFFER_SIZE as usize].into_boxed_slice(),
                queue,
                audio_subsystem,
            }
        )
    }
}

impl SimpleAudio for Audio {
    fn configure(&mut self, frequency: u32, buffer_size: u16) -> Result<()> {
        self.queue = self.audio_subsystem.open_queue(
            None,
            &sdl2::audio::AudioSpecDesired {
                freq: Some(frequency as i32),
                channels: Some(1),
                samples: Some(buffer_size as u16),
            }
        ).map_err(|s| CommonKind::Dead(format!("SDL audio error {}", s)))?;

        self.buffer = vec![0i16; buffer_size as usize].into_boxed_slice();

        Ok(())
    }

    fn play(&mut self) -> Result<()> {
        self.queue.resume();
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        self.queue.pause();
        Ok(())
    }

    fn buffer(&mut self) -> &mut [i16] {
        &mut self.buffer
    }

    fn queue_buffer(&mut self) -> Result<()> {
        if self.queue.queue(& self.buffer) {
            Ok(())
        } else {
            // I don't actually know whether these errors are fatal, so let's
            // just say they are to be safe.
            Err(
                CommonKind::Dead(
                    format!("SDL Audio error {}", sdl2::get_error())
                )
            )?
        }
    }

    fn clear(&mut self) -> Result<()> {
        self.queue.clear();
        Ok(())
    }
}
