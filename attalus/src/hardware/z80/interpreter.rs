use std::mem::transmute;

use hardware::io16::Io16;
use hardware::memory16::Memory16;
use memo::{Inbox, Payload};
use utilities;

use super::memo::manifests;
use super::*;

use self::ConditionCode::*;
use self::Reg16::*;
use self::Reg8::*;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Z80Interpreter;

fn check_interrupts<Z>(z: &mut Z)
where
    Z: Z80Irq + Z80Internal + Memory16 + Inbox + ?Sized,
{
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
}

fn run2<Z>(z: &mut Z, cycles: u64)
where
    Z: Z80Internal
        + Z80Irq
        + Inbox
        + Io16
        + Memory16
        + ?Sized,
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
        Z: Z80Internal + Memory16 + ?Sized,
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
            if z.cycles() >= cycles {
                // XXX || z.holding() {
                return;
            }
        };
    }

    macro_rules! do_instruction {
        // the halt instruction needs extra support, because we need to return early
        (halt, $t_states: expr $(,$arguments: tt)*) => {
            use std;
            apply_args!(halt, $($arguments),*);
            let new_cycles = std::cmp::max(z.cycles(), cycles);
            z.set_cycles(new_cycles);
            let pc = PC.view(z);
            PC.change(z, pc.wrapping_sub(1));
            return;
        };
        // the ei instruction also needs support, because we need to execute one
        // more instruction and then set `iff1` to `true`.
        // We also need to check for multiple eis in a row.
        (ei, $t_states: expr $(,$arguments: tt)*) => {
            apply_args!(ei, $($arguments),*);
            z.inc_cycles($t_states);
            let mut current_pc = z.reg16(PC);
            let initial_pc = current_pc.wrapping_sub(1);
            while z.read(current_pc) == 0xFB {
                current_pc = current_pc.wrapping_add(1);
                z.inc_cycles($t_states);
                z.inc_r(1);
                if current_pc == initial_pc {
                    unimplemented!();
                }
            }
            z.set_reg16(PC, current_pc);
            let current_cycles = z.cycles();
            // XXX - Check for error return
            run2(z, current_cycles + 1);
            z.set_iff1(true);
            check_interrupts(z);
            check_return!();
            prefix = Prefix::NoPrefix;
            continue;
        };
        ($mnemonic: ident, $t_states: expr $(,$arguments: tt)*) => {
            apply_args!($mnemonic, $($arguments),*);
            z.inc_cycles($t_states);
            check_return!();
            prefix = Prefix::NoPrefix;
            continue;
        };
    }

    macro_rules! send_instruction {
        ([$code0:expr]) => {
            let pc_op = PC.view(z).wrapping_sub(1);
            let pc_array: [u8; 2] = unsafe { transmute(pc_op) };
            manifests::INSTRUCTION.send(
                z,
                Payload::U8([pc_array[0], pc_array[1], 1, $code0, 0, 0, 0, 0]),
            );
        };
        ([$code0:expr, $code1:expr]) => {
            let pc_op = PC.view(z).wrapping_sub(2);
            let pc_array: [u8; 2] = unsafe { transmute(pc_op) };
            manifests::INSTRUCTION.send(
                z,
                Payload::U8([pc_array[0], pc_array[1], 2, $code0, $code1, 0, 0, 0]),
            );
        };
        ([$code0:expr, $code1:expr, $code2:expr]) => {
            let pc_op = PC.view(z).wrapping_sub(3);
            let pc_array: [u8; 2] = unsafe { transmute(pc_op) };
            manifests::INSTRUCTION.send(
                z,
                Payload::U8([pc_array[0], pc_array[1], 3, $code0, $code1, $code2, 0, 0]),
            );
        };
        ([$code0:expr, $code1:expr, $code2:expr, $code3:expr]) => {
            let pc_op = PC.view(z).wrapping_sub(4);
            let pc_array: [u8; 2] = unsafe { transmute(pc_op) };
            manifests::INSTRUCTION.send(
                z,
                Payload::U8([
                    pc_array[0],
                    pc_array[1],
                    4,
                    $code0,
                    $code1,
                    $code2,
                    $code3,
                    0,
                ]),
            );
        };
    }

    macro_rules! instruction_noprefix {
        (
            [$code: expr] ;
            $mnemonic: ident ;
            [$($arguments: tt),*] ;
            $t_states: expr ;
            $is_undoc: expr
        ) => {
            if opcode == $code {
                send_instruction!([$code]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        (
            [$code: expr, e] ;
            $mnemonic: ident ;
            [$($arguments: tt),*] ;
            $t_states: expr ;
            $is_undoc: expr
        ) => {
            if opcode == $code {
                e = read_pc(z) as i8;
                send_instruction!([$code, e as u8]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        (
            [$code: expr, d] ;
            $mnemonic: ident ;
            [$($arguments: tt),*] ;
            $t_states: expr ;
            $is_undoc: expr
        ) => {
            if opcode == $code {
                d = read_pc(z) as i8;
                send_instruction!([$code, d as u8]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        (
            [$code: expr, n] ;
            $mnemonic: ident ;
            [$($arguments: tt),*] ;
            $t_states: expr ;
            $is_undoc: expr
        ) => {
            if opcode == $code {
                n = read_pc(z);
                send_instruction!([$code, n]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        (
            [$code: expr, n, n] ;
            $mnemonic: ident ;
            [$($arguments: tt),*] ;
            $t_states: expr ;
            $is_undoc: expr
        ) => {
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
        (
            [0xED, $code: expr] ;
            $mnemonic: ident ;
            [$($arguments: tt),*] ;
            $t_states: expr ;
            $is_undoc: expr
        ) => {
            if opcode == $code {
                send_instruction!([0xED, $code]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        (
            [0xED, $code: expr, n, n] ;
            $mnemonic: ident ;
            [$($arguments: tt),*] ;
            $t_states: expr ;
            $is_undoc: expr
        ) => {
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
        (
            [0xCB, $code: expr] ;
            $mnemonic: ident ;
            [$($arguments: tt),*] ;
            $t_states: expr ;
            $is_undoc: expr
        ) => {
            if opcode == $code {
                send_instruction!([0xCB, $code]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ($($ignore: tt)*) => {
        };
    }

    macro_rules! instruction_ddcb {
        (
            [0xDD, 0xCB, d, $code: expr] ;
            $mnemonic: ident ;
            [$($arguments: tt),*] ;
            $t_states: expr ;
            $is_undoc: expr
        ) => {
            if opcode == $code {
                send_instruction!([0xDD, 0xCB, d as u8, $code]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ($($ignore: tt)*) => {
        };
    }

    macro_rules! instruction_fdcb {
        (
            [0xFD, 0xCB, d, $code: expr] ;
            $mnemonic: ident ;
            [$($arguments: tt),*] ;
            $t_states: expr ;
            $is_undoc: expr
        ) => {
            if opcode == $code {
                send_instruction!([0xFD, 0xCB, d as u8, $code]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ($($ignore: tt)*) => {
        };
    }

    macro_rules! instruction_dd {
        (
            [0xDD, $code: expr, n, n] ;
            $mnemonic: ident ;
            [$($arguments: tt),*] ;
            $t_states: expr ;
            $is_undoc: expr
        ) => {
            if opcode == $code {
                let n1: u8 = read_pc(z);
                let n2: u8 = read_pc(z);
                nn = utilities::to16(n1, n2);
                send_instruction!([0xDD, $code, n1, n2]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        (
            [0xDD, $code: expr, d, n] ;
            $mnemonic: ident ;
            [$($arguments: tt),*] ;
            $t_states: expr ;
            $is_undoc: expr
        ) => {
            if opcode == $code {
                d = read_pc(z) as i8;
                n = read_pc(z);
                send_instruction!([0xDD, $code, d as u8, n]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        (
            [0xDD, $code: expr, d] ;
            $mnemonic: ident ;
            [$($arguments: tt),*] ;
            $t_states: expr ;
            $is_undoc: expr
        ) => {
            if opcode == $code {
                d = read_pc(z) as i8;
                send_instruction!([0xDD, $code, d as u8]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        (
            [0xDD, $code: expr, n] ;
            $mnemonic: ident ;
            [$($arguments: tt),*] ;
            $t_states: expr ;
            $is_undoc: expr
        ) => {
            if opcode == $code {
                n = read_pc(z);
                send_instruction!([0xDD, $code, n]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        (
            [0xDD, $code: expr] ;
            $mnemonic: ident ;
            [$($arguments: tt),*] ;
            $t_states: expr ;
            $is_undoc: expr
        ) => {
            if opcode == $code {
                send_instruction!([0xDD, $code]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ($($ignore: tt)*) => {
        };
    }

    macro_rules! instruction_fd {
        (
            [0xFD, $code: expr, n, n] ;
            $mnemonic: ident ;
            [$($arguments: tt),*] ;
            $t_states: expr ;
            $is_undoc: expr
        ) => {
            if opcode == $code {
                let n1: u8 = read_pc(z);
                let n2: u8 = read_pc(z);
                nn = utilities::to16(n1, n2);
                send_instruction!([0xFD, $code, n1, n2]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        (
            [0xFD, $code: expr, d, n] ;
            $mnemonic: ident ;
            [$($arguments: tt),*] ;
            $t_states: expr ;
            $is_undoc: expr
        ) => {
            if opcode == $code {
                d = read_pc(z) as i8;
                n = read_pc(z);
                send_instruction!([0xFD, $code, d as u8, n]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        (
            [0xFD, $code: expr, d] ;
            $mnemonic: ident ;
            [$($arguments: tt),*] ;
            $t_states: expr ;
            $is_undoc: expr
        ) => {
            if opcode == $code {
                d = read_pc(z) as i8;
                send_instruction!([0xFD, $code, d as u8]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        (
            [0xFD, $code: expr, n] ;
            $mnemonic: ident ;
            [$($arguments: tt),*] ;
            $t_states: expr ;
            $is_undoc: expr
        ) => {
            if opcode == $code {
                n = read_pc(z);
                send_instruction!([0xFD, $code, n]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        (
            [0xFD, $code: expr] ;
            $mnemonic: ident ;
            [$($arguments: tt),*] ;
            $t_states: expr ;
            $is_undoc: expr
        ) => {
            if opcode == $code {
                send_instruction!([0xFD, $code]);
                do_instruction!($mnemonic, $t_states $(,$arguments)*);
            }
        };
        ($($ignore: tt)*) => {
        };
    }

    check_interrupts(z);

    loop {
        match prefix {
            Prefix::NoPrefix => {
                opcode = read_pc(z);
                z.inc_r(1);
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
            }
            Prefix::Ed => {
                opcode = read_pc(z);
                z.inc_r(1);
                process_instructions!(instruction_ed, d, e, n, nn);
                z.inc_cycles(6);
                check_return!();
                prefix = Prefix::NoPrefix;
                continue;
            }
            Prefix::Cb => {
                opcode = read_pc(z);
                z.inc_r(1);
                process_instructions!(instruction_cb, d, e, n, nn);
                z.inc_cycles(8);
                check_return!();
                prefix = Prefix::NoPrefix;
                continue;
            }
            Prefix::Ddcb => {
                d = read_pc(z) as i8;
                opcode = read_pc(z);
                // z.as_mut().inc_r();
                process_instructions!(instruction_ddcb, d, e, n, nn);
                panic!(
                    "Z80: can't happen: missing opcode DD CB {:0>2X} {:0>2X}",
                    d as u8, opcode
                );
            }
            Prefix::Fdcb => {
                d = read_pc(z) as i8;
                opcode = read_pc(z);
                // z.as_mut().inc_r();
                process_instructions!(instruction_fdcb, d, e, n, nn);
                panic!(
                    "Z80: can't happen: missing opcode FD CB {:0>2X} {:0>2X}",
                    d as u8, opcode
                );
            }
            Prefix::Dd => {
                opcode = read_pc(z);
                z.inc_r(1);
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
                    z.inc_cycles(4);
                    prefix = Prefix::Dd;
                    continue;
                }
                if opcode == 0xFD {
                    z.inc_cycles(4);
                    prefix = Prefix::Fd;
                    continue;
                }
                z.inc_cycles(4);
                if z.cycles() >= cycles {
                    return;
                }
                prefix = Prefix::NoPrefix;
                continue;
            }
            Prefix::Fd => {
                opcode = read_pc(z);
                z.inc_r(1);
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
                    z.inc_cycles(4);
                    prefix = Prefix::Dd;
                    continue;
                }
                if opcode == 0xFD {
                    z.inc_cycles(4);
                    prefix = Prefix::Fd;
                    continue;
                }
                z.inc_cycles(4);
                if z.cycles() >= cycles {
                    return;
                }
                prefix = Prefix::NoPrefix;
                continue;
            }
        }
    }
}

impl<S> Z80Impler<S> for Z80Interpreter
where
    S: Z80Internal
        + Z80Irq
        + Io16
        + Memory16
        + Inbox
        + ?Sized,
{
    #[inline]
    fn run(s: &mut S, cycles: u64) {
        run2(s, cycles)
    }
}
