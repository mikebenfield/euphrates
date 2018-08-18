#![deny(bare_trait_objects, anonymous_parameters)]

extern crate clap;
extern crate euphrates;
extern crate euphrates_sdl2;
#[cfg(euphrates_x64)]
extern crate euphrates_x64;
extern crate failure;
extern crate sdl2;

use std::path::PathBuf;
use std::sync::Arc;

use clap::{App, Arg, ArgMatches, SubCommand};
use failure::Error;
use sdl2::Sdl;

use euphrates::hardware::sms_roms;
use euphrates::hardware::sn76489::{FakeSn76489, Sn76489State};
use euphrates::host_multimedia::{FakeAudio, FakeGraphics};
use euphrates::memo::NothingInbox;
use euphrates::save;
use euphrates::systems::sms::{
    self, BoxedInbox, DebuggingInbox, Kind, MemWrap, Recording, Sms, SmsMemoryMapper, SmsState,
    TvSystem,
};

use euphrates_sdl2::sms_user_interface;
use euphrates_sdl2::{simple_audio::Audio, simple_graphics::Window};

#[cfg(state_memory)]
type MemoryType = sms::SmsMemoryState;

#[cfg(not(state_memory))]
type MemoryType = sms::PointerSmsMemory;

type Result<T> = std::result::Result<T, Error>;

fn new_sms(sdl: &Sdl, state: SmsState, matches: &ArgMatches) -> Result<Box<dyn Sms>> {
    let frequency = match matches.value_of("frequency").expect("unwrapping frequency") {
        "ntsc" => Some(sms::NTSC_Z80_FREQUENCY),
        "pal" => Some(sms::PAL_Z80_FREQUENCY),
        "unlimited" => None,
        x => Some(x.parse::<u64>().unwrap()),
    };

    macro_rules! eval_args {
        ($sn76489:expr, $audio:expr, $inbox:expr, $graphics:expr) => {
            Ok(sms::new_sms(
                frequency,
                state,
                $graphics,
                $audio,
                $sn76489,
                $inbox,
                MemWrap::<MemoryType>::default(),
            )?)
        };
        ($sn76489:expr, $audio:expr, $inbox:expr) => {
            match matches.value_of("graphics").expect("unwrapping graphics") {
                "true" => {
                    let mut graphics = Window::new(&sdl)?;
                    graphics.set_size(768, 576);
                    graphics.set_texture_size(256, 192);
                    graphics.set_title("Euphrates");
                    eval_args!($sn76489, $audio, $inbox, graphics)
                }
                _ => eval_args!($sn76489, $audio, $inbox, FakeGraphics::default()),
            }
        };
        ($sn76489:expr, $audio:expr) => {
            match matches.value_of("debug").expect("unwrapping debug") {
                "true" => {
                    eval_args!($sn76489, $audio, BoxedInbox::new(DebuggingInbox::default()))
                }
                _ => eval_args!($sn76489, $audio, BoxedInbox::new(NothingInbox::default())),
            }
        };
        () => {
            match matches.value_of("sound").expect("unwrapping sound") {
                "true" => eval_args!(Sn76489State::default(), Audio::new(sdl)?),
                _ => eval_args!(FakeSn76489, FakeAudio),
            }
        };
    }

    eval_args!()
}

fn run_rom(matches: &ArgMatches) -> Result<()> {
    let rom = {
        let filename = matches.value_of("rom").unwrap();
        sms_roms::from_file(&filename)?
    };
    let tv_system = match matches.value_of("tv").unwrap() {
        "NTSC" => TvSystem::Ntsc,
        _ => TvSystem::Pal,
    };
    let kind = match matches.value_of("kind").unwrap() {
        "sms" => Kind::Sms,
        "sms2" => Kind::Sms2,
        _ => Kind::Gg,
    };
    let memory_mapper = match matches.value_of("memory_map").unwrap() {
        "sg1000_1" => SmsMemoryMapper::Sg1000(1),
        "sg1000_2" => SmsMemoryMapper::Sg1000(2),
        "sg1000_4" => SmsMemoryMapper::Sg1000(4),
        "codemasters" => SmsMemoryMapper::Codemasters,
        _ => SmsMemoryMapper::Sega,
    };

    let state = SmsState::from_rom(Arc::new(rom), memory_mapper, tv_system, kind);

    let sdl = sdl2::init().unwrap();

    let sms = new_sms(&sdl, state, matches)?;

    let save_directory = match matches.value_of("save_directory") {
        None => None,
        Some(s) => Some(PathBuf::from(s)),
    };

    let mut user_interface = sms_user_interface::ui(sms, &sdl, save_directory, &[])?;
    user_interface.run()?;

    Ok(())
}

