// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

extern crate sdl2;
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate attalus;

use std::path::PathBuf;
use std::fs::File;
use std::io::{BufReader, BufRead};

use clap::{Arg, ArgMatches, App, SubCommand};

use attalus::hardware::z80;
use attalus::systems::sega_master_system::{self, HardwareBuilder, System, Decode};
use attalus::hardware::memory_16_8;
use attalus::hardware::vdp;
use attalus::sdl_wrap;
use attalus::memo::NothingInbox;

mod errors {
    use attalus;

    error_chain!{
        links {
            AttalusError(attalus::errors::Error, attalus::errors::ErrorKind);
        }

    }
}

use ::errors::*;

fn run_rom(matches: &ArgMatches) -> Result<()> {
    let filename = matches.value_of("rom").unwrap();
    let memory_map = matches.value_of("memorymap").unwrap();
    let save_directory = match matches.value_of("savedirectory") {
        None => None,
        Some(s) => Some(PathBuf::from(s)),
    };

    let sdl = sdl2::init().unwrap();

    let mut emulator = sega_master_system::Emulator::new(
        sega_master_system::Frequency::Ntsc,
        <z80::Interpreter as Default>::default(),
        <vdp::SimpleEmulator as Default>::default(),
    );

    let mut win = sdl_wrap::simple_graphics::Window::new(&sdl)?;
    win.set_size(768, 576);
    win.set_texture_size(256, 192);
    win.set_title("Attalus");

    let mut audio = sdl_wrap::simple_audio::Audio::new(&sdl)?;

    let mut user_interface = sdl_wrap::master_system_user_interface::SdlMasterSystemUserInterface::new(&sdl, save_directory).unwrap();

    if memory_map == "sega" {
        let master_system_hardware = HardwareBuilder::new().build_from_file::<memory_16_8::sega::Component>(filename).unwrap();
        let mut master_system = System::new(NothingInbox, master_system_hardware);
        emulator.run(
            &mut master_system,
            &mut win,
            &mut audio,
            &mut user_interface,
        )?;
    } else if memory_map == "codemasters" {
        let master_system_hardware = HardwareBuilder::new().build_from_file::<memory_16_8::codemasters::Component>(filename).unwrap();
        let mut master_system = System::new(NothingInbox, master_system_hardware);
        emulator.run(
            &mut master_system,
            &mut win,
            &mut audio,
            &mut user_interface,
        )?;
    } else {
        bail!(format!("Can't happen: Unknown memory map {}", memory_map))
    }

    Ok(())
}

fn run_load(matches: &ArgMatches) -> Result<()> {
    let load_filename = matches.value_of("loadfile").unwrap();
    let save_directory = match matches.value_of("savedirectory") {
        None => None,
        Some(s) => Some(PathBuf::from(s)),
    };

    let sdl = sdl2::init().unwrap();

    let mut emulator = sega_master_system::Emulator::new(
        sega_master_system::Frequency::Ntsc,
        <z80::Interpreter as Default>::default(),
        <vdp::SimpleEmulator as Default>::default(),
    );

    let mut win = sdl_wrap::simple_graphics::Window::new(&sdl)?;
    win.set_size(768, 576);
    win.set_texture_size(256, 192);
    win.set_title("Attalus");

    let mut audio = sdl_wrap::simple_audio::Audio::new(&sdl)?;

    let mut user_interface = sdl_wrap::master_system_user_interface::SdlMasterSystemUserInterface::new(&sdl, save_directory).unwrap();

    let mut load_file = BufReader::with_capacity(1024, File::open(load_filename).unwrap());

    let mut nothing = String::new();
    load_file.read_line(&mut nothing).unwrap();

    let mut master_system = <System<NothingInbox, memory_16_8::sega::Component> as Decode>::decode(&mut load_file).unwrap();
    emulator.run(
        &mut master_system,
        &mut win,
        &mut audio,
        &mut user_interface,
    )?;

    Ok(())
}

fn run() -> Result<()> {
    let memory_map_arg =
        Arg::with_name("memorymap")
            .long("memorymap")
            .value_name("(sega|codemasters)")
            .help("Specify the sega or codemasters memory map")
            .takes_value(true)
            .required(true)
            .default_value("sega");
    let save_directory_arg =
        Arg::with_name("savedirectory")
            .long("savedirectory")
            .value_name("DIRECTORY")
            .help("Specify the directory in which to save states")
            .takes_value(true);

    let app =
        App::new("Attalus")
            .version("0.1.0")
            .author("Michael Benfield")
            .about("Sega Master System emulator")
            .subcommand(SubCommand::with_name("rom")
                .about("Play a game from a ROM image")
                .arg(Arg::with_name("rom")
                    .long("rom")
                    .value_name("FILE")
                    .help("Specify the filename containing a ROM image")
                    .takes_value(true)
                    .required(true)
                )
                .arg(memory_map_arg.clone())
                .arg(save_directory_arg.clone())
            )
            .subcommand(SubCommand::with_name("load")
                .about("Load a saved state")
                .arg(save_directory_arg.clone())
                .arg(Arg::with_name("loadfile")
                    .long("loadfile")
                    .value_name("FILE")
                    .help("Specify the saved state file")
                    .takes_value(true)
                    .required(true)
                )
            );
    let matches = app.get_matches();
    
    return match matches.subcommand() {
        ("rom", Some(sub)) => run_rom(&sub),
        ("load", Some(sub)) => run_load(&sub),
        (x, _) => {
            eprintln!("Unknown subcommand {}", x);
            eprintln!("{}", matches.usage());
            bail!("No subcommand");
        }
    };
}

fn main() {
    if let Err(x) = run() {
        eprintln!("{:?}", x);
    }

    // match run() {
    //     Err(x) => eprintln!("{:?}")
    // }
    // let mut args: Vec<String> = Vec::new();
    // args.extend(std::env::args());
    // if args.len() != 3 {
    //     eprintln!("Usage: exec [sega|codemasters] filename");
    //     return;
    // }

    // let filename = &args[2];

    // let sdl = sdl2::init().unwrap();

    // let mut emulator = sega_master_system::Emulator::new(
    //     sega_master_system::Frequency::Ntsc,
    //     <z80::Interpreter as Default>::default(),
    //     <vdp::SimpleEmulator as Default>::default(),
    // );
    // let master_system_hardware = HardwareBuilder::new().build_from_file::<memory_16_8::sega::Component>(filename).unwrap();
    // let mut master_system = System::new(NothingInbox, master_system_hardware);

    // let mut win = sdl_wrap::simple_graphics::Window::new(&sdl).unwrap();
    // win.set_size(768, 576);
    // win.set_texture_size(256, 192);
    // win.set_title("Attalus");
    // let mut user_interface = sdl_wrap::master_system_user_interface::SdlMasterSystemUserInterface::new(&sdl).unwrap();

    // let mut audio = sdl_wrap::simple_audio::Audio::new(&sdl).unwrap();

    // emulator.run(
    //     &mut master_system,
    //     &mut win,
    //     &mut audio,
    //     &mut user_interface,
    // ).unwrap();
}
