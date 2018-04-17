use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};
use std;

use failure::Error;
use sdl2;

use hardware::z80;
use hardware::memory_16_8::sega::SegaMemoryMap;
use host_multimedia::SimpleAudio;
use systems::sega_master_system::{Frequency, Hardware, MasterSystem, PlaybackStatus, PlayerStatus,
                                  RecordingStatus, System, TimeStatus};
use utilities::FrameInfo;
use save;

pub type Result<T> = std::result::Result<T, Error>;

bitflags! {
    struct JoypadPortA: u8 {
        const JOYPAD2_DOWN = 0b10000000;
        const JOYPAD2_UP = 0b01000000;
        const JOYPAD1_B = 0b00100000;
        const JOYPAD1_A = 0b00010000;
        const JOYPAD1_RIGHT = 0b00001000;
        const JOYPAD1_LEFT = 0b00000100;
        const JOYPAD1_DOWN = 0b00000010;
        const JOYPAD1_UP = 0b00000001;
    }
}

bitflags! {
    struct JoypadPortB: u8 {
        const B_TH = 0b10000000;
        const A_TH = 0b01000000;
        const CONT = 0b00100000;
        const RESET = 0b00010000;
        const JOYPAD2_B = 0b00001000;
        const JOYPAD2_A = 0b00000100;
        const JOYPAD2_RIGHT = 0b00000010;
        const JOYPAD2_LEFT = 0b00000001;
    }
}

pub struct PlaybackInterface {
    playback_status: PlaybackStatus,
}

impl PlaybackInterface {
    pub fn new(player_statuses: &[PlayerStatus]) -> Self {
        PlaybackInterface {
            playback_status: PlaybackStatus::from_recorded(player_statuses),
        }
    }

    pub fn run<I>(
        &mut self,
        master_system: &mut System<I, SegaMemoryMap>,
        frequency: Frequency,
    ) -> Result<Duration>
    where
        System<I, SegaMemoryMap>: MasterSystem,
    {
        master_system.init(frequency)?;
        let mut frame_info = FrameInfo::default();

        master_system.play()?;

        let time_status = TimeStatus::new(z80::internal::T::cycles(master_system));

        let start = Instant::now();

        while let Some(player_status) = self.playback_status.pop() {
            master_system.run_frame(&player_status, &time_status, &mut frame_info)?;
        }

        let end = Instant::now();
        Ok(end.duration_since(start))
    }
}

pub struct UserInterface {
    save_directory: Option<PathBuf>,
    player_status: PlayerStatus,
    event_pump: sdl2::EventPump,
    recording_status: RecordingStatus<Hardware<SegaMemoryMap>>,
    playback_status: PlaybackStatus,
}

impl UserInterface {
    pub fn new(
        sdl: &sdl2::Sdl,
        save_directory: Option<PathBuf>,
        player_statuses: &[PlayerStatus],
    ) -> Result<Self> {
        sdl.event()
            .map_err(|s| format_err!("Error initializing the SDL event subsystem {}", s))?;

        let event_pump = sdl.event_pump()
            .map_err(|s| format_err!("Error obtaining the SDL event pump {}", s))?;

        Ok(UserInterface {
            save_directory: save_directory,
            player_status: Default::default(),
            event_pump: event_pump,
            recording_status: Default::default(),
            playback_status: PlaybackStatus::from_recorded(player_statuses),
        })
    }

    fn frame_update<I>(&mut self, master_system: &System<I, SegaMemoryMap>) -> bool
    where
        System<I, SegaMemoryMap>: MasterSystem,
    {
        self.player_status.pause = false;

        for event in self.event_pump.poll_iter() {
            use sdl2::keyboard::Scancode::*;

            match event {
                sdl2::event::Event::Quit { .. } => return false,
                sdl2::event::Event::KeyDown {
                    scancode: Some(k),
                    keymod,
                    ..
                } => {
                    match (
                        k,
                        keymod.contains(sdl2::keyboard::LSHIFTMOD)
                            || keymod.contains(sdl2::keyboard::RSHIFTMOD),
                    ) {
                        (P, _) => self.player_status.pause = true,
                        (R, false) => self.recording_status
                            .begin_recording(&master_system.hardware),
                        (R, true) => {
                            if let (&Some(ref path), Some(recording)) =
                                (&self.save_directory, self.recording_status.recording())
                            {
                                let cycles = z80::internal::T::cycles(master_system);
                                let mut path2 = path.clone();
                                let recording2 = recording.clone();

                                thread::spawn(move || {
                                    path2.push(format!("{:>0width$X}.record", cycles, width = 20));

                                    if let Err(e) = save::serialize_at(&path2, &recording2) {
                                        eprintln!("Error saving recording: {:?}", e);
                                    }
                                });
                            }
                        }
                        (Z, _) => {
                            if let Some(ref path) = self.save_directory {
                                // save in a new thread to avoid UI delay
                                let cycles = z80::internal::T::cycles(master_system);
                                let hardware = master_system.hardware.clone();
                                let mut path2 = path.clone();
                                thread::spawn(move || {
                                    path2.push(format!("{:>0width$X}.state", cycles, width = 20));

                                    if let Err(e) = save::serialize_at(&path2, &hardware) {
                                        eprintln!("Error saving state: {:?}", e);
                                    }
                                });
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        let keyboard_state = self.event_pump.keyboard_state();

        let mut joypad_a = JoypadPortA::all();
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::W) {
            joypad_a.remove(JoypadPortA::JOYPAD1_UP);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::A) {
            joypad_a.remove(JoypadPortA::JOYPAD1_LEFT);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::S) {
            joypad_a.remove(JoypadPortA::JOYPAD1_DOWN);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::D) {
            joypad_a.remove(JoypadPortA::JOYPAD1_RIGHT);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::F) {
            joypad_a.remove(JoypadPortA::JOYPAD1_A);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::G) {
            joypad_a.remove(JoypadPortA::JOYPAD1_B);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::I) {
            joypad_a.remove(JoypadPortA::JOYPAD2_UP);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::K) {
            joypad_a.remove(JoypadPortA::JOYPAD2_DOWN);
        }
        self.player_status.joypad_a = joypad_a.bits;

        let mut joypad_b = JoypadPortB::all();
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::J) {
            joypad_b.remove(JoypadPortB::JOYPAD2_LEFT);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::L) {
            joypad_b.remove(JoypadPortB::JOYPAD2_RIGHT);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Semicolon) {
            joypad_b.remove(JoypadPortB::JOYPAD2_A);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Apostrophe) {
            joypad_b.remove(JoypadPortB::JOYPAD2_B);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Space) {
            joypad_b.remove(JoypadPortB::RESET);
        }
        self.player_status.joypad_b = joypad_b.bits;

        if self.player_status != Default::default() {
            self.playback_status.end_playback();
        } else if let Some(player_status) = self.playback_status.pop() {
            self.player_status = player_status;
        }

        self.recording_status.update(self.player_status);

        true
    }

    pub fn run<I>(
        &mut self,
        master_system: &mut System<I, SegaMemoryMap>,
        frequency: Frequency,
    ) -> Result<()>
    where
        System<I, SegaMemoryMap>: MasterSystem,
    {
        let mut frame_info = FrameInfo::default();

        master_system.init(frequency)?;
        master_system.play()?;

        let time_status = TimeStatus::new(z80::internal::T::cycles(master_system));

        loop {
            if !self.frame_update(master_system) {
                return Ok(());
            }

            master_system.run_frame(&self.player_status, &time_status, &mut frame_info)?;
        }
    }
}
