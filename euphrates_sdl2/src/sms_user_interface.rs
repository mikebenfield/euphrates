use std::path::PathBuf;

use failure::Error;

use sdl2;

use euphrates::systems::sms::{
    joypad_a_bits, joypad_b_bits, Command, PlaybackStatus, Query, Sms, SmsEmulationError,
    SmsPlayerInput, Ui, UiHelper, UiStatus, UserMessage, Z80Display,
};

struct PlaybackHelper(PlaybackStatus);

impl UiHelper for PlaybackHelper {
    fn frame_update(
        &mut self,
        _status: &mut UiStatus,
    ) -> Result<Option<SmsPlayerInput>, SmsEmulationError> {
        let option_player_status = self.0.pop();
        if option_player_status.is_some() {
            Ok(option_player_status)
        } else {
            Ok(None)
        }
    }
}

pub fn playback_ui(master_system: Box<dyn Sms>, player_statuses: &[SmsPlayerInput]) -> Ui {
    let helper = Box::new(PlaybackHelper(PlaybackStatus::from_recorded(
        player_statuses,
    )));

    Ui::new(master_system, helper, None)
}

struct SdlUiHelper {
    event_pump: sdl2::EventPump,
    playback_status: PlaybackStatus,
}

impl UiHelper for SdlUiHelper {
    fn frame_update(
        &mut self,
        status: &mut UiStatus,
    ) -> Result<Option<SmsPlayerInput>, SmsEmulationError> {
        use sdl2::keyboard::Scancode::*;

        for message in status.messages() {
            match message {
                UserMessage::Ok(s) => println!("{}", s),
                UserMessage::Error(s) => eprintln!("{}", s),
                UserMessage::Fatal(s) => {
                    eprintln!("{}", s);
                    panic!("XXX");
                }
            }
        }

        let mut player_status = SmsPlayerInput::default();

        #[allow(dead_code)]
        fn do_command(status: &mut UiStatus, command: Command) {
            if let Some(d) = status.master_system_mut().debugger() {
                d.command(command)
            } else {
                eprintln!("Unsupported command {:?}", command);
            }
        }

        fn do_query(status: &mut UiStatus, query: Query) {
            if let Some(d) = status.master_system_mut().debugger() {
                println!("{}", d.query(query));
            } else {
                eprintln!("Unsupported query {:?}", query);
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
                    (Z, _) => println!("{}", Z80Display(status.master_system().z80())),
                    (P, _) => player_status.set_pause(true),
                    (R, false) => status.begin_recording(),
                    (R, true) => status.save_recording(None),
                    (X, _) => status.save_state(None),
                    (M, false) => do_query(status, Query::RecentMemos),
                    (Y, _) => do_command(status, Command::Step),
                    (N, false) => {
                        use euphrates::hardware::z80::Reg16::PC;
                        let pc = status.master_system().z80().reg16(PC);
                        do_query(status, Query::DisassemblyAt(pc));
                    }
                    (N, true) => do_query(status, Query::Disassembly),
                    (H, false) => status.master_system_mut().hold()?,
                    (H, true) => status.master_system_mut().resume()?,
                    _ => {}
                },
                _ => {}
            }
        }

        let keyboard_state = self.event_pump.keyboard_state();

        let mut joypad_a = 0xFF;
        let array_a = [
            (W, joypad_a_bits::JOYPAD1_UP),
            (A, joypad_a_bits::JOYPAD1_LEFT),
            (S, joypad_a_bits::JOYPAD1_DOWN),
            (D, joypad_a_bits::JOYPAD1_RIGHT),
            (F, joypad_a_bits::JOYPAD1_A),
            (G, joypad_a_bits::JOYPAD1_B),
            (I, joypad_a_bits::JOYPAD2_UP),
            (K, joypad_a_bits::JOYPAD2_DOWN),
        ];
        array_a
            .iter()
            .filter(|(scancode, _)| keyboard_state.is_scancode_pressed(*scancode))
            .for_each(|(_, bit)| joypad_a &= !*bit);
        player_status.set_joypad_a(joypad_a);

        let mut joypad_b = 0xFF;
        let array_b = [
            (J, joypad_b_bits::JOYPAD2_LEFT),
            (L, joypad_b_bits::JOYPAD2_RIGHT),
            (Semicolon, joypad_b_bits::JOYPAD2_A),
            (Apostrophe, joypad_b_bits::JOYPAD2_B),
            (Space, joypad_b_bits::RESET),
        ];
        array_b
            .iter()
            .filter(|(scancode, _)| keyboard_state.is_scancode_pressed(*scancode))
            .for_each(|(_, bit)| joypad_b &= !*bit);
        player_status.set_joypad_b(joypad_b);

        if player_status != Default::default() {
            self.playback_status.end_playback();
        } else if let Some(ps) = self.playback_status.pop() {
            player_status = ps;
        }

        Ok(Some(player_status))
    }
}

/// May return an error if there are problems with SDL
pub fn ui(
    master_system: Box<dyn Sms>,
    sdl: &sdl2::Sdl,
    save_directory: Option<PathBuf>,
    player_statuses: &[SmsPlayerInput],
) -> Result<Ui, Error> {
    sdl.event()
        .map_err(|s| format_err!("Error initializing the SDL event subsystem {}", s))?;

    let event_pump = sdl
        .event_pump()
        .map_err(|s| format_err!("Error obtaining the SDL event pump {}", s))?;

    let helper = Box::new(SdlUiHelper {
        event_pump,
        playback_status: PlaybackStatus::from_recorded(player_statuses),
    });

    Ok(Ui::new(master_system, helper, save_directory))
}
