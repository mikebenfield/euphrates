// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use sdl2;

use super::*;

pub fn check_quit() -> bool {
    use sdl2::sys as sdls;
    unsafe {
        sdls::event::SDL_PumpEvents();
        let has_event = 0 != sdls::event::SDL_HasEvent(sdls::event::SDL_QUIT);
        sdls::event::SDL_FlushEvent(sdls::event::SDL_QUIT);
        has_event
    }
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
        const RESET = 0b00010000;
        const JOYPAD2_B = 0b00001000;
        const JOYPAD2_A = 0b00000100;
        const JOYPAD2_RIGHT = 0b00000010;
        const JOYPAD2_LEFT = 0b00000001;
    }
}

pub struct InputStatus {
    pub joypad_a: u8,
    pub joypad_b: u8,
}

pub fn input_status(event_pump: &sdl2::EventPump) -> InputStatus {
        let keyboard_state = event_pump.keyboard_state();

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

        InputStatus {
            joypad_a: joypad_a.bits,
            joypad_b: joypad_b.bits,
        }
}
