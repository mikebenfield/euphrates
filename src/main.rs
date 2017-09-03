// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

extern crate sdl2;
extern crate attalus;

use attalus::sdl_wrap::event::HostIo;
use attalus::hardware::memory_map::*;
use attalus::emulation_manager::*;
use attalus::sdl_wrap::video::Window;

fn start_loop<M: MemoryMap>(mm: M, n: u64) {
    let sdl = sdl2::init().unwrap();
    let audio = sdl.audio().unwrap();
    let host_io = HostIo::new(&sdl).unwrap();

    let mut em = EmulationManager::new(mm, host_io);

    let mut palette_win = Window::new(&sdl).unwrap();
    palette_win.set_size(96, 3);
    palette_win.set_texture_size(32, 1);
    palette_win.set_title("palettes");

    let mut sprite_win = Window::new(&sdl).unwrap();
    sprite_win.set_size(256, 256);
    sprite_win.set_texture_size(256, 256);
    sprite_win.set_title("sprites");

    let mut tile_win = Window::new(&sdl).unwrap();
    tile_win.set_size(256, 224);
    sprite_win.set_texture_size(256, 224);
    tile_win.set_title("tiles");

    let mut win = Window::new(&sdl).unwrap();
    win.set_size(768, 576);
    win.set_texture_size(256, 192);
    win.set_title("Attalus");

    match em.main_loop(&mut win, &mut palette_win, &mut sprite_win, &mut tile_win, audio, n) {
        Ok(()) => println!("Exit OK"),
        _ => println!("Exit error"),
    }
}

fn main() {
    let mut args: Vec<String> = Vec::new();
    args.extend(std::env::args());
    if args.len() < 4 {
        eprintln!("Usage: exec [sega|codemasters] filename n");
        return;
    }
    let filename = &args[2];
    let n: u64 = args[3].parse().expect("Usage: exec [sega|codemasters] filename n");
    match args[1].as_ref() {
        "sega" => {
            start_loop(SegaMemoryMap::new_from_file(filename.as_ref()).unwrap(), n);
        }
        "codemasters" => {
            start_loop(CodemastersMemoryMap::new_from_file(filename.as_ref()).unwrap(), n);
        }
        _ => {
            eprintln!("Usage: exec [sega|codemasters] filename n");
            return;
        }
    }
}
