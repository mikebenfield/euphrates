// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use std;
use std::path::{Path, PathBuf};
use std::io::Write;

use failure::ResultExt;
use sdl2;

use ::errors::{Error, CommonKind};
use ::hardware::z80;
use ::systems::sega_master_system::{MasterSystem, UserInterface, PlayerStatus, Query, Command};

pub type Result<T> = std::result::Result<T, Error<CommonKind>>;

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
        const RESET = 0b00010000;
        const JOYPAD2_B = 0b00001000;
        const JOYPAD2_A = 0b00000100;
        const JOYPAD2_RIGHT = 0b00000010;
        const JOYPAD2_LEFT = 0b00000001;
    }
}

pub struct SdlMasterSystemUserInterface {
    save_directory: Option<PathBuf>,
    joypad_a: u8,
    joypad_b: u8,
    pause: bool,
    quit: bool,
    event_pump: sdl2::EventPump,
}

impl SdlMasterSystemUserInterface {
    pub fn new(sdl: &sdl2::Sdl, save_directory: Option<PathBuf>) -> Result<Self> {
        sdl.event().map_err(|s|
            CommonKind::Dead(format!("Error initializing the SDL event subsystem {}", s))
        )?;
        let event_pump = sdl.event_pump().map_err(|s|
            CommonKind::Dead(format!("Error obtaining the SDL event pump {}", s))
        )?;

        Ok(
            SdlMasterSystemUserInterface {
                save_directory: save_directory,
                joypad_a: 0xFF,
                joypad_b: 0xFF,
                pause: false,
                quit: false,
                event_pump: event_pump,
            },
        )
    }
}

fn save_master_system<P: AsRef<Path>>(path: P, master_system: &MasterSystem) -> Result<()> {
    use std::fs::File;

    let mut file = File::create(path.as_ref()).with_context(|e|
        CommonKind::Live(
            format!("SDL user interface: could not open file {:?} to save state: {}", path.as_ref(), e)
        )
    )?;

    file.write(master_system.tag().as_bytes()).unwrap();
    file.write(b"\n").unwrap();
    master_system.encode(&mut file).with_context(|e|
        CommonKind::Live(
            format!("SDL user interface: could not encode system image: {}", e)
        )
    )?;

    Ok(())
}

impl UserInterface for SdlMasterSystemUserInterface {
    fn update(&mut self, master_system: &mut MasterSystem) -> Result<()> {
        self.quit = false;
        self.pause = false;

        for event in self.event_pump.poll_iter() {
            use sdl2::keyboard::Scancode::*;

            match event {
                sdl2::event::Event::Quit { .. } => self.quit = true,
                sdl2::event::Event::KeyDown {
                    scancode: Some(k),
                    ..
                } => match k {
                    P => self.pause = true,
                    Z => {
                        if let Some(ref path) = self.save_directory {
                            let z80: &z80::Component = master_system.get();
                            let mut path2 = path.clone();
                            path2.push(format!("{:>0width$X}", z80.cycles, width=20));
                            if let Err(e) = save_master_system(path2, master_system) {
                                eprintln!("Error saving file: {:?}", e);
                            }
                        }
                    }
                    _ => {}
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
        self.joypad_a = joypad_a.bits;

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
        self.joypad_b = joypad_b.bits;

        Ok(())
    }

    fn player_status(&self) -> PlayerStatus {
        PlayerStatus {
            joypad_a: self.joypad_a,
            joypad_b: self.joypad_b,
            pause: self.pause,
        }
    }

    fn respond(&mut self, _s: String) {
    }

    fn command(&mut self) -> Option<Command> {
        None
    }

    fn query(&mut self) -> Option<Query> {
        None
    }

    fn wants_quit(&self) -> bool {
        self.quit
    }
}

