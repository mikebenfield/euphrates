// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use std::collections::VecDeque;
use std::fmt::Write;

use hardware::io_16_8;
use hardware::memory_16_8;
use hardware::vdp;
use hardware::z80::{self, Opcode};
use memo::{Inbox, NothingInbox, Pausable};
use runtime_pattern::{Matchable, WholePattern};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Matchable)]
pub enum Memo {
    Z80(z80::Memo),
    Memory(memory_16_8::sega::Memo),
    Io(io_16_8::sms2::Memo),
    Vdp(vdp::Memo),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Query {
    Disassemble(u16),
    RecentMemos,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Command {
    Hold,
    Resume,
    Step,
    BreakAtPc(u16),
    RemovePcBreakpoints,
    BreakAtMemo(MemoPattern),
    RemoveBreakMemos,
}

pub trait MasterSystemInbox
    : Inbox<Memo>
    + Inbox<z80::Memo>
    + Inbox<vdp::Memo>
    + Inbox<memory_16_8::sega::Memo>
    + Inbox<io_16_8::sms2::Memo>
    + Default {
    fn command(&mut self, command: Command);
    fn query(&mut self, query: Query) -> String;
}

impl MasterSystemInbox for NothingInbox {
    #[inline]
    fn command(&mut self, _command: Command) {}

    #[inline]
    fn query(&mut self, _query: Query) -> String {
        String::new()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
enum DebugStatus {
    Step,
    None,
}

impl Default for DebugStatus {
    fn default() -> Self {
        DebugStatus::None
    }
}

const MAX_MESSAGES: usize = 50;

#[derive(Clone)]
pub struct DebuggingInbox {
    last_pc: u16,
    opcodes: [Option<Opcode>; 0x10000],
    hold: bool,
    status: DebugStatus,
    pc_breakpoints: Vec<u16>,
    memo_patterns: Vec<MemoPattern>,
    recent_memos: VecDeque<Memo>,
}

serde_struct_arrays!{
    impl_serde,
    DebuggingInbox,
    [last_pc, hold, status, pc_breakpoints, memo_patterns, recent_memos,],
    [opcodes: [Option<::hardware::z80::Opcode>; 0x10000],],
    []
}


impl Default for DebuggingInbox {
    fn default() -> Self {
        DebuggingInbox::new()
    }
}

impl Pausable for DebuggingInbox {
    #[inline]
    fn wants_pause(&self) -> bool {
        self.hold
    }

    #[inline]
    fn clear_pause(&mut self) {}
}

macro_rules! impl_inbox {
    ($typename: ty, $variant: ident) => {
        impl Inbox<$typename> for DebuggingInbox {
            #[inline]
            fn receive(&mut self, _id: u32, memo: $typename) {
                self.receive_general(
                    Memo::$variant(memo)
                )
            }
        }
    }
}

impl_inbox!{z80::Memo, Z80}
impl_inbox!{memory_16_8::sega::Memo, Memory}
impl_inbox!{io_16_8::sms2::Memo, Io}
impl_inbox!{vdp::Memo, Vdp}

impl Inbox<Memo> for DebuggingInbox {
    #[inline]
    fn receive(&mut self, _id: u32, memo: Memo) {
        self.receive_general(memo)
    }
}

impl DebuggingInbox {
    fn new() -> Self {
        DebuggingInbox {
            last_pc: 0,
            opcodes: [None; 0x10000],
            hold: true,
            status: DebugStatus::None,
            pc_breakpoints: Vec::new(),
            memo_patterns: Vec::new(),
            recent_memos: VecDeque::new(),
        }
    }

    fn receive_general(&mut self, message: Memo) {
        if self.recent_memos.len() >= MAX_MESSAGES {
            self.recent_memos.pop_front();
        }
        if self.memo_patterns.iter().any(|pattern| {
            let mut patt: WholePattern<Memo, MemoPattern> = WholePattern::Patt(pattern.clone());
            message.matc(&mut patt)
        })
        {
            self.hold = true;
        }
        self.recent_memos.push_back(message);
    }

    /// Find the PC pointing at the instruction immediately before pc, if it exists
    fn back_1(&self, pc: u16) -> Option<u16> {
        for i in 1..5 {
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
        for _ in 0..n {
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
        for _ in 0..10 {
            let opcode = match self.opcodes[pc_current as usize] {
                None => return result,
                Some(x) => x,
            };
            result.push_str(&format!(
                "{:0>4X}: {: <width$}",
                pc_current,
                opcode,
                width = 12
            ));
            match opcode.mnemonic() {
                None => result.push_str("Unknown opcode"),
                Some(x) => result.push_str(&format!("{}", x)),
            }
            if pc_current == pc {
                result.push_str(" <<<");
            }
            result.push('\n');
            // XXX use wrapping add?
            pc_current += match opcode {
                Opcode::OneByte(_) => 1,
                Opcode::TwoBytes(_) => 2,
                Opcode::ThreeBytes(_) => 3,
                Opcode::FourBytes(_) => 4,
            };
        }
        result
    }
}

impl MasterSystemInbox for DebuggingInbox {
    fn query(&mut self, query: Query) -> String {
        use self::Query::*;
        match query {
            RecentMemos => {
                let mut result = String::new();
                for memo in self.recent_memos.iter() {
                    writeln!(result, "{:?}", memo).unwrap();
                }
                result
            }
            Disassemble(pc) => format!("{}", self.disassembly_around(pc)),
        }
    }

    fn command(&mut self, command: Command) {
        use self::Command::*;
        match command {
            Hold => self.hold = true,
            Resume => self.hold = false,
            BreakAtPc(pc) => self.pc_breakpoints.push(pc),
            RemovePcBreakpoints => self.pc_breakpoints = Vec::new(),
            Step => self.status = DebugStatus::Step,
            BreakAtMemo(pattern) => self.memo_patterns.push(pattern),
            RemoveBreakMemos => self.memo_patterns = Vec::new(),
        }

        // let mut holding_time = if self.hold {
        //     self.hold = false;
        //     Some(std::time::SystemTime::now())
        // } else {
        //     None
        // };

        //     if f(z80) {
        //         return None;
        //     }
        //     if holding_time.is_none() {
        //         return Some(std::time::Duration::from_millis(0));
        //     }
        //     std::thread::sleep(std::time::Duration::from_millis(50));
        // }
        // match holding_time {
        //     None => return Some(std::time::Duration::from_millis(0)),
        //     Some(x) => return Some(x.elapsed().unwrap()),
        // }
    }
}