fn run_playback(matches: &ArgMatches) -> Result<()> {
    use std::time::Instant;

    let load_filename = matches.value_of("loadfile").unwrap();

    let sdl = sdl2::init().unwrap();

    let recording: Recording<SmsState> = save::deserialize_at(&load_filename)?;

    let sms = new_sms(&sdl, recording.state, matches)?;

    let mut user_interface =
        euphrates_sdl2::sms_user_interface::playback_ui(sms, &recording.player_statuses);

    let start_cycles = user_interface.master_system().z80().cycles();
    let start_time = Instant::now();

    user_interface.run()?;

    let end_cycles = user_interface.master_system().z80().cycles();
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
    let save_directory = match matches.value_of("save_directory") {
        None => None,
        Some(s) => Some(PathBuf::from(s)),
    };

    let sdl = sdl2::init().unwrap();

    let state: SmsState = save::deserialize_at(&load_filename)?;

    let sms = new_sms(&sdl, state, matches)?;

    let mut user_interface = sms_user_interface::ui(sms, &sdl, save_directory, &[])?;

    user_interface.run()?;

    Ok(())
}

fn run_record(matches: &ArgMatches) -> Result<()> {
    let load_filename = matches.value_of("loadfile").unwrap();
    let save_directory = match matches.value_of("save_directory") {
        None => None,
        Some(s) => Some(PathBuf::from(s)),
    };

    let sdl = sdl2::init().unwrap();

    let recording: Recording<SmsState> = save::deserialize_at(&load_filename)?;
    let sms = new_sms(&sdl, recording.state, matches)?;

    let mut user_interface =
        sms_user_interface::ui(sms, &sdl, save_directory, &recording.player_statuses)?;

    user_interface.run()?;

    Ok(())
}

fn run() -> Result<()> {
    let memory_map_arg = Arg::with_name("memory_map")
        .long("memory_map")
        .value_name("(sega|codemasters|sg1000_1|sg1000_2|sg1000_4)")
        .help("Specify the sega, codemasters, or sg1000 memory map.")
        .takes_value(true)
        .possible_values(&["sega", "codemasters", "sg1000_1", "sg1000_2", "sg1000_4"])
        .default_value("sega");
    let save_directory_arg = Arg::with_name("save_directory")
        .long("save_directory")
        .value_name("DIRECTORY")
        .help("Specify the directory in which to save states")
        .takes_value(true);

    let kind_arg = Arg::with_name("kind")
        .long("kind")
        .value_name("(sms|sms2|gg)")
        .help("Use the SMS, SMS2, or Game Gear VDP")
        .takes_value(true)
        .required(true)
        .possible_values(&["sms", "sms2", "gg"])
        .default_value("sms2");

    let debug_arg = Arg::with_name("debug")
        .long("debug")
        .value_name("BOOL")
        .help("Enable or disable debugging")
        .takes_value(true)
        .possible_values(&["true", "false"])
        .default_value("false");

    let tv_arg = Arg::with_name("tv")
        .long("tv")
        .value_name("(NTSC|PAL)")
        .help("Use an NTSC or PAL Video Display Processor")
        .takes_value(true)
        .possible_values(&["NTSC", "PAL"])
        .default_value("NTSC");

    let frequency_validator = |s: String| {
        match s.as_ref() {
            "ntsc" | "pal" | "unlimited" => return Ok(()),
            _ => {}
        }
        if let Err(_) = s.parse::<u64>() {
            return Err("frequency must be ntsc, pal, unlimited, or a positive integer".to_owned());
        }
        Ok(())
    };
    let frequency_arg = Arg::with_name("frequency")
        .long("frequency")
        .value_name("(unlimited|ntsc|pal|number)")
        .takes_value(true)
        .default_value("ntsc")
        .validator(frequency_validator)
        .help("Frequency of the Z80 processor");

    let sound_arg = Arg::with_name("sound")
        .long("sound")
        .value_name("BOOL")
        .takes_value(true)
        .default_value("true")
        .possible_values(&["true", "false"])
        .help("Should sound be played?");

    let graphics_arg = Arg::with_name("graphics")
        .long("graphics")
        .value_name("BOOL")
        .takes_value(true)
        .default_value("true")
        .possible_values(&["true", "false"])
        .help("Should graphics be displayed?");

    let app = App::new("Euphrates")
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
                .arg(tv_arg.clone())
                .arg(debug_arg.clone())
                .arg(memory_map_arg.clone())
                .arg(save_directory_arg.clone())
                .arg(kind_arg.clone())
                .arg(sound_arg.clone())
                .arg(graphics_arg.clone())
                .arg(frequency_arg.clone())
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
                )
                .arg(frequency_arg.clone())
                .arg(sound_arg.clone())
                .arg(graphics_arg.clone())
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
                )
                .arg(frequency_arg.clone())
                .arg(sound_arg.clone())
                .arg(graphics_arg.clone())
        )
        .subcommand(
            SubCommand::with_name("playback")
                .about("Play back and time recorded gameplay")
                .arg(debug_arg.clone())
                .arg(
                    Arg::with_name("loadfile")
                        .long("loadfile")
                        .value_name("FILE")
                        .help("Specify the recorded gameplay file")
                        .takes_value(true)
                        .required(true),
                )
                .arg(frequency_arg.clone())
                .arg(sound_arg.clone())
                .arg(graphics_arg.clone())
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
    #[cfg(euphrates_x64)]
    unsafe {
        euphrates_x64::install_pattern_to_palette_indices();
    }

    if let Err(x) = run() {
        eprintln!("{:?}", x);
    }
}
