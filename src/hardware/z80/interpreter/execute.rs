#![allow(unused_variables)]
#![allow(dead_code)]

use bits::*;

use hardware::z80::types::*;

macro_rules! read_n {
    ($cpu: ident, $f: ident ($($args: tt)*)) => {
        {
            let pc = PC.get($cpu);
            let n: u8 = Address(pc).get($cpu);
            PC.set($cpu, pc + 1);
            apply!($f, ($cpu, $($args)*), n);
        }
    };
}

macro_rules! shift_d {
    ($cpu: ident, $reg: expr, $d_offset: expr, $pc_addition: expr,
    $f: ident ($($args: tt)*)) => {
        {
            let pc = PC.get($cpu);
            let d_addr = pc.wrapping_add($d_offset);
            let di: u8 = Address(d_addr).get($cpu);
            let d = di as i8;
            PC.set($cpu, pc + $pc_addition);
            let s = Shift($reg, d);
            apply!($f, ($cpu, $($args)*), s);
        }
    };
}

macro_rules! shift_d_read_n {
    ($cpu: ident, $reg: expr, $f: ident ($($args: tt)*)) => {
        {
            let d_addr = PC.get($cpu);
            let di: u8 = Address(d_addr).get($cpu);
            let d = di as i8;
            let s = Shift($reg, d);
            let n: u8 = Address(d_addr.wrapping_add(1)).get($cpu);
            PC.set($cpu, d_addr.wrapping_add(2));
            apply!($f, ($cpu, $($args)*), s, n);
        }
    };
}

macro_rules! read_nn {
    ($cpu: ident, $f: ident ($($args: tt)*)) => {
        {
            let pc = PC.get($cpu);
            let nn: u16 = Address(pc).get($cpu);
            PC.set($cpu, pc + 2);
            apply!($f, ($cpu, $($args)*), nn);
        }
    };
}

macro_rules! read_pnn {
    ($cpu: ident, $f: ident ($($args: tt)*)) => {
        {
            let pc = PC.get($cpu);
            let n1 = Address(pc).get($cpu);
            let n2 = Address(pc + 1).get($cpu);
            let pnn = Address(to16(n1, n2));
            PC.set($cpu, pc + 2);
            apply!($f, ($cpu, $($args)*), pnn);
        }
    };
}

//// Matching
/////////////

macro_rules! apply {
    (@ (), ($f: ident), $args: tt) => {
        $f! $args;
    };

    (@ (), $f: ident, $args: tt) => {
        $f $args;
    };

    (@
     (@ $($processing: tt)*),
     $f: tt,
     ($($arg: tt)*),
     $x: tt $(, $xs: tt)*
    ) => {
        apply!{@
         ($($processing)*),
          $f,
           ($($arg)* $x) $(, $xs)*
        }
    };

    (@
     ($proc1: tt $($processing: tt)*),
     $f: tt,
     ($($arg: tt)*)
     $(, $xs: tt)*
    ) => {
        apply!{@
         ($($processing)*),
          $f,
           ($($arg)* $proc1) $(, $xs)*
        }
    };

    ($f: tt, $args: tt $($rest: tt)*) => {
        apply!{@ $args, $f, () $($rest)*}
    };
}

macro_rules! prefix {
    ($pre: ident, $f: ident ! ($($args: tt)*)) => {
        $f ! ($pre, $($args)*);
    };

    ($pre: ident, $f: ident ($($args: tt)*)) => {
        $f ($pre, $($args)*);
    };
}

