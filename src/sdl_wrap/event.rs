// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use super::*;

pub fn check_quit() -> bool {
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

pub fn joypada() -> u8 {
    let mut jp_input = JoypadPortA::all();
    unsafe {
        let key_state: *const u8 = sdls::SDL_GetKeyboardState(std::ptr::null_mut());
        if *key_state.offset(sdls::SDL_SCANCODE_W as isize) != 0 {
            jp_input.remove(JOYPAD1_UP);
        }
        if *key_state.offset(sdls::SDL_SCANCODE_A as isize) != 0 {
            jp_input.remove(JOYPAD1_LEFT);
        }
        if *key_state.offset(sdls::SDL_SCANCODE_S as isize) != 0 {
            jp_input.remove(JOYPAD1_DOWN);
        }
        if *key_state.offset(sdls::SDL_SCANCODE_D as isize) != 0 {
            jp_input.remove(JOYPAD1_RIGHT);
        }
        if *key_state.offset(sdls::SDL_SCANCODE_F as isize) != 0 {
            jp_input.remove(JOYPAD1_A);
        }
        if *key_state.offset(sdls::SDL_SCANCODE_G as isize) != 0 {
            jp_input.remove(JOYPAD1_B);
        }
        if *key_state.offset(sdls::SDL_SCANCODE_I as isize) != 0 {
            jp_input.remove(JOYPAD2_UP);
        }
        if *key_state.offset(sdls::SDL_SCANCODE_K as isize) != 0 {
            jp_input.remove(JOYPAD2_DOWN);
        }
    }
    jp_input.bits
}

pub fn joypadb() -> u8 {
    let mut jp_input = JoypadPortB::all();
    unsafe {
        let key_state: *const u8 = sdls::SDL_GetKeyboardState(std::ptr::null_mut());
        if *key_state.offset(sdls::SDL_SCANCODE_J as isize) != 0 {
            jp_input.remove(JOYPAD2_LEFT);
        }
        if *key_state.offset(sdls::SDL_SCANCODE_L as isize) != 0 {
            jp_input.remove(JOYPAD2_RIGHT);
        }
        if *key_state.offset(sdls::SDL_SCANCODE_SEMICOLON as isize) != 0 {
            jp_input.remove(JOYPAD2_A);
        }
        if *key_state.offset(sdls::SDL_SCANCODE_APOSTROPHE as isize) != 0 {
            jp_input.remove(JOYPAD2_B);
        }
        if *key_state.offset(sdls::SDL_SCANCODE_SPACE as isize) != 0 {
            jp_input.remove(RESET);
        }
    }
    jp_input.bits
}