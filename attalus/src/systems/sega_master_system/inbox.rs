use std::collections::VecDeque;
use std::fmt::Write;
use std::time::Instant;

use hardware::z80::memo::{Opcode, TargetMnemonic};
use memo::{HoldableImpler, InboxImpler, Memo};

use super::emulator::TimeStatus;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Query {
    /// Show disassembly around this PC, with arrows pointing at this instruction
    DisassemblyAt(u16),
    /// Whole program disassembly
    Disassembly,
    /// Show the last few memos received
    RecentMemos,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QueryResult {
    Ok(String),
    Unsupported,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Command {
    Hold,
    Resume,
    Step,
    BreakAtPc(u16),
    RemovePcBreakpoints,
    // BreakAtMemo(MemoPattern),
    // RemoveBreakMemos,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum CommandResult {
    Ok,
    Unsupported,
}

pub trait Debugger {
    fn command(&mut self, command: Command) -> CommandResult;
    fn query(&self, query: Query) -> QueryResult;
}

pub trait DebuggerImpler<S: ?Sized> {
    fn command(&mut S, command: Command) -> CommandResult;
    fn query(&S, query: Query) -> QueryResult;
}

pub trait DebuggerImpl {
    type Impler: DebuggerImpler<Self>;
}

impl<S> Debugger for S
where
    S: DebuggerImpl + ?Sized,
{
    #[inline]
    fn command(&mut self, command: Command) -> CommandResult {
        <S::Impler as DebuggerImpler<Self>>::command(self, command)
    }

    #[inline]
    fn query(&self, query: Query) -> QueryResult {
        <S::Impler as DebuggerImpler<Self>>::query(self, query)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct HoldingDebugger;

impl HoldingDebugger {
    pub fn new() -> HoldingDebugger {
        HoldingDebugger
    }
}

impl<S> HoldableImpler<S> for HoldingDebugger
where
    S: ?Sized + AsRef<TimeStatus>,
{
    #[inline]
    fn holding(s: &S) -> bool {
        s.as_ref().hold.is_some()
    }
}

impl<S> InboxImpler<S> for HoldingDebugger
where
    S: ?Sized,
{
    #[inline]
    fn receive(_s: &mut S, _memo: Memo) {}
}

impl<S> DebuggerImpler<S> for HoldingDebugger
where
    S: ?Sized
        + AsRef<HoldingDebugger>
        + AsMut<HoldingDebugger>
        + AsRef<TimeStatus>
        + AsMut<TimeStatus>,
{
    #[inline]
    fn command(s: &mut S, command: Command) -> CommandResult {
        use self::Command::*;
        let time_status = AsMut::<TimeStatus>::as_mut(s);
        match (command, time_status.hold) {
            (Hold, None) => time_status.hold = Some(Instant::now()),
            (Resume, Some(instant)) => {
                time_status.hold = None;
                let elapsed = Instant::now().duration_since(instant);
                time_status.hold_duration += elapsed;
            }
            (Hold, _) => {}
            _ => return CommandResult::Unsupported,
        }
        return CommandResult::Ok;
    }

    #[inline]
    fn query(_s: &S, _query: Query) -> QueryResult {
        QueryResult::Unsupported
    }
}

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
    recent_memos: VecDeque<Memo>,
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
                    return Some(pc.wrapping_sub(i))
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

impl<S> InboxImpler<S> for DebuggingInbox
where
    S: ?Sized + AsMut<DebuggingInbox> + AsRef<DebuggingInbox>,
{
    fn receive(s: &mut S, memo: Memo) {
        use hardware::z80::memo::manifests::INSTRUCTION;
        use memo::Payload;
        use std::mem::transmute;

        if s.as_ref().recent_memos.len() >= MAX_MEMOS {
            s.as_mut().recent_memos.pop_front();
        }

        if memo.has_manifest(INSTRUCTION) {
            let payload = match memo.payload() {
                Payload::U8(x) => x,
                _ => unreachable!("INSTRUCTION payload not of U8 type?"),
            };
            let pc_array: [u8; 2] = [payload[0], payload[1]];
            let pc: u16 = unsafe { transmute(pc_array) };
            let opcode = Opcode::from_payload(payload);
            let current_info = s.as_ref().instructions[pc as usize];
            s.as_mut().instructions[pc as usize] = MemoryLocation {
                opcode: Some(opcode),
                label: current_info.label,
            };
            opcode.mnemonic().map(|mnemonic| {
                mnemonic.jump_target(pc).map(|target| {
                    let next_label = s.as_ref().next_label;
                    let mut target_location = s.as_mut().instructions[target as usize];
                    if target_location.label.is_none() {
                        target_location.label = Some(next_label);
                        s.as_mut().next_label += 1;
                        s.as_mut().instructions[target as usize] = target_location;
                    }
                })
            });
            s.as_mut().last_pc = pc;
        }

        // if the new memo matches a pattern, hold

        s.as_mut().recent_memos.push_back(memo);
    }
}

impl<S> HoldableImpler<S> for DebuggingInbox
where
    S: ?Sized + AsRef<TimeStatus>,
{
    #[inline]
    fn holding(s: &S) -> bool {
        s.as_ref().hold.is_some()
    }
}

impl<S> DebuggerImpler<S> for DebuggingInbox
where
    S: ?Sized
        + AsRef<DebuggingInbox>
        + AsMut<DebuggingInbox>
        + AsRef<TimeStatus>
        + AsMut<TimeStatus>,
{
    fn query(s: &S, query: Query) -> QueryResult {
        use self::Query::*;
        let result = match query {
            RecentMemos => {
                let mut result = String::new();
                for memo in AsRef::<DebuggingInbox>::as_ref(s).recent_memos.iter() {
                    writeln!(result, "{}", memo).unwrap();
                }
                result
            }
            DisassemblyAt(pc) => AsRef::<DebuggingInbox>::as_ref(s).disassembly_around(pc),
            Disassembly => AsRef::<DebuggingInbox>::as_ref(s).disassembly(None, 0, 0xFFFF),
        };
        QueryResult::Ok(result)
    }

    fn command(s: &mut S, command: Command) -> CommandResult {
        use self::Command::*;

        match (command, AsRef::<TimeStatus>::as_ref(s).hold) {
            (Hold, None) => AsMut::<TimeStatus>::as_mut(s).hold = Some(Instant::now()),
            (Resume, Some(instant)) => {
                AsMut::<TimeStatus>::as_mut(s).hold = None;
                let elapsed = Instant::now().duration_since(instant);
                AsMut::<TimeStatus>::as_mut(s).hold_duration += elapsed;
            }
            (BreakAtPc(pc), _) => AsMut::<DebuggingInbox>::as_mut(s).pc_breakpoints.push(pc),
            (RemovePcBreakpoints, _) => {
                AsMut::<DebuggingInbox>::as_mut(s).pc_breakpoints = Vec::new()
            }
            (Step, _) => AsMut::<DebuggingInbox>::as_mut(s).status = DebugStatus::Step,
            // BreakAtMemo(pattern) => self.memo_patterns.push(pattern),
            // RemoveBreakMemos => self.memo_patterns = Vec::new(),
            _ => {}
        }

        CommandResult::Ok
    }
}
