// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use std;
use std::sync::mpsc;
use std::collections::vec_deque::VecDeque;

use sdl2;

use ::lua::repl;
use ::message::{NothingReceiver, Pausable, Receiver, Sender};
use ::hardware::memory_map::MemoryMap;
use ::hardware::io::sms2::Sms2Io;
use ::hardware::z80::*;
use ::hardware::vdp::*;

pub struct EmulationManager<M: MemoryMap>
{
    z80: Z80<Sms2Io<M>>,
    receiver: DisassemblingReceiver,
}

quick_error! {
    #[derive(Clone, Debug)]
    pub enum Error {
        Custom(s: String) {}

        Screen(err: ScreenError) {
            from()
        }

        Time(err: std::time::SystemTimeError) {
            from()
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Command {
    Hold,
    Resume,
    Z80Status,
    Disassemble,
    Step,
    BreakAtPc(u16),
    RemovePcBreakpoints,
    ShowRecentMessages,
}

const SYSTEM_FREQUENCY: u64 = 10738580;
const AUDIO_BUFFER_SIZE: usize = 0x800;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum DisassemblingReceiverStatus {
    Step,
    None,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
enum DisassemblingReceiverMessage {
    Z80(Z80Message),
}

const MAX_MESSAGES: usize = 50;

#[derive(Clone)]
pub struct DisassemblingReceiver {
    last_pc: u16,
    opcodes: [Option<Opcode>; 0x10000],
    hold: bool,
    status: DisassemblingReceiverStatus,
    pc_breakpoints: Vec<u16>,
    recent_messages: VecDeque<DisassemblingReceiverMessage>,
}

impl Default for DisassemblingReceiver {
    fn default() -> Self {
        DisassemblingReceiver::new()
    }
}

impl DisassemblingReceiver {
    fn new() -> DisassemblingReceiver {
        DisassemblingReceiver {
            last_pc: 0,
            opcodes: [None; 0x10000],
            hold: false,
            status: DisassemblingReceiverStatus:: None,
            pc_breakpoints: Vec::new(),
            recent_messages: VecDeque::new(),
        }
    }

    /// Find the PC pointing at the instruction immediately before pc, if it exists
    fn back_1(&self, pc: u16) -> Option<u16> {
        for i in 1 .. 5 {
            if pc < i {
                return None;
            }
            match (self.opcodes[(pc - i) as usize], i) {
                (Some(Opcode::OneByte(_)), 1) => return Some(pc - i),
                (Some(Opcode::TwoBytes(_)), 2) => return Some(pc - i),
                (Some(Opcode::ThreeBytes(_)), 3) => return Some(pc - i),
                (Some(Opcode::FourBytes(_)), 4) => return Some(pc - i),
                _ => {}
            }
        }
        return None;
    }

    /// Find the earliest PC pointing at an opcode, at most n steps back
    fn back_n(&self, n: u16, pc: u16) -> u16 {
        let mut pc_current = pc;
        for _ in 0 .. n {
            match self.back_1(pc_current) {
                None => return pc_current,
                Some(pc_new) => pc_current = pc_new,
            }
        }
        return pc_current;
    }

    fn disassembly_around(&self, pc: u16) -> String {
        let mut pc_current = self.back_n(5, pc);
        let mut result = "".to_owned();
        for _ in 0 .. 10 {
            let opcode = match self.opcodes[pc_current as usize] {
                None => return result,
                Some(x) => x,
            };
            result.push_str(&format!("{:0>4X}: {: <width$}", pc_current, opcode, width=12));
            match opcode.mnemonic() {
                None => result.push_str("Unknown opcode"),
                Some(x) => result.push_str(&format!("{}", x)),
            }
            if pc_current == pc {
                result.push_str(&" <<<");
            }
            result.push('\n');
            // XXX use wrapping add?
            pc_current += match opcode {
                Opcode::OneByte(_) => { 1 },
                Opcode::TwoBytes(_) => { 2 },
                Opcode::ThreeBytes(_) => { 3 },
                Opcode::FourBytes(_) => { 4 },
            };
        }
        result
    }

    fn receive_loop<F, M>(
        &mut self,
        mpsc_receiver: &mpsc::Receiver<Command>,
        z80: &mut Z80<Sms2Io<M>>,
        f: F
    ) -> Option<std::time::Duration>
    where
        F: Fn(&mut Z80<Sms2Io<M>>) -> bool,
        M: MemoryMap,
        Self: Receiver<<M as Sender>::Message>,
    {
        let mut holding_time = if self.hold {
            self.hold = false;
            Some(std::time::SystemTime::now())
        } else {
            None
        };

        loop {
            match (mpsc_receiver.try_recv(), holding_time) {
                (Ok(Command::Hold), None) => holding_time = Some(std::time::SystemTime::now()),
                (Ok(Command::ShowRecentMessages), _) => {
                    for message in self.recent_messages.iter() {
                        println!("{:?}", message);
                    }
                }
                (Ok(Command::Resume), _) => break,
                (Ok(Command::Z80Status), _) => println!("{}", z80),
                (Ok(Command::Disassemble), _) => println!("{}", self.disassembly_around(z80.get_reg16(PC))),
                (Ok(Command::BreakAtPc(pc)), _) => self.pc_breakpoints.push(pc),
                (Ok(Command::RemovePcBreakpoints), _) => self.pc_breakpoints = Vec::new(),
                (Ok(Command::Step), _) => {
                    self.status = DisassemblingReceiverStatus::Step;
                    break;
                },
                _ => {}
            }
            if f(z80) {
                return None;
            }
            if holding_time.is_none() {
                return Some(std::time::Duration::from_millis(0));
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        match holding_time {
            None => return Some(std::time::Duration::from_millis(0)),
            Some(x) => return Some(x.elapsed().unwrap()),
        }
    }
}

impl Pausable for DisassemblingReceiver {
    fn wants_pause(&self) -> bool { self.hold }
    fn clear_pause(&mut self) {}
}

impl Receiver<Z80Message> for DisassemblingReceiver {
    fn receive(&mut self, _id: u32, message: Z80Message) {
        match message {
            Z80Message::ReadingPcToExecute(pc) => {
                let may_break = match self.recent_messages.back() {
                    Some(&DisassemblingReceiverMessage::Z80(Z80Message::ReadingPcToExecute(pc2))) => {
                        pc != pc2
                    },
                    _ => {
                        true
                    }
                };
                if may_break && self.pc_breakpoints.contains(&pc) {
                    self.hold = true;
                }
            },
            Z80Message::InstructionAtPc(pc) => self.last_pc = pc,
            Z80Message::InstructionOpcode(opcode) => {
                self.opcodes[self.last_pc as usize] = Some(opcode);
                if let DisassemblingReceiverStatus::Step = self.status {
                    self.hold = true;
                    self.status = DisassemblingReceiverStatus::None;
                }
            },
            _ => {},
        }
        if self.recent_messages.len() >= MAX_MESSAGES {
            self.recent_messages.pop_front();
        }
        self.recent_messages.push_back(DisassemblingReceiverMessage::Z80(message));
    }
}

impl Receiver<::hardware::io::sms2::Sms2IoMessage> for DisassemblingReceiver {
    fn receive(&mut self, _id: u32, _message: ::hardware::io::sms2::Sms2IoMessage) {}
}

impl Receiver<::hardware::memory_map::sega_memory_map::SegaMemoryMapMessage> for DisassemblingReceiver {
    fn receive(&mut self, _id: u32, _message: ::hardware::memory_map::sega_memory_map::SegaMemoryMapMessage) {}
}

#[allow(dead_code)]
struct PrintingReceiver;

impl Pausable for PrintingReceiver {
    fn wants_pause(&self) -> bool { false }
    fn clear_pause(&mut self) {}
}

impl<D> Receiver<D> for PrintingReceiver
where
    D: std::fmt::Debug
{
    fn receive(&mut self, id: u32, message: D) {
        println!("{}: {:?}", id, message);
    }
}

impl<M: MemoryMap> EmulationManager<M>
where
    <M as Sender>::Message: std::fmt::Debug,
    DisassemblingReceiver: Receiver<<M as Sender>::Message>,
{
    pub fn new(mm: M) -> EmulationManager<M> {
        let io = Sms2Io::new(mm);
        EmulationManager {
            z80: Z80::new(io),
            receiver: Default::default(),
        }
    }

    pub fn main_loop<S>(
        &mut self,
        screen: &mut S,
        audio: sdl2::AudioSubsystem,
        event_pump: sdl2::EventPump,
    ) -> Result<()>
    where
        S: Screen
    {
        use sdl_wrap;

        let result = audio.open_queue(
            None,
            &sdl2::audio::AudioSpecDesired {
                freq: Some((SYSTEM_FREQUENCY / 48) as i32),
                channels: Some(1),
                samples: Some(AUDIO_BUFFER_SIZE as u16),
            }
        );
        let audio_queue = match result {
            Err(s) => return Err(Error::Custom(s)),
            Ok(a) => a,
        };
        audio_queue.resume();

        let mut audio_buffer = [0i16; AUDIO_BUFFER_SIZE];

        let system_time = std::time::SystemTime::now();

        let mut total_hold_duration = std::time::Duration::new(0, 0);

        let (sender, mpsc_receiver) = std::sync::mpsc::channel::<Command>();

        std::thread::Builder::new()
            .name("Lua REPL".into())
            .spawn(
                || {
                    repl::repl(sender, include_str!("emulation_manager_lua.lua"));
                }
            ).unwrap();

        let z80_cycles_start = self.z80.cycles;

        macro_rules! check_message_receiver {
            () => {
                if let Some(duration) = self.receiver.receive_loop(
                    &mpsc_receiver,
                    &mut self.z80,
                    |_| {
                        sdl_wrap::event::check_quit()
                    }
                ) {
                    total_hold_duration = total_hold_duration.checked_add(duration).unwrap();
                } else {
                    return Ok(())
                }
            }
        }

        loop {
            if self.receiver.wants_pause() {
                check_message_receiver!();
            }

            self.z80.io.vdp.draw_line(screen)?;

            let vdp_cycles = self.z80.io.vdp.cycles;
            let z80_target_cycles = 2 * vdp_cycles / 3;

            while self.z80.cycles < z80_target_cycles {
                if self.receiver.wants_pause() {
                    check_message_receiver!();
                }
                Z80Interpreter {}.run(&mut self.receiver, &mut self.z80, z80_target_cycles);
            }

            if self.z80.io.vdp.read_v() == 0 {
                check_message_receiver!();

                let sound_target_cycles = z80_target_cycles / 16;

                self.z80.io.sn76489.queue(
                    sound_target_cycles,
                    &mut audio_buffer,
                    |buf| {
                        audio_queue.queue(buf);
                    }
                );

                let input_status = sdl_wrap::event::input_status(&event_pump);
                self.z80.io.set_joypad_a(input_status.joypad_a);
                self.z80.io.set_joypad_b(input_status.joypad_b);

                let z80_effective_cycles = self.z80.cycles - z80_cycles_start;
                let total_duration = system_time.elapsed()?;
                let desired_time_seconds = (3 * z80_effective_cycles) / SYSTEM_FREQUENCY;
                let cycles_given_seconds = (desired_time_seconds * SYSTEM_FREQUENCY) / 3;
                let remaining_cycles = z80_effective_cycles - cycles_given_seconds;
                let desired_time_nanos = (3000000000 * remaining_cycles) / SYSTEM_FREQUENCY;
                debug_assert!(desired_time_nanos < 1000000000);
                let desired_duration = std::time::Duration::new(
                    desired_time_seconds,
                    desired_time_nanos as u32
                );
                match desired_duration.checked_add(total_hold_duration)
                    .map(|d| d.checked_sub(total_duration))
                {
                    Some(Some(diff)) => {
                        std::thread::sleep(diff);
                    },
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
