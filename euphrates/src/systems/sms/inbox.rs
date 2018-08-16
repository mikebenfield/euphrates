use std::collections::VecDeque;
use std::fmt::Write;

use hardware::z80::{Opcode, TargetMnemonic};
use memo::{Inbox, NothingInbox};

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Query {
    /// Show disassembly around this PC, with arrows pointing at this instruction
    DisassemblyAt(u16),
    /// Whole program disassembly
    Disassembly,
    /// Show the last few memos received
    RecentMemos,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Command {
    Step,
    BreakAtPc(u16),
    RemovePcBreakpoints,
    // BreakAtMemo(MemoPattern),
    // RemoveBreakMemos,
}

pub trait Debugger {
    fn command(&mut self, command: Command);
    fn query(&self, query: Query) -> String;
}

pub struct DebuggerImpl;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
enum DebugStatus {
    None,
    Step,
}

impl Default for DebugStatus {
    fn default() -> Self {
        DebugStatus::None
    }
}

/// Debugging information about the instruction at each memory location.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
struct MemoryLocation {
    opcode: Option<Opcode>,
    /// If this PC is the target of some jump, here's a label to use.
    label: Option<u16>,
}

const MAX_MEMOS: usize = 400;

#[derive(Clone)]
pub struct DebuggingInbox {
    last_pc: u16,
    instructions: [MemoryLocation; 0x10000],
    next_label: u16,
    status: DebugStatus,
    pc_breakpoints: Vec<u16>,
    // memo_patterns: Vec<MemoPattern>,
    recent_memos: VecDeque<Z80Memo>,
}

impl DebuggingInbox {
    fn new() -> Self {
        DebuggingInbox {
            last_pc: 0,
            instructions: [Default::default(); 0x10000],
            next_label: 0,
            status: DebugStatus::None,
            pc_breakpoints: Vec::new(),
            recent_memos: VecDeque::new(),
        }
    }

    /// Find the PC pointing at the instruction immediately before pc, if it exists
    fn back_1(&self, pc: u16) -> Option<u16> {
        for i in 1..5 {
            if let Some(opcode) = self.instructions[pc.wrapping_sub(i) as usize].opcode {
                if opcode.len() == i as usize {
                    return Some(pc.wrapping_sub(i));
                }
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

    /// `current`: if provided, arrows will be pointing at this instruction
    ///
    /// `first`: first memory address to start looking for disassembly
    ///
    /// `last`: last memory address to look for disassembly
    fn disassembly(&self, current: Option<u16>, first: u16, last: u16) -> String {
        let mut prev_instruction_pc = first;
        let mut prev_instruction_size = 0u16;
        let mut result = String::new();

        macro_rules! w {
            ($($args: expr),*) => {
                write!(result, $($args),*).unwrap()
            }
        }

        let last_str = |pc: u16| -> &'static str {
            match current {
                Some(x) if x == pc => "<<<<<",
                _ => "",
            }
        };

        for pc in first..=last {
            let location = self.instructions[pc as usize];
            let opcode = match location.opcode {
                None => continue,
                Some(x) => x,
            };
            if pc - prev_instruction_pc < prev_instruction_size {
                result.write_str("<Instruction overlap>\n").unwrap();
            } else if pc - prev_instruction_pc > prev_instruction_size {
                result.write_str("<Gap>\n").unwrap();
            }
            prev_instruction_pc = pc;
            prev_instruction_size = opcode.len() as u16;
            match location.label {
                None => w!("        {:0>4X} ", pc),
                Some(x) => w!("L_{:0>4X}: {:0>4X} ", x, pc),
            }
            let mnemonic = match opcode.mnemonic() {
                None => {
                    w!(
                        "{:<30} {}\n",
                        format!("{} <Unknown instruction>", opcode),
                        last_str(pc)
                    );
                    continue;
                }
                Some(x) => x,
            };
            let target = match mnemonic.jump_target(pc) {
                None => {
                    w!("{:<30} {}\n", mnemonic, last_str(pc));
                    continue;
                }
                Some(x) => x,
            };
            match self.instructions[target as usize].label {
                Some(label) => w!(
                    "{:<30} {}\n",
                    TargetMnemonic {
                        full_mnemonic: mnemonic,
                        target_label: &format!("L_{:0>4X}", label),
                    },
                    last_str(pc)
                ),
                _ => w!("{:<30} {}\n", mnemonic, last_str(pc)),
            }
        }
        result
    }

    fn disassembly_around(&self, pc: u16) -> String {
        let start = self.back_n(8, pc);
        self.disassembly(Some(pc), start, pc + 40)
    }
}

impl Default for DebuggingInbox {
    fn default() -> Self {
        DebuggingInbox::new()
    }
}

impl Inbox for DebuggingInbox {
    type Memo = Z80Memo;

    fn receive_impl(&mut self, memo: Z80Memo) {
        if self.recent_memos.len() >= MAX_MEMOS {
            self.recent_memos.pop_front();
        }

        if let Z80Memo::Instruction { pc, opcode } = memo {
            let current_info = self.instructions[pc as usize];
            self.instructions[pc as usize] = MemoryLocation {
                opcode: Some(opcode),
                label: current_info.label,
            };
            opcode.mnemonic().map(|mnemonic| {
                mnemonic.jump_target(pc).map(|target| {
                    let next_label = self.next_label;
                    let mut target_location = self.instructions[target as usize];
                    if target_location.label.is_none() {
                        target_location.label = Some(next_label);
                        self.next_label += 1;
                        self.instructions[target as usize] = target_location;
                    }
                })
            });
            self.last_pc = pc;
        }

        // if the new memo matches a pattern, hold

        self.recent_memos.push_back(memo);
    }
}

impl Debugger for DebuggingInbox {
    fn query(&self, query: Query) -> String {
        use self::Query::*;

        let result = match query {
            RecentMemos => {
                let mut result = String::new();
                for memo in self.recent_memos.iter() {
                    writeln!(result, "{}", memo).unwrap();
                }
                result
            }
            DisassemblyAt(pc) => self.disassembly_around(pc),
            Disassembly => self.disassembly(None, 0, 0xFFFF),
        };
        result
    }

    fn command(&mut self, command: Command) {
        use self::Command::*;

        match command {
            BreakAtPc(pc) => self.pc_breakpoints.push(pc),
            RemovePcBreakpoints => self.pc_breakpoints = Vec::new(),
            Step => self.status = DebugStatus::Step,
            // BreakAtMemo(pattern) => self.memo_patterns.push(pattern),
            // RemoveBreakMemos => self.memo_patterns = Vec::new(),
        }
    }
}

pub trait GetDebugger {
    fn debugger(&mut self) -> Option<&mut dyn Debugger>;
}

impl GetDebugger for DebuggingInbox {
    fn debugger(&mut self) -> Option<&mut dyn Debugger> {
        Some(self)
    }
}

impl GetDebugger for NothingInbox<Z80Memo> {
    fn debugger(&mut self) -> Option<&mut dyn Debugger> {
        None
    }
}
