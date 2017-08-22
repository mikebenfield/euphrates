// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use ::log;
use ::bits::*;

use super::instructions;
use hardware::z80::types::*;
use hardware::io::Io;

pub fn run<I: Io>(z: &mut Z80<I>, cycles: u64) {
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

    fn read_pc<I: Io>(z: &mut Z80<I>) -> u8 {
        let pc = PC.get(z);
        log_minor!("Z80: PC: {:0>4X}", pc);
        let opcode: u8 = Address(pc).get(z);
        PC.set(z, pc.wrapping_add(1));
        log_minor!("Z80: opcode: {:0>2X}", opcode);
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

    macro_rules! do_instruction {
        (halt, $t_states: expr $(,$arguments: tt)*) => {
            use std;
            log_minor!("Z80: op: halt");
            apply_args!(halt, $($arguments),*);
            z.cycles = std::cmp::max(z.cycles, cycles);
            let pc = PC.get(z);
            PC.set(z, pc.wrapping_sub(1));
            return;
        };
        ($mnemonic: ident, $t_states: expr $(,$arguments: tt)*) => {
            log_minor!("Z80: op: {}", stringify!($mnemonic $($arguments),*));
            apply_args!($mnemonic, $($arguments),*);
            z.cycles += $t_states;
            if z.cycles >= cycles {
                return;
            }
            prefix = Prefix::NoPrefix;
            continue;
        };
    }

    macro_rules! instruction_noprefix {
        ([$code: expr] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ([$code: expr, e] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                e = read_pc(z) as i8;
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ([$code: expr, d] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                d = read_pc(z) as i8;
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ([$code: expr, n] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                n = read_pc(z);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ([$code: expr, n, n] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                let n1: u8 = read_pc(z);
                let n2: u8 = read_pc(z);
                nn = to16(n1, n2);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ($($ignore: tt)*) => {
        };
    }

    macro_rules! instruction_ed {
        ([0xED, $code: expr] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ([0xED, $code: expr, n, n] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                let n1: u8 = read_pc(z);
                let n2: u8 = read_pc(z);
                nn = to16(n1, n2);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ($($ignore: tt)*) => {
        };
    }

    macro_rules! instruction_cb {
        ([0xCB, $code: expr] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ($($ignore: tt)*) => {
        };
    }

    macro_rules! instruction_ddcb {
        ([0xDD, 0xCB, d, $code: expr] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ($($ignore: tt)*) => {
        };
    }

    macro_rules! instruction_fdcb {
        ([0xFD, 0xCB, d, $code: expr] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
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
                nn = to16(n1, n2);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ([0xDD, $code: expr, d, n] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                d = read_pc(z) as i8;
                n = read_pc(z);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ([0xDD, $code: expr, d] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                d = read_pc(z) as i8;
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ([0xDD, $code: expr, n] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                n = read_pc(z);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ([0xDD, $code: expr] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
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
                nn = to16(n1, n2);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ([0xFD, $code: expr, d, n] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                d = read_pc(z) as i8;
                n = read_pc(z);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ([0xFD, $code: expr, d] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                d = read_pc(z) as i8;
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ([0xFD, $code: expr, n] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                n = read_pc(z);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ([0xFD, $code: expr] ; $mnemonic: ident ; [$($arguments: tt),*] ; $t_states: expr ; $is_undoc: expr ) => {
            if opcode == $code {
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ($($ignore: tt)*) => {
        };
    }

    loop {
        match prefix {
            Prefix::NoPrefix => {
                opcode = read_pc(z);
                inc_r(z);
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
                inc_r(z);
                process_instructions!(instruction_ed, d, e, n, nn);
                z.cycles += 8;
                if z.cycles >= cycles {
                    return;
                }
                prefix = Prefix::NoPrefix;
                continue;
            },
            Prefix::Cb => {
                opcode = read_pc(z);
                inc_r(z);
                process_instructions!(instruction_cb, d, e, n, nn);
                z.cycles += 8;
                if z.cycles >= cycles {
                    return;
                }
                prefix = Prefix::NoPrefix;
                continue;
            },
            Prefix::Ddcb => {
                d = read_pc(z) as i8;
                opcode = read_pc(z);
                // inc_r(z);
                process_instructions!(instruction_ddcb, d, e, n, nn);
                panic!("Z80: can't happen: missing opcode DD CB {:0>2X} {:0>2X}", d as u8, opcode);
            },
            Prefix::Fdcb => {
                d = read_pc(z) as i8;
                opcode = read_pc(z);
                // inc_r(z);
                process_instructions!(instruction_fdcb, d, e, n, nn);
                panic!("Z80: can't happen: missing opcode FD CB {:0>2X} {:0>2X}", d as u8, opcode);
            },
            Prefix::Dd => {
                opcode = read_pc(z);
                inc_r(z);
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
                    z.cycles += 4;
                    prefix = Prefix::Dd;
                    continue;
                }
                if opcode == 0xFD {
                    z.cycles += 4;
                    prefix = Prefix::Fd;
                    continue;
                }
                z.cycles += 4;
                if z.cycles >= cycles {
                    return;
                }
                prefix = Prefix::NoPrefix;
                continue;
            },
            Prefix::Fd => {
                opcode = read_pc(z);
                inc_r(z);
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
                    z.cycles += 4;
                    prefix = Prefix::Dd;
                    continue;
                }
                if opcode == 0xFD {
                    z.cycles += 4;
                    prefix = Prefix::Fd;
                    continue;
                }
                z.cycles += 4;
                if z.cycles >= cycles {
                    return;
                }
                prefix = Prefix::NoPrefix;
                continue;
            },
        }
    }
}
