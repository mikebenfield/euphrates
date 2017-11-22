// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use ::has::Has;
use ::utilities;
use ::memo::{Inbox, Outbox};
use super::*;

fn receive<Z>(z: &mut Z, memo: Memo)
where
    Z: Inbox<Memo> + Has<Component> + ?Sized
{
    let id = z.get().id();
    z.receive(id, memo);
}

fn run<Z>(z: &mut Z, cycles: u64)
where
    Z: Machine + ?Sized
{
    let mut opcode: u8;
    let mut n: u8;
    let mut nn: u16;
    let mut e: i8;
    let mut d: i8;

    enum Prefix {
        NoPrefix,
        Ed,
        Cb,
        Dd,
        Fd,
        Ddcb,
        Fdcb,
    }

    let mut prefix = Prefix::NoPrefix;

    fn read_pc<Z>(z: &mut Z) -> u8
    where
        Z: Machine + ?Sized
    {
        let pc = PC.view(z);
        let opcode: u8 = Address(pc).view(z);
        PC.change(z, pc.wrapping_add(1));
        opcode
    }

    macro_rules! apply_args {
        (@ ($x: tt + d)) => {
            Shift(apply_args!(@ $x), d)
        };
        (@ ($x: tt)) => {
            Address(apply_args!(@ $x))
        };
        (@ $x: tt) => {
            $x
        };
        ($mnemonic: ident, $($args: tt),*) => {
            instructions::$mnemonic
            (
                z,
                $(
                    apply_args!(@ $args)
                ),*
            )
        };
    }

    macro_rules! check_return {
        () => {
            if z.get().cycles >= cycles || z.wants_pause() {
                return;
            }
        }
    }

    macro_rules! do_instruction {
        (halt, $t_states: expr $(,$arguments: tt)*) => {
            use std;
            apply_args!(halt, $($arguments),*);
            z.get_mut().cycles = std::cmp::max(z.get().cycles, cycles);
            let pc = PC.view(z);
            PC.change(z, pc.wrapping_sub(1));
            return;
        };
        ($mnemonic: ident, $t_states: expr $(,$arguments: tt)*) => {
            apply_args!($mnemonic, $($arguments),*);
            z.get_mut().cycles += $t_states;
            check_return!();
            prefix = Prefix::NoPrefix;
            continue;
        };
    }

    macro_rules! send_instruction {
        ([$code0: expr]) => {
            let pc_op = PC.view(z).wrapping_sub(1);
            receive(
                z,
                Memo::InstructionAtPc(pc_op),
            );
            receive(
                z,
                Memo::InstructionOpcode(Opcode::OneByte([$code0])),
            );
        };
        ([$code0: expr, $code1: expr]) => {
            let pc_op = PC.view(z).wrapping_sub(2);
            receive(
                z,
                Memo::InstructionAtPc(pc_op),
            );
            receive(
                z,
                Memo::InstructionOpcode(Opcode::TwoBytes([$code0, $code1])),
            );
        };
        ([$code0: expr, $code1: expr, $code2: expr]) => {
            let pc_op = PC.view(z).wrapping_sub(3);
            receive(
                z,
                Memo::InstructionAtPc(pc_op),
            );
            receive(
                z,
                Memo::InstructionOpcode(Opcode::ThreeBytes([$code0, $code1, $code2])),
            );
        };
        ([$code0: expr, $code1: expr, $code2: expr, $code3: expr]) => {
            let pc_op = PC.view(z).wrapping_sub(4);
            receive(
                z,
                Memo::InstructionAtPc(pc_op),
            );
            receive(
                z,
                Memo::InstructionOpcode(Opcode::FourBytes([$code0, $code1, $code2, $code3])),
            );
        };
    }

    macro_rules! instruction_noprefix {
        ([$code: expr] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                send_instruction!([$code]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ([$code: expr, e] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                e = read_pc(z) as i8;
                send_instruction!([$code, e as u8]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ([$code: expr, d] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                d = read_pc(z) as i8;
                send_instruction!([$code, d as u8]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ([$code: expr, n] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                n = read_pc(z);
                send_instruction!([$code, n]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ([$code: expr, n, n] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                let n1: u8 = read_pc(z);
                let n2: u8 = read_pc(z);
                send_instruction!([$code, n1, n2]);
                nn = utilities::to16(n1, n2);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ($($ignore: tt)*) => {
        };
    }

    macro_rules! instruction_ed {
        ([0xED, $code: expr] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                send_instruction!([0xED, $code]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ([0xED, $code: expr, n, n] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                let n1: u8 = read_pc(z);
                let n2: u8 = read_pc(z);
                nn = utilities::to16(n1, n2);
                send_instruction!([0xED, $code, n1, n2]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ($($ignore: tt)*) => {
        };
    }

    macro_rules! instruction_cb {
        ([0xCB, $code: expr] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                send_instruction!([0xCB, $code]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ($($ignore: tt)*) => {
        };
    }

    macro_rules! instruction_ddcb {
        ([0xDD, 0xCB, d, $code: expr] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                send_instruction!([0xDD, 0xCB, d as u8, $code]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ($($ignore: tt)*) => {
        };
    }

    macro_rules! instruction_fdcb {
        ([0xFD, 0xCB, d, $code: expr] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                send_instruction!([0xFD, 0xCB, d as u8, $code]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ($($ignore: tt)*) => {
        };
    }

    macro_rules! instruction_dd {
        ([0xDD, $code: expr, n, n] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                let n1: u8 = read_pc(z);
                let n2: u8 = read_pc(z);
                nn = utilities::to16(n1, n2);
                send_instruction!([0xDD, $code, n1, n2]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ([0xDD, $code: expr, d, n] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                d = read_pc(z) as i8;
                n = read_pc(z);
                send_instruction!([0xDD, $code, d as u8, n]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ([0xDD, $code: expr, d] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                d = read_pc(z) as i8;
                send_instruction!([0xDD, $code, d as u8]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ([0xDD, $code: expr, n] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                n = read_pc(z);
                send_instruction!([0xDD, $code, n]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ([0xDD, $code: expr] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                send_instruction!([0xDD, $code]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ($($ignore: tt)*) => {
        };
    }

    macro_rules! instruction_fd {
        ([0xFD, $code: expr, n, n] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                let n1: u8 = read_pc(z);
                let n2: u8 = read_pc(z);
                nn = utilities::to16(n1, n2);
                send_instruction!([0xFD, $code, n1, n2]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ([0xFD, $code: expr, d, n] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                d = read_pc(z) as i8;
                n = read_pc(z);
                send_instruction!([0xFD, $code, d as u8, n]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ([0xFD, $code: expr, d] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                d = read_pc(z) as i8;
                send_instruction!([0xFD, $code, d as u8]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ([0xFD, $code: expr, n] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                n = read_pc(z);
                send_instruction!([0xFD, $code, n]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ([0xFD, $code: expr] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                send_instruction!([0xFD, $code]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ($($ignore: tt)*) => {
        };
    }

    if z.requesting_nmi() {
        instructions::nonmaskable_interrupt(z);
        z.clear_nmi();
    } else {
        match z.requesting_mi() {
            Some(x) => {
                instructions::maskable_interrupt(z, x);
            }
            _ => {}
        };
    }

    loop {
        match prefix {
            Prefix::NoPrefix => {
                {
                    let pc = z.get().get_reg16(PC);
                    receive(
                        z,
                        Memo::ReadingPcToExecute(pc)
                    );
                }
                if z.wants_pause() {
                    return;
                }
                opcode = read_pc(z);
                z.get_mut().inc_r();
                process_instructions!(instruction_noprefix, d, e, n, nn);
                if opcode == 0xED {
                    prefix = Prefix::Ed;
                    continue;
                }
                if opcode == 0xCB {
                    prefix = Prefix::Cb;
                    continue;
                }
                if opcode == 0xDD {
                    prefix = Prefix::Dd;
                    continue;
                }
                if opcode == 0xFD {
                    prefix = Prefix::Fd;
                    continue;
                }
                panic!("Z80: can't happen: missing opcode {:0>2X}", opcode);
            },
            Prefix::Ed => {
                opcode = read_pc(z);
                z.get_mut().inc_r();
                process_instructions!(instruction_ed, d, e, n, nn);
                z.get_mut().cycles += 8;
                check_return!();
                prefix = Prefix::NoPrefix;
                continue;
            },
            Prefix::Cb => {
                opcode = read_pc(z);
                z.get_mut().inc_r();
                process_instructions!(instruction_cb, d, e, n, nn);
                z.get_mut().cycles += 8;
                check_return!();
                prefix = Prefix::NoPrefix;
                continue;
            },
            Prefix::Ddcb => {
                d = read_pc(z) as i8;
                opcode = read_pc(z);
                // z.get_mut().inc_r();
                process_instructions!(instruction_ddcb, d, e, n, nn);
                panic!("Z80: can't happen: missing opcode DD CB {:0>2X} {:0>2X}", d as u8, opcode);
            },
            Prefix::Fdcb => {
                d = read_pc(z) as i8;
                opcode = read_pc(z);
                // z.get_mut().inc_r();
                process_instructions!(instruction_fdcb, d, e, n, nn);
                panic!("Z80: can't happen: missing opcode FD CB {:0>2X} {:0>2X}", d as u8, opcode);
            },
            Prefix::Dd => {
                opcode = read_pc(z);
                z.get_mut().inc_r();
                process_instructions!(instruction_dd, d, e, n, nn);
                if opcode == 0xED {
                    prefix = Prefix::Ed;
                    continue;
                }
                if opcode == 0xCB {
                    prefix = Prefix::Ddcb;
                    continue;
                }
                if opcode == 0xDD {
                    z.get_mut().cycles += 4;
                    prefix = Prefix::Dd;
                    continue;
                }
                if opcode == 0xFD {
                    z.get_mut().cycles += 4;
                    prefix = Prefix::Fd;
                    continue;
                }
                z.get_mut().cycles += 4;
                if z.get().cycles >= cycles {
                    z.get_mut().iff1 = z.get().cycles;
                    return;
                }
                prefix = Prefix::NoPrefix;
                continue;
            },
            Prefix::Fd => {
                opcode = read_pc(z);
                z.get_mut().inc_r();
                process_instructions!(instruction_fd, d, e, n, nn);
                if opcode == 0xED {
                    prefix = Prefix::Ed;
                    continue;
                }
                if opcode == 0xCB {
                    prefix = Prefix::Fdcb;
                    continue;
                }
                if opcode == 0xDD {
                    z.get_mut().cycles += 4;
                    prefix = Prefix::Dd;
                    continue;
                }
                if opcode == 0xFD {
                    z.get_mut().cycles += 4;
                    prefix = Prefix::Fd;
                    continue;
                }
                z.get_mut().cycles += 4;
                if z.get().cycles >= cycles {
                    z.get_mut().iff1 = z.get().cycles;
                    return;
                }
                prefix = Prefix::NoPrefix;
                continue;
            },
        }
    }

}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Interpreter;

impl<Z> Emulator<Z> for Interpreter
where
    Z: Machine + ?Sized
{
    fn run(&mut self, z: &mut Z, cycles: u64)
    {
        run(z, cycles);
    }
}
