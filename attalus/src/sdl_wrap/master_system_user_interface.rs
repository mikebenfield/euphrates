// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use std::convert::AsRef;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std;

use failure::Error;
use sdl2;

use hardware::vdp;
use hardware::z80;
use host_multimedia::SimpleAudio;
use sdl_wrap;
use systems::sega_master_system::{Emulator, MasterSystem, PlaybackStatus, PlayerStatus,
                                  RecordingStatus, TimeStatus};
use utilities::{FrameInfo, Tag};

pub type Result<T> = std::result::Result<T, Error>;

fn save_tag<P: AsRef<Path>, T: Tag>(path: P, tag: &T) -> Result<()> {
    use std::fs::File;

    let mut file = File::create(path.as_ref())?;

    tag.write(&mut file)?;

    Ok(())
}

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
            playback_status: PlaybackStatus::from_recorded(player_statuses)
        }
    }

    pub fn run<Z80Emulator, VdpEmulator, S>(
        &mut self,
        sdl: &sdl2::Sdl,
        emulator: &mut Emulator<Z80Emulator, VdpEmulator>,
        master_system: &mut S,
    ) -> Result<Duration>
    where
        S: MasterSystem,
        Z80Emulator: z80::Emulator<S>,
        VdpEmulator: vdp::Emulator<sdl_wrap::simple_graphics::Window>,
    {
        let mut win = sdl_wrap::simple_graphics::Window::new(&sdl)?;
        win.set_size(768, 576);
        win.set_texture_size(256, 192);
        win.set_title("Attalus");

        let mut frame_info = FrameInfo::default();
        let start = Instant::now();

        // XXX - replace with a fake audio object?
        let mut audio = sdl_wrap::simple_audio::Audio::new(&sdl)?;
        emulator.configure_audio(&mut audio)?;
        audio.play()?;

        let time_status =
            TimeStatus::new(<S as AsRef<z80::Component>>::as_ref(master_system).cycles);

        while let Some(player_status) = self.playback_status.pop() {
            emulator.run_frame(
                master_system,
                &mut win,
                &mut audio,
                &player_status,
                &time_status,
                &mut frame_info,
            )?;
        }

        let end = Instant::now();
        Ok(end.duration_since(start))
    }
}

pub struct UserInterface<S> {
    save_directory: Option<PathBuf>,
    player_status: PlayerStatus,
    event_pump: sdl2::EventPump,
    recording_status: RecordingStatus<S>,
    playback_status: PlaybackStatus,
}

impl<S> UserInterface<S> {
    pub fn new(
        sdl: &sdl2::Sdl,
        save_directory: Option<PathBuf>,
        player_statuses: &[PlayerStatus],
    ) -> Result<Self> {
        sdl.event().map_err(|s| {
            format_err!("Error initializing the SDL event subsystem {}", s)
        })?;

        let event_pump = sdl.event_pump().map_err(|s| {
            format_err!("Error obtaining the SDL event pump {}", s)
        })?;

        Ok(UserInterface {
            save_directory: save_directory,
            player_status: Default::default(),
            event_pump: event_pump,
            recording_status: Default::default(),
            playback_status: PlaybackStatus::from_recorded(player_statuses),
        })
    }

    fn frame_update(&mut self, master_system: &S) -> bool
    where
        S: MasterSystem,
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
                        keymod.contains(sdl2::keyboard::LSHIFTMOD) ||
                            keymod.contains(sdl2::keyboard::RSHIFTMOD),
                    ) {
                        (P, _) => self.player_status.pause = true,
                        (R, false) => self.recording_status.begin_recording(master_system),
                        (R, true) => {
                            if let (&Some(ref path), Some(recording)) =
                                (&self.save_directory, self.recording_status.recording())
                            {
                                let z80: &z80::Component = master_system.as_ref();
                                let mut path2 = path.clone();
                                path2.push(format!("{:>0width$X}.record", z80.cycles, width = 20));
                                if let Err(e) = save_tag(path2, recording) {
                                    eprintln!("Error saving file: {:?}", e);
                                }
                            }
                        }
                        (Z, _) => {
                            if let Some(ref path) = self.save_directory {
                                let z80: &z80::Component = master_system.as_ref();
                                let mut path2 = path.clone();
                                path2.push(format!("{:>0width$X}.state", z80.cycles, width = 20));
                                if let Err(e) = save_tag(path2, master_system) {
                                    eprintln!("Error saving file: {:?}", e);
                                }
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
            joypad_a.remove(JOYPAD1_UP);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::A) {
            joypad_a.remove(JOYPAD1_LEFT);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::S) {
            joypad_a.remove(JOYPAD1_DOWN);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::D) {
            joypad_a.remove(JOYPAD1_RIGHT);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::F) {
            joypad_a.remove(JOYPAD1_A);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::G) {
            joypad_a.remove(JOYPAD1_B);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::I) {
            joypad_a.remove(JOYPAD2_UP);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::K) {
            joypad_a.remove(JOYPAD2_DOWN);
        }
        self.player_status.joypad_a = joypad_a.bits;

        let mut joypad_b = JoypadPortB::all();
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::J) {
            joypad_b.remove(JOYPAD2_LEFT);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::L) {
            joypad_b.remove(JOYPAD2_RIGHT);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Semicolon) {
            joypad_b.remove(JOYPAD2_A);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Apostrophe) {
            joypad_b.remove(JOYPAD2_B);
        }
        if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Space) {
            joypad_b.remove(RESET);
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

    pub fn run<Z80Emulator, VdpEmulator>(
        &mut self,
        sdl: &sdl2::Sdl,
        emulator: &mut Emulator<Z80Emulator, VdpEmulator>,
        master_system: &mut S,
    ) -> Result<()>
    where
        S: MasterSystem,
        Z80Emulator: z80::Emulator<S>,
        VdpEmulator: vdp::Emulator<sdl_wrap::simple_graphics::Window>,
    {
        let mut win = sdl_wrap::simple_graphics::Window::new(&sdl)?;
        win.set_size(768, 576);
        win.set_texture_size(256, 192);
        win.set_title("Attalus");

        let mut frame_info = FrameInfo::default();

        let mut audio = sdl_wrap::simple_audio::Audio::new(&sdl)?;
        emulator.configure_audio(&mut audio)?;
        audio.play()?;

        let time_status =
            TimeStatus::new(<S as AsRef<z80::Component>>::as_ref(master_system).cycles);

        loop {
            if !self.frame_update(master_system) {
                return Ok(());
            }

            emulator.run_frame(
                master_system,
                &mut win,
                &mut audio,
                &self.player_status,
                &time_status,
                &mut frame_info,
            )?;
        }
    }
}
