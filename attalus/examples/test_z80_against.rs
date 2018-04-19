//! test_z80_against: Test the z80 interpreter in `attalus` against the emulator
//! `z80sim`. Requires that `z80sim` be in your $PATH. Will test many instruction
//! sequences with several different starting Z80 states. `z80sim` doesn't
//! implement most undocumented opcodes, so we don't test undocumented opcodes.
//! We also don't test opcodes like `lddr` that are likely to overwrite our own
//! instructions. It is still in principle possible that our instructions could be
//! overwritten by the various ld instructions with undocumented opcodes, and
//! that could lead to different results between `z80sim` and `attalus`. In my
//! light testing that seems not to happen.
//!
//! Note that `z80sim` has a couple relevant bugs:
//! 1. A buffer overflow in instructions with a parameter of the form (IX+d) or
//! (IY+d).
//! 2. The duplicate (but documented) instructions LD HL, (nn) and LD (nn), HL
//! are not implemented.
//!
//! Udo Monk has a modified header file fixing the first bug, but has not
//! publicly released a version of `z80pack` with the fix. Right now you will
//! certainly get failed test results if you run `test_z80_against` on a `z80sim`
//! without the fixed header. If he doesn't release a fixed version soon, maybe I
//! will distribute it.
//!
//! He will likely fix the second bug soon, but for now I am just not testing the
//! relevant instructions.
//!
//! This is in `examples/` rather than as a regular test, at least for now,
//! because:
//! 1. I want tests to be able to be run without `z80sim` installed.
//! 2. `test_z80_against` can take a while to run.
//!
//! Run `test_z80_against --help` (or, more likely, `cargo run --example
//! test_z80_against -- --help`) for usage instructions.
//!
//! Possible future enhancements:
//! 1. Adjust initial value for IX, IY, HL, and SP (?), together with d and e
//! bytes in opcodes, to ensure instructions are never overwritten.
//! 2. Test against other emulators.
//! 3. A big one: test against actual hardware.

extern crate failure;
extern crate rand;
extern crate tempdir;

#[macro_use]
extern crate attalus;

use std::convert::{AsMut, AsRef};
use std::env::args;
use std::fmt;
use std::mem;
use std::os::raw::{c_int, c_long};
use std::path::Path;
use std::process::exit;
use std::str::FromStr;

use failure::Error;

use rand::{Rng, SeedableRng};

use attalus::hardware::io_16_8::Io16_8;
use attalus::hardware::memory_16_8::Memory16Impl;
use attalus::hardware::z80::Reg16::*;
use attalus::hardware::z80::Reg8::*;
use attalus::hardware::z80::{self, Changeable, Safe, Z80, Z80Impl, Z80Internal,
                             Z80InternalImpl, Z80Interpreter, Z80Irq, Z80State};
use attalus::memo::{Holdable, Inbox, Memo};

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
struct Z80System {
    memory: [u8; 0x10000],
    z80: Z80State,
    interpreter: Z80Interpreter<Safe>,
}

impl Default for Z80System {
    fn default() -> Self {
        Z80System {
            memory: [0u8; 0x10000],
            z80: Default::default(),
            interpreter: Default::default(),
        }
    }
}

impl Io16_8 for Z80System {
    fn input(&mut self, _address: u16) -> u8 {
        0
    }
    fn output(&mut self, _address: u16, _value: u8) {}
}

impl fmt::Display for Z80System {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.z80, f)
    }
}

impl AsRef<Z80State> for Z80System {
    fn as_ref(&self) -> &z80::Z80State {
        &self.z80
    }
}

impl AsMut<Z80State> for Z80System {
    fn as_mut(&mut self) -> &mut z80::Z80State {
        &mut self.z80
    }
}

impl Z80InternalImpl for Z80System {
    type Impler = Z80State;
}

impl AsRef<[u8; 0x10000]> for Z80System {
    #[inline]
    fn as_ref(&self) -> &[u8; 0x10000] {
        &self.memory
    }
}

impl AsMut<[u8; 0x10000]> for Z80System {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8; 0x10000] {
        &mut self.memory
    }
}

impl Memory16Impl for Z80System {
    type Impler = [u8; 0x10000];
}

impl Holdable for Z80System {}

