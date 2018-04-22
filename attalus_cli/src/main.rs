extern crate attalus;
extern crate attalus_sdl2;
#[cfg(attalus_x64)]
extern crate attalus_x64;
extern crate clap;
extern crate failure;
extern crate sdl2;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use clap::{App, Arg, ArgMatches, SubCommand};
use failure::Error;
use sdl2::Sdl;

use attalus::hardware::z80::Z80Internal;
use attalus::save;
use attalus::systems::sega_master_system::{self, MasterSystem, MasterSystemResource,
                                           MasterSystemState, Recording, SimpleMultimediaResource};
use attalus_sdl2::master_system_user_interface;
use attalus_sdl2::{simple_audio::Audio, simple_graphics::Window};

type Result<T> = std::result::Result<T, Error>;

fn new_master_system(
    state: MasterSystemState,
    sdl: &Sdl,
    frequency: sega_master_system::Frequency,
    debug: bool,
) -> Result<Box<MasterSystem<SimpleMultimediaResource<Audio, Window>>>> {
    let audio = attalus_sdl2::simple_audio::Audio::new(&sdl)?;

    let mut graphics = attalus_sdl2::simple_graphics::Window::new(&sdl)?;
    graphics.set_size(768, 576);
    graphics.set_texture_size(256, 192);
    graphics.set_title("Attalus");

    Ok(SimpleMultimediaResource {
        graphics,
        audio,
        debug,
        frequency,
    }.create(state, Default::default()))
}

fn run_rom(matches: &ArgMatches) -> Result<()> {
    let filename = matches.value_of("rom").unwrap();
    let memory_map = matches.value_of("memorymap").unwrap();
    let save_directory = match matches.value_of("savedirectory") {
        None => None,
        Some(s) => Some(PathBuf::from(s)),
    };
    let debug = matches.value_of("debug").unwrap() == "true";

    let rom = {
        let mut contents = Vec::new();
        File::open(&filename)?.read_to_end(&mut contents)?;
        contents
    };

    let sdl = sdl2::init().unwrap();

    let state = if memory_map == "sega" {
        MasterSystemState::new_with_sega_memory_map(&rom)?
    } else {
        MasterSystemState::new_with_codemasters_memory_map(&rom)?
    };

    let master_system = new_master_system(state, &sdl, sega_master_system::Frequency::Ntsc, debug)?;

    let mut user_interface =
        master_system_user_interface::ui(master_system, &sdl, save_directory, &[])?;
    user_interface.run()?;

    Ok(())
}

fn run_playback(matches: &ArgMatches) -> Result<()> {
    use std::time::Instant;

    let load_filename = matches.value_of("loadfile").unwrap();

    let sdl = sdl2::init().unwrap();

    let recording: Recording<MasterSystemState> = save::deserialize_at(&load_filename)?;

    let master_system = new_master_system(
        recording.state,
        &sdl,
        sega_master_system::Frequency::Unlimited,
        false,
    )?;

    let mut user_interface = attalus_sdl2::master_system_user_interface::playback_ui(
        master_system,
        &recording.player_statuses,
    )?;

    let start_cycles = Z80Internal::cycles(user_interface.master_system());
    let start_time = Instant::now();

    user_interface.run()?;

    let end_cycles = Z80Internal::cycles(user_interface.master_system());
    let end_time = Instant::now();

    let time = end_time.duration_since(start_time);

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
    let debug = matches.value_of("debug").unwrap() == "true";

    let sdl = sdl2::init().unwrap();

    let state: MasterSystemState = save::deserialize_at(&load_filename)?;

    let master_system = new_master_system(state, &sdl, sega_master_system::Frequency::Ntsc, debug)?;

    let mut user_interface =
        master_system_user_interface::ui(master_system, &sdl, save_directory, &[])?;

    user_interface.run()?;

    Ok(())
}

fn run_record(matches: &ArgMatches) -> Result<()> {
    let load_filename = matches.value_of("loadfile").unwrap();
    let save_directory = match matches.value_of("savedirectory") {
        None => None,
        Some(s) => Some(PathBuf::from(s)),
    };
    let debug = matches.value_of("debug").unwrap() == "true";

    let sdl = sdl2::init().unwrap();

    let recording: Recording<MasterSystemState> = save::deserialize_at(&load_filename)?;
    let master_system = new_master_system(
        recording.state,
        &sdl,
        sega_master_system::Frequency::Ntsc,
        debug,
    )?;

    let mut user_interface =
        master_system_user_interface::ui(master_system, &sdl, save_directory, &[])?;

    user_interface.run()?;

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

    let debug_arg = Arg::with_name("debug")
        .long("debug")
        .value_name("BOOL")
        .help("Enable or disable debugging")
        .takes_value(true)
        .possible_values(&["true", "false"])
        .default_value("false");

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
                .arg(debug_arg.clone())
                .arg(memory_map_arg.clone())
                .arg(save_directory_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("load")
                .about("Load a saved state")
                .arg(debug_arg.clone())
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
                .arg(debug_arg.clone())
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
