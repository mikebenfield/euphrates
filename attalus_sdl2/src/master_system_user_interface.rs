use std;
use std::path::PathBuf;

use failure::Error;
use sdl2;

use attalus::systems::sega_master_system::{Command, CommandResult, MasterSystem, PlaybackStatus,
                                           PlayerStatus, Query, QueryResult, Ui, UiHelper,
                                           UiStatus, UserMessage};

pub type Result<T> = std::result::Result<T, Error>;

bitflags! {
    struct JoypadPortA: u8 {
        const JOYPAD2_DOWN = 0b10000000;
        const JOYPAD2_UP = 0b01000000;
        const JOYPAD1_B = 0b00100000;
        const JOYPAD1_A = 0b00010000;
        const JOYPAD1_RIGHT = 0b00001000;
        const JOYPAD1_LEFT = 0b00000100;
        const JOYPAD1_DOWN = 0b00000010;
        const JOYPAD1_UP = 0b00000001;
    }
}

bitflags! {
    struct JoypadPortB: u8 {
        const B_TH = 0b10000000;
        const A_TH = 0b01000000;
        const CONT = 0b00100000;
        const RESET = 0b00010000;
        const JOYPAD2_B = 0b00001000;
        const JOYPAD2_A = 0b00000100;
        const JOYPAD2_RIGHT = 0b00000010;
        const JOYPAD2_LEFT = 0b00000001;
    }
}

struct PlaybackHelper(PlaybackStatus);

impl<R> UiHelper<R> for PlaybackHelper {
    fn frame_update(&mut self, _status: &mut UiStatus<R>) -> Result<Option<PlayerStatus>> {
        let option_player_status = self.0.pop();
        if option_player_status.is_some() {
            Ok(option_player_status)
        } else {
            Ok(None)
        }
    }
}

pub fn playback_ui<R>(
    master_system: Box<MasterSystem<R>>,
    player_statuses: &[PlayerStatus],
) -> Result<Ui<R>> {
    let helper = Box::new(PlaybackHelper(PlaybackStatus::from_recorded(
        player_statuses,
    )));

    Ok(Ui::new(master_system, helper, None))
}

struct SdlUiHelper {
    event_pump: sdl2::EventPump,
    playback_status: PlaybackStatus,
}

impl<R> UiHelper<R> for SdlUiHelper {
    fn frame_update(&mut self, status: &mut UiStatus<R>) -> Result<Option<PlayerStatus>> {
        use sdl2::keyboard::Scancode::*;

        for message in status.messages() {
            match message {
                UserMessage::Ok(s) => println!("{}", s),
                UserMessage::Error(s) => eprintln!("{}", s),
                UserMessage::Fatal(s) => {
                    eprintln!("{}", s);
                    return Err(format_err!("{}", s));
                }
            }
        }

        let mut player_status = PlayerStatus::default();

        fn do_command<R>(status: &mut UiStatus<R>, command: Command) {
            if CommandResult::Unsupported == status.master_system_mut().command(command) {
                eprintln!("Unsupported command {:?}", command);
            }
        }

        fn do_query<R>(status: &UiStatus<R>, query: Query) {
            match status.master_system().query(query) {
                QueryResult::Ok(s) => println!("{}", s),
                QueryResult::Unsupported => eprintln!("Unsupported query {:?}", query),
            }
        }

        for event in self.event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => return Ok(None),
                sdl2::event::Event::KeyDown {
                    scancode: Some(k),
                    keymod,
                    ..
                } => match (
                    k,
                    keymod.contains(sdl2::keyboard::LSHIFTMOD)
                        || keymod.contains(sdl2::keyboard::RSHIFTMOD),
                ) {
                    (P, _) => player_status.pause = true,
                    (R, false) => status.begin_recording(),
                    (R, true) => status.save_recording(None),
                    (Z, _) => status.save_state(None),
                    (M, false) => do_query(status, Query::RecentMemos),
                    (N, false) => {
                        use attalus::hardware::z80::Reg16::PC;
                        let pc = status.master_system().reg16(PC);
                        do_query(status, Query::DisassemblyAt(pc));
                    }
                    (N, true) => do_query(status, Query::Disassembly),
                    (H, false) => do_command(status, Command::Hold),
                    (H, true) => do_command(status, Command::Resume),
                    _ => {}
                },
                _ => {}
            }
        }

        let keyboard_state = self.event_pump.keyboard_state();

        let mut joypad_a = JoypadPortA::all();
        let array_a = [
            (W, JoypadPortA::JOYPAD1_UP),
            (A, JoypadPortA::JOYPAD1_LEFT),
            (S, JoypadPortA::JOYPAD1_DOWN),
            (D, JoypadPortA::JOYPAD1_RIGHT),
            (F, JoypadPortA::JOYPAD1_A),
            (G, JoypadPortA::JOYPAD1_B),
            (I, JoypadPortA::JOYPAD1_UP),
            (K, JoypadPortA::JOYPAD1_DOWN),
        ];
        array_a
            .iter()
            .filter(|(scancode, _)| keyboard_state.is_scancode_pressed(*scancode))
            .for_each(|(_, button)| joypad_a.remove(*button));
        player_status.joypad_a = joypad_a.bits;

        let mut joypad_b = JoypadPortB::all();
        let array_b = [
            (J, JoypadPortB::JOYPAD2_LEFT),
            (L, JoypadPortB::JOYPAD2_RIGHT),
            (Semicolon, JoypadPortB::JOYPAD2_A),
            (Apostrophe, JoypadPortB::JOYPAD2_B),
            (Space, JoypadPortB::RESET),
        ];
        array_b
            .iter()
            .filter(|(scancode, _)| keyboard_state.is_scancode_pressed(*scancode))
            .for_each(|(_, button)| joypad_b.remove(*button));
        player_status.joypad_b = joypad_b.bits;

        if player_status != Default::default() {
            self.playback_status.end_playback();
        } else if let Some(ps) = self.playback_status.pop() {
            player_status = ps;
        }

        Ok(Some(player_status))
    }
}

pub fn ui<R>(
    master_system: Box<MasterSystem<R>>,
    sdl: &sdl2::Sdl,
    save_directory: Option<PathBuf>,
    player_statuses: &[PlayerStatus],
) -> Result<Ui<R>> {
    sdl.event()
        .map_err(|s| format_err!("Error initializing the SDL event subsystem {}", s))?;

    let event_pump = sdl.event_pump()
        .map_err(|s| format_err!("Error obtaining the SDL event pump {}", s))?;

    let helper = Box::new(SdlUiHelper {
        event_pump,
        playback_status: PlaybackStatus::from_recorded(player_statuses),
    });

    Ok(Ui::new(master_system, helper, save_directory))
}