impl Inbox for Z80System {
    fn receive(&mut self, _memo: Memo) {}
}

impl Z80Irq for Z80System {
    fn requesting_mi(&self) -> Option<u8> {
        None
    }

    fn requesting_nmi(&self) -> bool {
        false
    }

    fn clear_nmi(&mut self) {}
}

impl AsRef<Z80Interpreter<Safe>> for Z80System {
    fn as_ref(&self) -> &Z80Interpreter<Safe> {
        &self.interpreter
    }
}

impl AsMut<Z80Interpreter<Safe>> for Z80System {
    fn as_mut(&mut self) -> &mut Z80Interpreter<Safe> {
        &mut self.interpreter
    }
}

impl Z80Impl for Z80System {
    type Impler = Z80Interpreter<Safe>;
}

fn random_system<R>(mem_start: &[u8], rng: &mut R) -> Z80System
where
    R: Rng,
{
    // 0x76: HALT
    let mut mem = [0x76u8; 0x10000];
    for i in 0x800..0x10000 {
        mem[i] = rng.gen();
    }
    mem[0..mem_start.len()].copy_from_slice(mem_start);

    let mut z80 = Z80System {
        z80: Default::default(),
        memory: mem,
        interpreter: Default::default(),
    };

    for reg in [
        B, C, D, E, A, H, L, B0, C0, D0, E0, A0, L0, H0, IXH, IXL, IYH, IYL,
    ].iter()
    {
        z80.set_reg8(*reg, rng.gen());
    }

    z80
}

fn write_core<P: AsRef<Path>>(path: P, z80: &Z80System) -> Result<()> {
    use std::fs::File;
    use std::io::Write;

    let z80_size: usize = 16 + 2 * 4 + 2 * mem::size_of::<c_int>() + mem::size_of::<c_long>();

    let mut buf: Vec<u8> = Vec::with_capacity(z80_size);

    fn write_from<T>(val: T, buf: &mut Vec<u8>) {
        let size = mem::size_of::<T>() as isize;
        unsafe {
            let valp: *const u8 = std::mem::transmute(&val);
            for j in 0..size {
                buf.push(*valp.offset(j));
            }
        }
    }

    write_from(z80.reg8(A), &mut buf);
    write_from(z80.reg8(F) as c_int, &mut buf);
    write_from(z80.reg8(B), &mut buf);
    write_from(z80.reg8(C), &mut buf);
    write_from(z80.reg8(D), &mut buf);
    write_from(z80.reg8(E), &mut buf);
    write_from(z80.reg8(H), &mut buf);
    write_from(z80.reg8(L), &mut buf);

    write_from(z80.reg8(A0), &mut buf);
    write_from(z80.reg8(F0) as c_int, &mut buf);
    write_from(z80.reg8(B0), &mut buf);
    write_from(z80.reg8(C0), &mut buf);
    write_from(z80.reg8(D0), &mut buf);
    write_from(z80.reg8(E0), &mut buf);
    write_from(z80.reg8(H0), &mut buf);
    write_from(z80.reg8(L0), &mut buf);

    write_from(z80.reg8(I), &mut buf);

    let iff1: u8 = if z80.iff1() { 1 } else { 0 };
    let iff2: u8 = if z80.iff2() { 2 } else { 0 };
    write_from(iff1 | iff2, &mut buf);

    write_from(z80.reg8(R) as c_long, &mut buf);

    write_from(z80.reg16(PC), &mut buf);
    write_from(z80.reg16(SP), &mut buf);
    write_from(z80.reg16(IX), &mut buf);
    write_from(z80.reg16(IY), &mut buf);

    let mut f = File::create(path)?;

    f.write_all(&buf[..])?;
    f.write_all(&z80.memory)?;
    Ok(())
}

