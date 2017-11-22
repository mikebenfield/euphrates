// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

extern crate sdl2;
extern crate attalus;

use attalus::hardware::z80;
use attalus::systems::sega_master_system::{self, HardwareBuilder, System};
use attalus::hardware::memory_16_8;
use attalus::hardware::vdp;
use attalus::sdl_wrap;
use attalus::memo::NothingInbox;

fn main() {
    let mut args: Vec<String> = Vec::new();
    args.extend(std::env::args());
    if args.len() != 3 {
        eprintln!("Usage: exec [sega|codemasters] filename");
        return;
    }
    let filename = &args[2];

    let sdl = sdl2::init().unwrap();

    let mut emulator = sega_master_system::Emulator::new(
        sega_master_system::Frequency::Ntsc,
        <z80::Interpreter as Default>::default(),
        <vdp::SimpleEmulator as Default>::default(),
    );
    let master_system_hardware = HardwareBuilder::new().build_from_file::<memory_16_8::sega::Component>(filename).unwrap();
    let mut master_system = System::new(NothingInbox, master_system_hardware);

    let mut win = sdl_wrap::simple_graphics::Window::new(&sdl).unwrap();
    win.set_size(768, 576);
    win.set_texture_size(256, 192);
    win.set_title("Attalus");
    let mut user_interface = sdl_wrap::master_system_user_interface::SdlMasterSystemUserInterface::new(&sdl).unwrap();

    let mut audio = sdl_wrap::simple_audio::Audio::new(&sdl).unwrap();

    emulator.run(
        &mut master_system,
        &mut win,
        &mut audio,
        &mut user_interface,
    ).unwrap();
}
