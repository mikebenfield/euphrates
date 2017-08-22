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