macro_rules! operation {
    (@noeq $arg1: ident, $arg2: ident, $op: ident,
    ($($opcode_parameter1: expr),*),
    ($($value_parameter1: expr),*),
    $opcode_parameters2: tt,
    $value_parameters2: tt,
    $opcode: expr, $last: expr) => {
        {
            $(
                operation!(@noeq
                    $arg1, $arg2, $op,
                    $opcode_parameter1, $value_parameter1,
                    $opcode_parameters2, $value_parameters2,
                    $opcode, $last
                );
            )*
        };
    };

    (@noeq $arg1: ident, $arg2: ident, $op: ident,
    $opcode_parameter1: expr,
    $value_parameter1: expr,
    ($($opcode_parameter2: expr),*),
    ($($value_parameter2: expr),*),
    $opcode: expr, $last: expr) => {
        $(
            {
                let $arg1 = $opcode_parameter1;
                let $arg2 = $opcode_parameter2;
                if ($op == $opcode) && ($arg1 != $arg2) {
                    let $arg1 = $value_parameter1;
                    let $arg2 = $value_parameter2;
                    {
                        use super::instructions::*;
                        $last;
                    }
                }
            }
        )*
    };

    (@dual $arg1: ident, $arg2: ident, $op: ident,
    ($($opcode_parameter1: expr),*),
    ($($value_parameter1: expr),*),
    $opcode_parameters2: tt,
    $value_parameters2: tt,
    $opcode: expr, $call: expr) => {
        {
            $(
                operation!(@dual
                    $arg1, $arg2, $op,
                    $opcode_parameter1, $value_parameter1,
                    $opcode_parameters2, $value_parameters2,
                    $opcode, $call
                );
            )*
        };
    };

    (@dual $arg1: ident, $arg2: ident, $op: ident,
    $opcode_parameter1: expr,
    $value_parameter1: expr,
    ($($opcode_parameter2: expr),*),
    ($($value_parameter2: expr),*),
    $opcode: expr, $call: expr) => {
        $(
            {
                let $arg1 = $opcode_parameter1;
                let $arg2 = $opcode_parameter2;
                if $op == $opcode {
                    let $arg1 = $value_parameter1;
                    let $arg2 = $value_parameter2;
                    {
                        use super::instructions::*;
                        $call;
                    }
                }
            }
        )*
    };

    (@ $arg1: ident, $arg2: ident, $op: ident,
    ($($opcode_parameter: expr),*),
    ($($value_parameter: expr),*),
    $opcode: expr, $call: expr) => {
        $(
            {
                let $arg1 = $opcode_parameter;
                if $op == $opcode {
                    let $arg1 = $value_parameter;
                    {
                        use super::instructions::*;
                        $call;
                    }
                }
            }
        )*
    };

    ($arg1: ident, $arg2: ident, $z80: ident, $op: ident,
    r, $opcode: expr, $t_states: expr, $($rest: tt)*) => {
        operation!(@
            $arg1, $arg2, $op,
            (0, 1, 2, 3, 4, 5, 7),
            (B, C, D, E, H, L, A),
            $opcode,
            {
                prefix!($z80, $($rest)*);
                $z80.cycles(& $t_states);
                return;
            }
        );
    };

    ($arg1: ident, $arg2: ident, $z80: ident, $op: ident,
    r r, $opcode: expr, $t_states: expr, $($rest: tt)*) => {
        operation!(@noeq
            $arg1, $arg2, $op,
            (0, 1, 2, 3, 4, 5, 7),
            (B, C, D, E, H, L, A),
            (0, 1, 2, 3, 4, 5, 7),
            (B, C, D, E, H, L, A),
            $opcode,
            {
                prefix!($z80, $($rest)*);
                $z80.cycles(& $t_states);
                return;
            }
        );
    };

    ($arg1: ident, $arg2: ident, $z80: ident, $op: ident,
    p, $opcode: expr, $t_states: expr, $($rest: tt)*) => {
        operation!(@
            $arg1, $arg2, $op,
            (0, 1, 2, 3, 4, 5, 7),
            (B, C, D, E, IXH, IXL, A),
            $opcode,
            {
                prefix!($z80, $($rest)*);
                $z80.cycles(& $t_states);
                return;
            }
        );
    };

    ($arg1: ident, $arg2: ident, $z80: ident, $op: ident,
    p p, $opcode: expr, $t_states: expr, $($rest: tt)*) => {
        operation!(@noeq
            $arg1, $arg2, $op,
            (0, 1, 2, 3, 4, 5, 7),
            (B, C, D, E, IXH, IXL, A),
            (0, 1, 2, 3, 4, 5, 7),
            (B, C, D, E, IXH, IXL, A),
            $opcode,
            {
                prefix!($z80, $($rest)*);
                $z80.cycles(& $t_states);
                return;
            }
        );
    };

    ($arg1: ident, $arg2: ident, $z80: ident, $op: ident,
    q, $opcode: expr, $t_states: expr, $($rest: tt)*) => {
        operation!(@
            $arg1, $arg2, $op,
            (0, 1, 2, 3, 4, 5, 7),
            (B, C, D, E, IYH, IYL, A),
            $opcode,
            {
                prefix!($z80, $($rest)*);
                $z80.cycles(& $t_states);
                return;
            }
        );
    };

    ($arg1: ident, $arg2: ident, $z80: ident, $op: ident,
    q q, $opcode: expr, $t_states: expr, $($rest: tt)*) => {
        operation!(@noeq
            $arg1, $arg2, $op,
            (0, 1, 2, 3, 4, 5, 7),
            (B, C, D, E, IYH, IYL, A),
            (0, 1, 2, 3, 4, 5, 7),
            (B, C, D, E, IYH, IYL, A),
            $opcode,
            {
                prefix!($z80, $($rest)*);
                $z80.cycles(& $t_states);
                return;
            }
        );
    };

    ($arg1: ident, $arg2: ident, $z80: ident, $op: ident,
    b r, $opcode: expr, $t_states: expr, $($rest: tt)*) => {
        operation!(@dual
            $arg1, $arg2, $op,
            (0, 1, 2, 3, 4, 5, 6, 7),
            (0, 1, 2, 3, 4, 5, 6, 7),
            (0, 1, 2, 3, 4, 5, 7),
            (B, C, D, E, H, L, A),
            $opcode,
            {
                prefix!($z80, $($rest)*);
                $z80.cycles(& $t_states);
                return;
            }
        );
    };

    ($arg1: ident, $arg2: ident, $z80: ident, $op: ident,
    b, $opcode: expr, $t_states: expr, $($rest: tt)*) => {
        operation!(@
            $arg1, $arg2, $op,
            (0, 1, 2, 3, 4, 5, 6, 7),
            (0, 1, 2, 3, 4, 5, 6, 7),
            $opcode,
            {
                prefix!($z80, $($rest)*);
                $z80.cycles(& $t_states);
                return;
            }
        );
    };

    ($arg1: ident, $arg2: ident, $z80: ident, $op: ident,
    dd, $opcode: expr, $t_states: expr, $($rest: tt)*) => {
        operation!(@
            $arg1, $arg2, $op,
            (0, 1, 2, 3),
            (BC, DE, HL, SP),
            $opcode,
            {
                prefix!($z80, $($rest)*);
                $z80.cycles(& $t_states);
                return;
            }
        );
    };

    ($arg1: ident, $arg2: ident, $z80: ident, $op: ident,
    qq1, $opcode: expr, $t_states: expr, $($rest: tt)*) => {
        operation!(@
            $arg1, $arg2, $op,
            (0, 1, 2, 3),
            (BC, DE, HL, AF),
            $opcode,
            {
                prefix!($z80, $($rest)*);
                $z80.cycles(& $t_states);
                return;
            }
        );
    };

    ($arg1: ident, $arg2: ident, $z80: ident, $op: ident,
    ss, $opcode: expr, $t_states: expr, $($rest: tt)*) => {
        operation!(@
            $arg1, $arg2, $op,
            (0, 1, 2, 3),
            (BC, DE, HL, SP),
            $opcode,
            {
                prefix!($z80, $($rest)*);
                $z80.cycles(& $t_states);
                return;
            }
        );
    };

    ($arg1: ident, $arg2: ident, $z80: ident, $op: ident,
    pp, $opcode: expr, $t_states: expr, $($rest: tt)*) => {
        operation!(@
            $arg1, $arg2, $op,
            (0, 1, 2, 3),
            (BC, DE, IX, SP),
            $opcode,
            {
                prefix!($z80, $($rest)*);
                $z80.cycles(& $t_states);
                return;
            }
        );
    };

    ($arg1: ident, $arg2: ident, $z80: ident, $op: ident,
    qq2, $opcode: expr, $t_states: expr, $($rest: tt)*) => {
        operation!(@
            $arg1, $arg2, $op,
            (0, 1, 2, 3),
            (BC, DE, IY, SP),
            $opcode,
            {
                prefix!($z80, $($rest)*);
                $z80.cycles(& $t_states);
                return;
            }
        );
    };

    ($arg1: ident, $arg2: ident, $z80: ident, $op: ident,
    $opcode: expr, $t_states: expr, $($rest: tt)*) => {
        if $op == $opcode {
            use super::instructions::*;
            prefix!($z80, $($rest)*);
            $z80.cycles(& $t_states);
            return;
        }
    };
}

