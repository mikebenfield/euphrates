
use super::*;

pub fn check_quit() -> bool {
    unsafe {
        sdls::event::SDL_PumpEvents();
        let has_event = 0 != sdls::event::SDL_HasEvent(sdls::event::SDL_QUIT);
        sdls::event::SDL_FlushEvent(sdls::event::SDL_QUIT);
        has_event
    }
}
