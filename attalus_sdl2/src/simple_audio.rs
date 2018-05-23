use std;

use failure::Error;

use sdl2::audio::AudioQueue;
use sdl2::{self, AudioSubsystem};

use attalus::host_multimedia::SimpleAudio;

pub const DEFAULT_BUFFER_SIZE: u16 = 0x800;

pub const DEFAULT_FREQUENCY: u32 = 1000000;

pub struct Audio {
    buffer: Box<[i16]>,
    queue: AudioQueue<i16>,
    audio_subsystem: AudioSubsystem,
}

impl Audio {
    pub fn new(sdl: &sdl2::Sdl) -> std::result::Result<Audio, Error> {
        let audio_subsystem = sdl
            .audio()
            .map_err(|s| format_err!("Unable to create SDL audio subsystem: {}", s))?;

        let queue = audio_subsystem
            .open_queue(
                None,
                &sdl2::audio::AudioSpecDesired {
                    freq: Some(DEFAULT_FREQUENCY as i32),
                    channels: Some(1),
                    samples: Some(DEFAULT_BUFFER_SIZE as u16),
                },
            )
            .map_err(|s| format_err!("Unable to create SDL audio subsystem: {}", s))?;

        Ok(Audio {
            buffer: vec![0i16; DEFAULT_BUFFER_SIZE as usize].into_boxed_slice(),
            queue,
            audio_subsystem,
        })
    }
}

impl SimpleAudio for Audio {
    fn configure(&mut self, frequency: u32, buffer_size: u16) -> std::result::Result<(), Error> {
        self.queue = self
            .audio_subsystem
            .open_queue(
                None,
                &sdl2::audio::AudioSpecDesired {
                    freq: Some(frequency as i32),
                    channels: Some(1),
                    samples: Some(buffer_size as u16),
                },
            )
            .map_err(|s| format_err!("SDL audio error {}", s))?;

        self.buffer = vec![0i16; buffer_size as usize].into_boxed_slice();

        Ok(())
    }

    fn play(&mut self) -> std::result::Result<(), Error> {
        self.queue.resume();
        Ok(())
    }

    fn pause(&mut self) -> std::result::Result<(), Error> {
        self.queue.pause();
        Ok(())
    }

    #[inline]
    fn buffer_set(&mut self, i: usize, value: i16) {
        self.buffer[i] = value
    }

    #[inline]
    fn buffer_len(&self) -> usize {
        self.buffer.len()
    }

    fn queue_buffer(&mut self) -> std::result::Result<(), Error> {
        if self.queue.queue(&self.buffer) {
            Ok(())
        } else {
            // I don't actually know whether these errors are fatal, so let's
            // just say they are to be safe.
            Err(format_err!("SDL Audio error {}", sdl2::get_error()))
        }
    }

    fn clear(&mut self) -> std::result::Result<(), Error> {
        self.queue.clear();
        Ok(())
    }
}