macro_rules! operations {
    (@noprefix $arg1: ident, $arg2: ident, $z80: ident, $op: ident,
    0xDD $($rest: tt)*) => {};
    (@noprefix $arg1: ident, $arg2: ident, $z80: ident, $op: ident,
    0xFD $($rest: tt)*) => {};
    (@noprefix $arg1: ident, $arg2: ident, $z80: ident, $op: ident,
    0xED $($rest: tt)*) => {};
    (@noprefix $arg1: ident, $arg2: ident, $z80: ident, $op: ident,
    0xCB $($rest: tt)*) => {};

    (@noprefix $($rest: tt)*) => {
        operation!($($rest)*);
    };

    (@dd $arg1: ident, $arg2: ident, $z80: ident, $op: ident,
    0xDD 0xCB $($rest: tt)*) => { };

    (@dd $arg1: ident, $arg2: ident, $z80: ident, $op: ident,
    0xDD $($rest: tt)*) => {
        operation!($arg1, $arg2, $z80, $op, $($rest)*);
    };

    (@dd $($rest: tt)*) => { };

    (@fd $arg1: ident, $arg2: ident, $z80: ident, $op: ident,
    0xFD 0xCB $($rest: tt)*) => { };

    (@fd $arg1: ident, $arg2: ident, $z80: ident, $op: ident,
    0xFD $($rest: tt)*) => {
        operation!($arg1, $arg2, $z80, $op, $($rest)*);
    };

    (@fd $($rest: tt)*) => { };

    (@ed $arg1: ident, $arg2: ident, $z80: ident, $op: ident,
    0xED $($rest: tt)*) => {
        operation!($arg1, $arg2, $z80, $op, $($rest)*);
    };

    (@ed $($rest: tt)*) => { };

    (@cb $arg1: ident, $arg2: ident, $z80: ident, $op: ident,
    0xCB $($rest: tt)*) => {
        operation!($arg1, $arg2, $z80, $op, $($rest)*);
    };

    (@cb $($rest: tt)*) => { };

    (@dd_cb $arg1: ident, $arg2: ident, $z80: ident, $op: ident,
    0xDD 0xCB $($rest: tt)*) => {
        operation!($arg1, $arg2, $z80, $op, $($rest)*);
    };

    (@dd_cb $($rest: tt)*) => { };

    (@fd_cb $arg1: ident, $arg2: ident, $z80: ident, $op: ident,
    0xFD 0xCB $($rest: tt)*) => {
        operation!($arg1, $arg2, $z80, $op, $($rest)*);
    };

    (@fd_cb $($rest: tt)*) => { };

    ($execute: ident, $arg1: ident, $arg2: ident,
     $([$($more: tt)*])*) => {

        pub fn $execute<Z: Z80>(z: &mut Z, op: u8) {
            if op == 0xDD {
                let newop: u8 = Address(PC).get(z);
                let pc = PC.get(z);
                PC.set(z, pc.wrapping_add(1));
                return fn_dd(z, newop);
            }
            if op == 0xFD {
                let newop: u8 = Address(PC).get(z);
                let pc = PC.get(z);
                PC.set(z, pc.wrapping_add(1));
                return fn_fd(z, newop);
            }
            if op == 0xED {
                let newop: u8 = Address(PC).get(z);
                let pc = PC.get(z);
                PC.set(z, pc.wrapping_add(1));
                return fn_ed(z, newop);
            }
            if op == 0xCB {
                let newop: u8 = Address(PC).get(z);
                let pc = PC.get(z);
                PC.set(z, pc.wrapping_add(1));
                return fn_cb(z, newop);
            }
            log_minor!(z, "Z80: opcode: {:0>2X}", op);

            $(
                operations!(@noprefix $arg1, $arg2,
                 z, op, $($more)*);
            )*

            panic!("Unknown opcode: {:0>2X}", op);
        }

        fn fn_dd<Z: Z80>(z: &mut Z, op: u8) {
            if op == 0xCB {
                let newop: u8 = Address(PC).get(z);
                let pc = PC.get(z);
                PC.set(z, pc.wrapping_add(1));
                return fn_dd_cb(z, newop);
            }
            log_minor!(z, "Z80: opcode: DD {:0>2X}", op);

            $(
                operations!(@dd $arg1, $arg2,
                    z, op, $($more)*);
            )*

            panic!("Unknown opcode: DD {:0>2X}", op);
        }

        fn fn_fd<Z: Z80>(z: &mut Z, op: u8) {
            if op == 0xCB {
                let newop: u8 = Address(PC).get(z);
                let pc = PC.get(z);
                PC.set(z, pc.wrapping_add(1));
                return fn_fd_cb(z, newop);
            }
            log_minor!(z, "Z80: opcode: CB {:0>2X}", op);

            $(
                operations!(@fd $arg1, $arg2,
                    z, op, $($more)*);
            )*

            panic!("Unknown opcode: FD {:0>2X}", op);
        }

        fn fn_ed<Z: Z80>(z: &mut Z, op: u8) {
            log_minor!(z, "Z80: opcode: ED {:0>2X}", op);
            $(
                operations!(@ed $arg1, $arg2,
                    z, op, $($more)*);
            )*

            panic!("Unknown opcode: ED {:0>2X}", op);
        }

        fn fn_cb<Z: Z80>(z: &mut Z, op: u8) {
            log_minor!(z, "Z80: opcode: CB {:0>2X}", op);
            $(
                operations!(@cb $arg1, $arg2,
                    z, op, $($more)*);
            )*

            panic!("Unknown opcode: CB {:0>2X}", op);
        }

        fn fn_dd_cb<Z: Z80>(z: &mut Z, op: u8) {
            let newop: u8 = Address(PC).get(z);
            log_minor!(z, "Z80: opcode: DD CB {:0>2X} {:0>2X}", op, newop);
            let pc = PC.get(z);
            PC.set(z, pc.wrapping_add(1));
            $(
                operations!(@dd_cb $arg1, $arg2,
                    z, newop, $($more)*);
            )*

            panic!("Unknown opcode: DD CB {:0>2X} {:0>2X}", op, newop);
        }

        fn fn_fd_cb<Z: Z80>(z: &mut Z, op: u8) {
            let newop: u8 = Address(PC).get(z);
            log_minor!(z, "Z80: opcode: FD CB {:0>2X} {:0>2X}", op, newop);
            let pc = PC.get(z);
            PC.set(z, pc.wrapping_add(1));
            $(
                operations!(@fd_cb $arg1, $arg2,
                    z, newop, $($more)*);
            )*

            panic!("Unknown opcode: FD CB {:0>2X} {:0>2X}", op, newop);
        }
     };
}

pub fn execute1<Z: Z80>(z: &mut Z) {
    let pc = PC.get(z);
    log_minor!(z, "Z80: PC: {:0>4X}", pc);
    let op: u8 = Address(pc).get(z);
    PC.set(z, pc.wrapping_add(1));
    execute(z, op);
    let s = format!("{}", z.get_z80_hardware());
    log_minor!(z, "Z80: status: {}", s);
}

include!{"../operation_list.rs"}
