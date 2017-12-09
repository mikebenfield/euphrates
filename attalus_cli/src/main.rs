// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

extern crate sdl2;
extern crate clap;
#[macro_use]
extern crate failure;
extern crate attalus;

use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use clap::{App, Arg, ArgMatches, SubCommand};
use failure::Error;

use attalus::Tag;
use attalus::hardware::memory_16_8;
use attalus::hardware::vdp;
use attalus::hardware::z80;
use attalus::memo::NothingInbox;
use attalus::sdl_wrap;
use attalus::systems::sega_master_system::{self, HardwareBuilder, Recording, System};

type Result<T> = std::result::Result<T, Error>;

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

    if memory_map == "sega" {
        let mut user_interface =
            sdl_wrap::master_system_user_interface::UserInterface::new(&sdl, save_directory, &[])?;
        let master_system_hardware = HardwareBuilder::new()
            .build_from_file::<memory_16_8::sega::Component>(filename)
            .unwrap();
        let mut master_system = System::new(NothingInbox, master_system_hardware);
        user_interface.run(&sdl, &mut emulator, &mut master_system)?;
    } else if memory_map == "codemasters" {
        let mut user_interface =
            sdl_wrap::master_system_user_interface::UserInterface::new(&sdl, save_directory, &[])?;
        let master_system_hardware = HardwareBuilder::new()
            .build_from_file::<memory_16_8::codemasters::Component>(filename)
            .unwrap();
        let mut master_system = System::new(NothingInbox, master_system_hardware);
        user_interface.run(&sdl, &mut emulator, &mut master_system)?;
    } else {
        Err(format_err!(
            "Can't happen: Unknown memory map {}",
            memory_map
        ))?;
    }

    Ok(())
}

fn run_playback(matches: &ArgMatches) -> Result<()> {
    let load_filename = matches.value_of("loadfile").unwrap();

    let sdl = sdl2::init().unwrap();

    let mut emulator = sega_master_system::Emulator::new(
        sega_master_system::Frequency::Unlimited,
        <z80::Interpreter as Default>::default(),
        <vdp::SimpleEmulator as Default>::default(),
    );

    let mut load_file = BufReader::with_capacity(1024, File::open(load_filename)?);

    let mut recording =
        <Recording<System<NothingInbox, memory_16_8::sega::Component>> as Tag>::read(
            &mut load_file
        )?;

    let mut user_interface =
        sdl_wrap::master_system_user_interface::PlaybackInterface::new(&recording.player_statuses);

    let start_cycles = <AsRef<z80::Component>>::as_ref(&recording.master_system).cycles;
    let time = user_interface.run(
        &sdl,
        &mut emulator,
        &mut recording.master_system,
    )?;

    let end_cycles = <AsRef<z80::Component>>::as_ref(&recording.master_system).cycles;

    let sec_time = time.as_secs() as f64 + time.subsec_nanos() as f64 * 10e-9;
    println!("Total cycles: {}", end_cycles - start_cycles);
    println!("Time: {} secs", sec_time);
    println!(
        "Frequency: {} Hz",
        (end_cycles - start_cycles) as f64 / sec_time
    );

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

    let mut user_interface =
        sdl_wrap::master_system_user_interface::UserInterface::new(&sdl, save_directory, &[])?;

    let mut load_file = BufReader::with_capacity(1024, File::open(load_filename)?);

    let mut master_system =
        <System<NothingInbox, memory_16_8::sega::Component> as Tag>::read(&mut load_file)?;

    user_interface.run(&sdl, &mut emulator, &mut master_system)?;

    Ok(())
}

fn run_record(matches: &ArgMatches) -> Result<()> {
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

    let mut load_file = BufReader::with_capacity(1024, File::open(load_filename)?);

    let mut recording =
        <Recording<System<NothingInbox, memory_16_8::sega::Component>> as Tag>::read(
            &mut load_file
        )?;

    let mut user_interface = sdl_wrap::master_system_user_interface::UserInterface::new(
        &sdl,
        save_directory,
        &recording.player_statuses,
    )?;

    user_interface.run(
        &sdl,
        &mut emulator,
        &mut recording.master_system,
    )?;

    Ok(())
}

fn run() -> Result<()> {
    let memory_map_arg = Arg::with_name("memorymap")
        .long("memorymap")
        .value_name("(sega|codemasters)")
        .help("Specify the sega or codemasters memory map")
        .takes_value(true)
        .required(true)
        .default_value("sega");
    let save_directory_arg = Arg::with_name("savedirectory")
        .long("savedirectory")
        .value_name("DIRECTORY")
        .help("Specify the directory in which to save states")
        .takes_value(true);

    let app = App::new("Attalus")
        .version("0.1.0")
        .author("Michael Benfield")
        .about("Sega Master System emulator")
        .subcommand(
            SubCommand::with_name("rom")
                .about("Play a game from a ROM image")
                .arg(
                    Arg::with_name("rom")
                        .long("rom")
                        .value_name("FILE")
                        .help("Specify the filename containing a ROM image")
                        .takes_value(true)
                        .required(true),
                )
                .arg(memory_map_arg.clone())
                .arg(save_directory_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("load")
                .about("Load a saved state")
                .arg(save_directory_arg.clone())
                .arg(
                    Arg::with_name("loadfile")
                        .long("loadfile")
                        .value_name("FILE")
                        .help("Specify the saved state file")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("loadrecord")
                .about("Load recorded gameplay")
                .arg(save_directory_arg.clone())
                .arg(
                    Arg::with_name("loadfile")
                        .long("loadfile")
                        .value_name("FILE")
                        .help("Specify the recorded gameplay file")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("playback")
                .about("Play back and time recorded gameplay")
                .arg(
                    Arg::with_name("loadfile")
                        .long("loadfile")
                        .value_name("FILE")
                        .help("Specify the recorded gameplay file")
                        .takes_value(true)
                        .required(true),
                ),
        );
    let matches = app.get_matches();

    return match matches.subcommand() {
        ("rom", Some(sub)) => run_rom(&sub),
        ("load", Some(sub)) => run_load(&sub),
        ("loadrecord", Some(sub)) => run_record(&sub),
        ("playback", Some(sub)) => run_playback(&sub),
        (x, _) => {
            eprintln!("Unknown subcommand {}", x);
            eprintln!("{}", matches.usage());
            return Err(failure::err_msg("No subcommand"));
        }
    };
}

fn main() {
    if let Err(x) = run() {
        eprintln!("{:?}", x);
    }
}