fn read_core<P>(path: P) -> Result<Z80System>
where
    P: AsRef<Path>,
{
    use std::fs::File;
    use std::io::Read;

    // z80sim uses a byte for each of A, B, C, D, E, H, L, (and the shadows) IFF, and I
    // it uses 2 bytes for each of IX, IY, PC, and SP.
    // it uses an int for F and F0, and a long for R.
    // it seems the multi-byte registers are in the native byte order
    // (this should be a global const, but can't be due to call of
    // size_of)
    let correct_core_size: usize =
        0x10000 + 16 + 2 * 4 + 2 * mem::size_of::<c_int>() + mem::size_of::<c_long>();

    let mut z80: Z80System = Default::default();

    let mut buf: Vec<u8> = Vec::new();
    {
        let mut f = File::open(path)?;
        f.read_to_end(&mut buf)?;
    }

    if buf.len() != correct_core_size {
        return Err(failure::err_msg(format!(
            "Core file of wrong length {} (should be {})",
            buf.len(),
            correct_core_size
        )));
    }

    let mut i: usize = 0;

    fn read_into<T>(val: &mut T, i: &mut usize, buf: &mut [u8]) {
        let size = mem::size_of::<T>() as isize;
        unsafe {
            let valp: *mut u8 = mem::transmute(val);
            for j in 0..size {
                *valp.offset(j) = buf[*i];
                *i += 1;
            }
        }
    }

    fn read_register<R, T>(z: &mut Z80System, reg: R, i: &mut usize, buf: &mut [u8])
    where
        R: Changeable<T, Z80System>,
        T: Default,
    {
        let mut t: T = Default::default();
        read_into(&mut t, i, buf);
        reg.change(z, t);
    }

    read_register(&mut z80, A, &mut i, &mut buf);
    let mut ff: c_int = 0;
    read_into(&mut ff, &mut i, &mut buf);
    z80.set_reg8(F, ff as u8);
    read_register(&mut z80, B, &mut i, &mut buf);
    read_register(&mut z80, C, &mut i, &mut buf);
    read_register(&mut z80, D, &mut i, &mut buf);
    read_register(&mut z80, E, &mut i, &mut buf);
    read_register(&mut z80, H, &mut i, &mut buf);
    read_register(&mut z80, L, &mut i, &mut buf);

    read_register(&mut z80, A0, &mut i, &mut buf);
    let mut ff0: c_int = 0;
    read_into(&mut ff0, &mut i, &mut buf);
    z80.set_reg8(F0, ff0 as u8);
    read_register(&mut z80, B0, &mut i, &mut buf);
    read_register(&mut z80, C0, &mut i, &mut buf);
    read_register(&mut z80, D0, &mut i, &mut buf);
    read_register(&mut z80, E0, &mut i, &mut buf);
    read_register(&mut z80, H0, &mut i, &mut buf);
    read_register(&mut z80, L0, &mut i, &mut buf);

    read_register(&mut z80, I, &mut i, &mut buf);

    let mut iff: u8 = 0;
    read_into(&mut iff, &mut i, &mut buf);
    z80.z80.iff1 =  (iff & 1) != 0;
    z80.z80.iff2 = (iff & 2) != 0;

    let mut rr: c_long = 0;
    read_into(&mut rr, &mut i, &mut buf);
    z80.set_reg8(R, rr as u8);

    read_register(&mut z80, PC, &mut i, &mut buf);
    read_register(&mut z80, SP, &mut i, &mut buf);
    read_register(&mut z80, IX, &mut i, &mut buf);
    read_register(&mut z80, IY, &mut i, &mut buf);

    let mut mem = [0u8; 0x10000];
    mem.copy_from_slice(&buf[i..]);
    z80.memory = mem;

    Ok(z80)
}

/// Are the registers in the two Z80s identical?
///
/// Actually, doesn't check I or R registers. Doesn't check undefined bits in F
/// or F0.
fn z80_same_state(lhs: &Z80System, rhs: &Z80System) -> bool {
    for reg in [
        B, C, D, E, A, H, L, B0, C0, D0, E0, A0, L0, H0, IXH, IXL, IYH, IYL, SPH, SPL, PCH, PCL,
    ].iter()
    {
        if lhs.reg8(*reg) != rhs.reg8(*reg) {
            println!("diff register {:?}", reg);
            return false;
        }
    }

    let f_lhs = lhs.reg8(F);
    let f_rhs = rhs.reg8(F);
    // for the flag registers, don't check the undefined bits
    if f_lhs & 0b11010111 != f_rhs & 0b11010111 {
        println!("diff flags {:b} {:b}", f_lhs, f_rhs);
        return false;
    }

    let f0_lhs = lhs.reg8(F0);
    let f0_rhs = rhs.reg8(F0);
    // for the flag registers, don't check the undefined bits
    if f0_lhs & 0b11010111 != f0_rhs & 0b11010111 {
        println!("diff flags' {:b} {:b}", f0_lhs, f0_rhs);
        return false;
    }

    let lhs_slice = &lhs.memory[0..];
    let rhs_slice = &rhs.memory[0..];

    if lhs_slice != rhs_slice {
        for i in 0..lhs_slice.len() {
            if lhs_slice[i] != rhs_slice[i] {
                println!(
                    "diff memory at {:x}, {:x}, {:x}",
                    i, lhs_slice[i], rhs_slice[i]
                );
            }
        }
        return false;
    }

    true
}

