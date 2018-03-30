#[macro_use]
extern crate attalus;
#[cfg(attalus_x64)]
extern crate attalus_x64;
extern crate clap;
#[macro_use]
extern crate failure;
extern crate sdl2;

use std::path::PathBuf;

use clap::{App, Arg, ArgMatches, SubCommand};
use failure::Error;

use attalus::host_multimedia::{SimpleAudio, SimpleGraphics};
use attalus::hardware::{memory_16_8, z80};
use attalus::memo::NothingInbox;
use attalus::save;
use attalus::sdl_wrap;
use attalus::systems::sega_master_system::{self, Hardware, HardwareBuilder, Recording, System};

type Result<T> = std::result::Result<T, Error>;

fn run_rom(matches: &ArgMatches) -> Result<()> {
    let filename = matches.value_of("rom").unwrap();
    let memory_map = matches.value_of("memorymap").unwrap();
    let save_directory = match matches.value_of("savedirectory") {
        None => None,
        Some(s) => Some(PathBuf::from(s)),
    };

    let sdl = sdl2::init().unwrap();

    type_select! {
        match memory_map {
            "sega" => memory_16_8::sega::T,
            // "codemasters" => memory_16_8::codemasters::T,
        } for M {
            let mut user_interface =
                sdl_wrap::master_system_user_interface::UserInterface::new(
                    &sdl,
                    save_directory,
                    &[]
                )?;
            let audio: Box<SimpleAudio> = Box::new(sdl_wrap::simple_audio::Audio::new(&sdl)?);

            let graphics: Box<SimpleGraphics> = {
                let mut win = sdl_wrap::simple_graphics::Window::new(&sdl)?;
                win.set_size(768, 576);
                win.set_texture_size(256, 192);
                win.set_title("Attalus");
                Box::new(win)
            };

            let master_system_hardware = HardwareBuilder::new()
                .build_from_file::<M>(filename)?;
            let mut master_system = System::new(
                NothingInbox,
                master_system_hardware,
                graphics,
                audio
            );
            user_interface.run(
                &mut master_system,
                sega_master_system::Frequency::Ntsc
            )?;
        } else {
            Err(format_err!(
                "Can't happen: Unknown memory map {}",
                memory_map
            ))?;
        }
    }

    Ok(())
}

fn run_playback(matches: &ArgMatches) -> Result<()> {
    let load_filename = matches.value_of("loadfile").unwrap();

    let sdl = sdl2::init().unwrap();

    let recording: Recording<Hardware<memory_16_8::sega::T>> =
        save::deserialize_at(&load_filename)?;
    let mut user_interface =
        sdl_wrap::master_system_user_interface::PlaybackInterface::new(&recording.player_statuses);

    let audio: Box<SimpleAudio> = Box::new(sdl_wrap::simple_audio::Audio::new(&sdl)?);
    let graphics: Box<SimpleGraphics> = {
        let mut win = sdl_wrap::simple_graphics::Window::new(&sdl)?;
        win.set_size(768, 576);
        win.set_texture_size(256, 192);
        win.set_title("Attalus");
        Box::new(win)
    };

    let mut master_system = System::new(NothingInbox, recording.hardware, graphics, audio);

    let start_cycles = <z80::internal::T>::cycles(&master_system);

    let time = user_interface.run(&mut master_system, sega_master_system::Frequency::Unlimited)?;

    let end_cycles = <z80::internal::T>::cycles(&master_system);

    let sec_time = time.as_secs() as f64 + time.subsec_nanos() as f64 * 1e-9;
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

    let hardware = save::deserialize_at(&load_filename)?;

    let mut user_interface =
        sdl_wrap::master_system_user_interface::UserInterface::new(&sdl, save_directory, &[])?;

    let audio: Box<SimpleAudio> = Box::new(sdl_wrap::simple_audio::Audio::new(&sdl)?);
    let graphics: Box<SimpleGraphics> = {
        let mut win = sdl_wrap::simple_graphics::Window::new(&sdl)?;
        win.set_size(768, 576);
        win.set_texture_size(256, 192);
        win.set_title("Attalus");
        Box::new(win)
    };

    let mut master_system = System::new(NothingInbox, hardware, graphics, audio);
    user_interface.run(&mut master_system, sega_master_system::Frequency::Ntsc)?;

    Ok(())
}

fn run_record(matches: &ArgMatches) -> Result<()> {
    let load_filename = matches.value_of("loadfile").unwrap();
    // let memory_map = matches.value_of("memorymap").unwrap();
    let save_directory = match matches.value_of("savedirectory") {
        None => None,
        Some(s) => Some(PathBuf::from(s)),
    };

    let sdl = sdl2::init().unwrap();

    let recording: Recording<Hardware<memory_16_8::sega::T>> =
        save::deserialize_at(&load_filename)?;
    let mut user_interface = sdl_wrap::master_system_user_interface::UserInterface::new(
        &sdl,
        save_directory,
        &recording.player_statuses,
    )?;

    let audio: Box<SimpleAudio> = Box::new(sdl_wrap::simple_audio::Audio::new(&sdl)?);
    let graphics: Box<SimpleGraphics> = {
        let mut win = sdl_wrap::simple_graphics::Window::new(&sdl)?;
        win.set_size(768, 576);
        win.set_texture_size(256, 192);
        win.set_title("Attalus");
        Box::new(win)
    };

    let mut master_system = System::new(NothingInbox, recording.hardware, graphics, audio);

    user_interface.run(&mut master_system, sega_master_system::Frequency::Ntsc)?;

    Ok(())
}

fn run() -> Result<()> {
    let memory_map_arg = Arg::with_name("memorymap")
        .long("memorymap")
        // .value_name("(sega|codemasters)")
        .value_name("(sega)")
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
    #[cfg(attalus_x64)]
    unsafe {
        attalus_x64::install_pattern_to_palette_indices();
    }

    if let Err(x) = run() {
        eprintln!("{:?}", x);
    }
}
