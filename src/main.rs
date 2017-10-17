// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

extern crate serde;
extern crate sdl2;
extern crate rlua;
#[macro_use]
extern crate quick_error;
extern crate attalus;

use std::io::Read;

use serde::ser::Serialize;

use attalus::hardware::memory_map::*;
use attalus::emulation_manager::*;
use attalus::sdl_wrap::video::Window;
use attalus::message::{Receiver, Sender};
use attalus::lua::serde as lua_serde;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Lua(err: rlua::Error) {
            from()
        }

        LuaSerde(err: lua_serde::Error) {
            from()
        }

        Io(err: std::io::Error) {
            from()
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

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

fn do_lua(filename: &str) -> Result<()> {
    use rlua::{Lua};
    let lua = Lua::new();
    let val = Some(123usize);
    let v = val.serialize(lua_serde::Serializer::new(&lua))?;
    lua.globals().raw_set("this_var_name", v)?;
    lua.globals().raw_set("this_other_var",
        lua_serde::ToLuaN(&attalus::hardware::z80::Z80Message::MaskableInterruptDenied)
    )?;

    let mut file = std::fs::File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let f = lua.load(contents.as_ref(), Some(filename))?;
    let () = f.call(())?;
    let z: lua_serde::FromLuaN<attalus::hardware::z80::Z80Message> =
        lua.globals().raw_get("output")?;
    println!("{:?}", z.0);
    Ok(())
}

fn main() {
    let mut args: Vec<String> = Vec::new();
    args.extend(std::env::args());
    if args.len() == 3 && args[1] == "lua" {
        return do_lua(args[2].as_ref()).unwrap();
    }
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