/// A sequence of instructions, by their opcodes, together with their mnemonics
/// in order.
#[derive(Clone, Default, Debug)]
struct InstructionSequence {
    opcodes: Vec<u8>,
    mnemonics: String,
}

/// Generate a `Vec` of `InstructionSequence`s, each containing one instruction.
///
/// Most instruction categories will be represented. Some will not to avoid the
/// risk of writing over our own code, or because they involve IO that we are not
/// testing.
///
/// `count` is the number of instructions to generate for each instruction
/// category representing multiple instructions. For instance, LD A, n
/// corresponds to 256 different instructions, and `count` of them will appear in
/// the returned `Vec`.
fn generate_instructions<R: Rng>(count: usize, r: &mut R) -> Vec<InstructionSequence> {
    let mut instructions: Vec<InstructionSequence> = Vec::new();

    macro_rules! add_instruction {
        ($opcodes: expr, $mnemonic: ident, [$($arguments: tt),*]) => {
            let instruction_seq = InstructionSequence {
                opcodes: $opcodes,
                mnemonics: format!(
                    "{} {}\n",
                    stringify!($mnemonic),
                    stringify!(
                        $(
                            $arguments
                        ),*
                    )
                )
            };
            instructions.push(instruction_seq);
        };
    }

    macro_rules! process_instruction {
        // strangely, z80sim doesn't implement these two...
        ([0xED,0x6B,n,n]; $mnemonic:ident; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ([0xED,0x63,n,n]; $mnemonic:ident; $args:tt; $t_states:expr; $is_undoc:expr) => {};

        // avoid these two so we don't write over our own code
        ($codes:tt; lddr; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; ldir; $args:tt; $t_states:expr; $is_undoc:expr) => {};

        // z80pack doesn't seem to increment r correctly
        ($codes:tt; ld_ir; $args:tt; $t_states:expr; $is_undoc:expr) => {};

        ($codes:tt; im; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; ld; [A,R]; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; ld; [R,A]; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; ld; [A,I]; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; ld; [I,A]; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; ei; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; di; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; halt; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; pop; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; push; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; callcc; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; call; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; jr; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; jrcc; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; jp; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; jpcc; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; djnz; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; ret; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; retn; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; reti; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; retcc; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; rst; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; in_c; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; in_n; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; in_f; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; ind; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; ini; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; indr; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; inir; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; out_c; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; out_n; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; outd; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; outi; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; otir; $args:tt; $t_states:expr; $is_undoc:expr) => {};
        ($codes:tt; otdr; $args:tt; $t_states:expr; $is_undoc:expr) => {};

        ([0xDD, $code:expr]; $mnemonic:ident; $arguments:tt; $t_states:expr; false) => {
            let opcodes = vec![0xDD, $code];
            add_instruction!(opcodes, $mnemonic, $arguments);
        };
        ([0xDD, $code:expr,d]; $mnemonic:ident; $arguments:tt; $t_states:expr; false) => {
            for _ in 0..count {
                let n: u8 = r.gen();
                let opcodes = vec![0xDD, $code, n];
                add_instruction!(opcodes, $mnemonic, $arguments);
            }
        };
        ([0xDD, $code:expr,n]; $mnemonic:ident; $arguments:tt; $t_states:expr; false) => {
            for _ in 0..count {
                let n: u8 = r.gen();
                let opcodes = vec![0xDD, $code, n];
                add_instruction!(opcodes, $mnemonic, $arguments);
            }
        };
        ([0xDD, $code:expr,n,n]; $mnemonic:ident; $arguments:tt; $t_states:expr; false) => {
            for _ in 0..count {
                // avoid low addresses so we don't write over our own instruction
                let n11: u8 = r.gen();
                let n1: u8 = n11 | 0xF;
                let n2: u8 = r.gen();
                let opcodes = vec![0xDD, $code, n1, n2];
                add_instruction!(opcodes, $mnemonic, $arguments);
            }
        };
        ([0xDD, $code:expr,d,n]; $mnemonic:ident; $arguments:tt; $t_states:expr; false) => {
            for _ in 0..count {
                let n11: u8 = r.gen();
                let n1: u8 = n11 | 0xF;
                let n2: u8 = r.gen();
                let opcodes = vec![0xDD, $code, n1, n2];
                add_instruction!(opcodes, $mnemonic, $arguments);
            }
        };
        ([0xFD, $code:expr]; $mnemonic:ident; $arguments:tt; $t_states:expr; false) => {
            let opcodes = vec![0xFD, $code];
            add_instruction!(opcodes, $mnemonic, $arguments);
        };
        ([0xFD, $code:expr,d]; $mnemonic:ident; $arguments:tt; $t_states:expr; false) => {
            for _ in 0..count {
                let n: u8 = r.gen();
                let opcodes = vec![0xFD, $code, n];
                add_instruction!(opcodes, $mnemonic, $arguments);
            }
        };
        ([0xFD, $code:expr,n]; $mnemonic:ident; $arguments:tt; $t_states:expr; false) => {
            for _ in 0..count {
                let n: u8 = r.gen();
                let opcodes = vec![0xFD, $code, n];
                add_instruction!(opcodes, $mnemonic, $arguments);
            }
        };
        ([0xFD, $code:expr,n,n]; $mnemonic:ident; $arguments:tt; $t_states:expr; false) => {
            for _ in 0..count {
                let n11: u8 = r.gen();
                let n1: u8 = n11 | 0xF;
                let n2: u8 = r.gen();
                let opcodes = vec![0xFD, $code, n1, n2];
                add_instruction!(opcodes, $mnemonic, $arguments);
            }
        };
        ([0xFD, $code:expr,d,n]; $mnemonic:ident; $arguments:tt; $t_states:expr; false) => {
            for _ in 0..count {
                let n11: u8 = r.gen();
                let n1: u8 = n11 | 0xF;
                let n2: u8 = r.gen();
                let opcodes = vec![0xFD, $code, n1, n2];
                add_instruction!(opcodes, $mnemonic, $arguments);
            }
        };

        // need a patched version of z80sim for these instructions. z80sim has
        // buffer overflows in these instructions
        ([0xDD,0xCB,d, $code:expr]; $mnemonic:ident; $arguments:tt; $t_states:expr; false) => {
            let n: u8 = r.gen();
            let opcodes = vec![0xDD, 0xCB, n, $code];
            add_instruction!(opcodes, $mnemonic, $arguments);
        };
        ([0xFD,0xCB,d, $code:expr]; $mnemonic:ident; $arguments:tt; $t_states:expr; false) => {
            for _ in 0..count {
                let n: u8 = r.gen();
                let opcodes = vec![0xFD, 0xCB, n, $code];
                add_instruction!(opcodes, $mnemonic, $arguments);
            }
        };
        ([0xCB, $code:expr]; $mnemonic:ident; $arguments:tt; $t_states:expr; false) => {
            let opcodes = vec![0xCB, $code];
            add_instruction!(opcodes, $mnemonic, $arguments);
        };
        ([0xED, $code:expr,n,n]; $mnemonic:ident; $arguments:tt; $t_states:expr; false) => {
            for _ in 0..count {
                let n11: u8 = r.gen();
                let n1: u8 = n11 | 0xF;
                let n2: u8 = r.gen();
                let opcodes = vec![0xED, $code, n1, n2];
                add_instruction!(opcodes, $mnemonic, $arguments);
            }
        };
        ([0xED, $code:expr]; $mnemonic:ident; $arguments:tt; $t_states:expr; false) => {
            let opcodes = vec![0xED, $code];
            add_instruction!(opcodes, $mnemonic, $arguments);
        };
        ([$code:expr,n]; $mnemonic:ident; $arguments:tt; $t_states:expr; false) => {
            for _ in 0..count {
                let n: u8 = r.gen();
                let opcodes = vec![$code, n];
                add_instruction!(opcodes, $mnemonic, $arguments);
            }
        };
        ([$code:expr,e]; $mnemonic:ident; $arguments:tt; $t_states:expr; false) => {
            for _ in 0..count {
                let n: u8 = r.gen();
                let opcodes = vec![$code, n];
                add_instruction!(opcodes, $mnemonic, $arguments);
            }
        };
        ([$code:expr,n,n]; $mnemonic:ident; $arguments:tt; $t_states:expr; false) => {
            for _ in 0..count {
                let n11: u8 = r.gen();
                let n1: u8 = n11 | 0xF;
                let n2: u8 = r.gen();
                let opcodes = vec![$code, n1, n2];
                add_instruction!(opcodes, $mnemonic, $arguments);
            }
        };
        ([$code:expr]; $mnemonic:ident; $arguments:tt; $t_states:expr; false) => {
            let opcodes = vec![$code];
            add_instruction!(opcodes, $mnemonic, $arguments);
        };
        ($($nothing:tt)*) => {};
    }

    process_instructions!(process_instruction, d, e, n, nn);

    instructions
}

/// Given a buffer `instructions` of `InstructionSequence`s, return a Vector of
/// `count` `InstructionSequence`s, each of which is the concatenation of `size`
/// of the `InstructionSequence`s from `instructions`.
fn generate_instructions_sequence<R>(
    instructions: &[InstructionSequence],
    count: usize,
    size: usize,
    rng: &mut R,
) -> Vec<InstructionSequence>
where
    R: Rng,
{
    use rand::distributions::range::Range;
    use rand::distributions::IndependentSample;

    let mut result: Vec<InstructionSequence> = Vec::with_capacity(count);
    for _ in 0..count {
        let inst = InstructionSequence {
            opcodes: Vec::new(),
            mnemonics: "".to_owned(),
        };
        result.push(inst);
    }
    for _ in 0..size {
        for i in 0..count {
            let j = Range::new(0usize, instructions.len()).ind_sample(rng);
            let new_inst = &instructions[j];
            result[i].opcodes.extend_from_slice(&new_inst.opcodes[..]);
            result[i].mnemonics.push_str(&new_inst.mnemonics);
        }
    }
    result
}

#[derive(Clone)]
pub struct TestFailure {
    mnemonics: String,
    original_z80: Z80System,
    sim_z80: Z80System,
    attalus_z80: Z80System,
}

impl std::fmt::Debug for TestFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "TestFailure \
            {{ \n\
                mnemonics: {:?}, \n\
                original_z80: \n{}\n,\n\
                sim_z80: \n{}\n,\n\
                attalus_z80: \n{}\n
            }}",
            self.mnemonics, self.original_z80, self.sim_z80, self.attalus_z80,
        )
    }
}

