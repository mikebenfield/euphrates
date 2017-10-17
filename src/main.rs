// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

extern crate sdl2;
extern crate rlua;
extern crate attalus;

use attalus::hardware::memory_map::*;
use attalus::emulation_manager::*;
use attalus::sdl_wrap::video::Window;
use attalus::message::{Receiver, Sender};

fn start_loop<M: MemoryMap>(mm: M)
where
    M: MemoryMap,
    <M as Sender>::Message: std::fmt::Debug,
    DisassemblingReceiver: Receiver<<M as Sender>::Message>,
{
    let sdl = sdl2::init().unwrap();
    let audio = sdl.audio().unwrap();

    let event_pump = sdl.event_pump().unwrap();

    let mut em = EmulationManager::new(mm);

    let mut win = Window::new(&sdl).unwrap();
    win.set_size(768, 576);
    win.set_texture_size(256, 192);
    win.set_title("Attalus");

    match em.main_loop(&mut win, audio, event_pump) {
        Ok(()) => println!("Exit OK"),
        _ => println!("Exit error"),
    }
}

fn main() {
    let mut args: Vec<String> = Vec::new();
    args.extend(std::env::args());
    if args.len() != 3 {
        eprintln!("Usage: exec [sega|codemasters] filename");
        return;
    }
    let filename = &args[2];
    match args[1].as_ref() {
        "sega" => {
            start_loop(SegaMemoryMap::new_from_file(filename.as_ref()).unwrap());
        }
        "codemasters" => {
            start_loop(CodemastersMemoryMap::new_from_file(filename.as_ref()).unwrap());
        }
        _ => {
            eprintln!("Usage: exec [sega|codemasters] filename");
            return;
        }
    }
}