#[derive(Clone, Debug)]
pub enum TestResult {
    TestOk,
    TestFailed(TestFailure),
}

use self::TestResult::*;

/// Run both `z80sim` and `attalus` on the instruction sequence.
///
/// The first 32 KiB of memory (other than the instructions) is filled with 0x76
/// HALT instructions. The last 32 KiB of memory, together with most of the
/// registers of the Z80, are randomly generated.
///
/// `count` tests will be run, each with different randomly generated registers
/// and memory.
fn test_instruction_sequence<R>(
    instruction_sequence: &InstructionSequence,
    count: usize,
    rng: &mut R,
) -> Result<TestResult>
where
    R: Rng,
{
    use std::io::{Read, Write};
    use std::process::{Child, ChildStdout, Command, Stdio};

    use tempdir::TempDir;

    fn wait_for_prompt(stdout: &mut ChildStdout) -> Result<()> {
        let mut arrow_count = 0u32;
        let mut buf = [0u8];
        let arrow = ">".as_bytes();
        let space = " ".as_bytes();
        for _ in 0..10000 {
            stdout.read(&mut buf)?;
            if arrow[0] == buf[0] {
                arrow_count += 1;
            } else if arrow_count == 3 && space[0] == buf[0] {
                return Ok(());
            }
        }
        Err(failure::err_msg("z80sim won't give a prompt"))
    }

    fn wait_for_exit(child: &mut Child) -> Result<()> {
        for _ in 0..50 {
            let status = child.try_wait()?;
            match status {
                Some(status) => {
                    if status.success() {
                        return Ok(());
                    } else {
                        return Err(failure::err_msg("exit failure from z80sim"));
                    }
                }
                _ => {}
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        child.kill()?;

        Err(failure::err_msg("z80sim would not exit"))
    }

    let instructions = instruction_sequence.opcodes.clone();
    for i in 0..count {
        println!("\nTest {} of \n{:}", i, instruction_sequence.mnemonics);
        let mut z80 = random_system(&instructions[..], rng);
        z80.set_reg16(PC, 0);

        let dir = TempDir::new("attalus_tmp")?;
        let file_path = dir.path().join("core.z80");
        write_core(&file_path, &z80)?;
        std::thread::sleep(std::time::Duration::from_millis(1));
        let mut child = Command::new("z80sim")
                    .current_dir(dir.path())
                    .arg("-l")      // load from the core.z80 file
                    .arg("-s")      // save on exit to the core.z80 file
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()?;
        {
            let child_stdin = child.stdin.as_mut().unwrap();
            let child_stdout = child.stdout.as_mut().unwrap();
            wait_for_prompt(child_stdout)?;
            child_stdin.write_all("g\n".as_bytes())?;
            wait_for_prompt(child_stdout)?;
            child_stdin.write_all("q\n".as_bytes())?;
        }
        wait_for_exit(&mut child)?;
        let sim_z80 = read_core(&file_path)?;
        let mut attalus_z80 = z80.clone();
        attalus_z80.run(1000);

        // z80sim bumps up PC even after a halt
        let pc = attalus_z80.reg16(PC);
        attalus_z80.set_reg16(PC, pc + 1);

        if !z80_same_state(&attalus_z80, &sim_z80) {
            return Ok(TestFailed(TestFailure {
                mnemonics: instruction_sequence.mnemonics.clone(),
                original_z80: z80,
                sim_z80: sim_z80,
                attalus_z80: attalus_z80,
            }));
        }
        println!("Test passed\n");
    }
    return Ok(TestOk);
}

pub fn test_against<R>(
    count: usize,
    n_instructions: usize,
    trials: usize,
    rng: &mut R,
) -> Result<TestResult>
where
    R: Rng,
{
    let instructions = generate_instructions(5, rng);
    let all_instructions =
        generate_instructions_sequence(&instructions, count, n_instructions, rng);
    for inst in all_instructions.iter() {
        match test_instruction_sequence(inst, trials, rng)? {
            TestOk => continue,
            x => return Ok(x),
        }
    }
    return Ok(TestOk);
}

fn print_usage_and_exit() -> ! {
    println!(
        "Usage: test_z80_against \
         [-n Number-of-tested-sequences] \
         [-i Instructions-per-sequence] \
         [-t Trials-per-sequence] \
         [-s random-number-Seed] \
         "
    );
    exit(1)
}

fn parse_or_die<F>(s: Option<String>) -> F
where
    F: FromStr,
{
    match s {
        Some(t) => match t.parse() {
            Ok(f) => f,
            _ => print_usage_and_exit(),
        },
        _ => print_usage_and_exit(),
    }
}

fn main() {
    let mut n: usize = 1000;
    let mut i: usize = 1;
    let mut t: usize = 5;
    let mut s: u32 = 9800;

    let mut args_iter = args();
    args_iter.next();
    while let Some(arg) = args_iter.next() {
        match arg.as_ref() {
            "-n" => n = parse_or_die(args_iter.next()),
            "-i" => i = parse_or_die(args_iter.next()),
            "-t" => t = parse_or_die(args_iter.next()),
            "-s" => s = parse_or_die(args_iter.next()),
            _ => print_usage_and_exit(),
        }
    }

    let mut rng = rand::XorShiftRng::from_seed([s, 2, 3, 4]);
    let r = test_against(n, i, t, &mut rng);
    match r {
        Ok(TestResult::TestFailed(x)) => {
            println!("test failure {:?}", x);
            return;
        }
        Ok(TestOk) => {
            println!("All tests passed");
            return;
        }
        Err(e) => {
            eprintln!("Unable to conduct tests: {}", e);
            exit(1);
        }
    }
}
