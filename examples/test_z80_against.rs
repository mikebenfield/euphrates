// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

//! test_z80_against: Test the z80 interpreter in `attalus` against the emulator
//! `z80sim`. Requires that `z80sim` be in your $PATH. Will test many instruction
//! sequences with several different starting Z80 states. `z80sim` doesn't
//! implement most undocumented opcodes, so we don't test undocumented opcodes.
//! We also don't test opcodes like `lddr` that are likely to overwrite our own
//! instructions. It is still in princple possible that our instructions could be
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

extern crate tempdir;
extern crate rand;

#[macro_use]
extern crate attalus;

use std::path::Path;
use std::error::Error;
use std::os::raw::{c_int, c_long};
use std::mem::size_of;
use std::env::args;
use std::str::FromStr;
use std::process::exit;

use rand::{Rng, SeedableRng};

use attalus::hardware::z80::*;
use attalus::hardware::io::*;

#[derive(Clone, Debug)]
pub struct TestAgainstError {
    msg: String
}

impl std::convert::From<std::io::Error> for TestAgainstError {
    fn from(err: std::io::Error) -> TestAgainstError {
        TestAgainstError {
            msg: format!("Error reading file: {}", err.description())
        }
    }
}

impl std::fmt::Display for TestAgainstError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for TestAgainstError {
    fn description(&self) -> &str {
        &self.msg
    }
}

fn random_z80<R>(mem_start: &[u8], rng: &mut R) -> Z80<SimpleIo>
where
    R: Rng
{
    // 0x76: HALT
    let mut mem = [0x76u8; 0x10000];
    for i in 0x800..0x10000 {
        mem[i] = rng.gen();
    }
    mem[0..mem_start.len()].copy_from_slice(mem_start);
    let mut z = Z80::new(SimpleIo { mem: mem });
    for reg in [
        B, C, D, E, A, H, L,
        B0, C0, D0, E0, A0, L0, H0,
        IXH, IXL, IYH, IYL,
    ].iter() {
        reg.set(&mut z, rng.gen());
    }
    z
}

fn write_core<P: AsRef<Path>>(
    path: P,
    z80: &Z80<SimpleIo>
) -> Result<(), TestAgainstError> {
    use std::fs::File;
    use std::io::Write;

    let z80_size: usize =
        16 + 2*4 + 2 * size_of::<c_int>() + size_of::<c_long>();

    let mut buf: Vec<u8> = Vec::with_capacity(z80_size);

    fn write_from<T>(val: T, buf: &mut Vec<u8>) {
        let size = size_of::<T>() as isize;
        unsafe {
            let valp: *const u8 = std::mem::transmute(&val);
            for j in 0..size {
                buf.push(*valp.offset(j));
            }
        }
    }

    write_from(A.get(z80), &mut buf);
    write_from(F.get(z80) as c_int, &mut buf);
    write_from(B.get(z80), &mut buf);
    write_from(C.get(z80), &mut buf);
    write_from(D.get(z80), &mut buf);
    write_from(E.get(z80), &mut buf);
    write_from(H.get(z80), &mut buf);
    write_from(L.get(z80), &mut buf);

    write_from(A0.get(z80), &mut buf);
    write_from(F0.get(z80) as c_int, &mut buf);
    write_from(B0.get(z80), &mut buf);
    write_from(C0.get(z80), &mut buf);
    write_from(D0.get(z80), &mut buf);
    write_from(E0.get(z80), &mut buf);
    write_from(H0.get(z80), &mut buf);
    write_from(L0.get(z80), &mut buf);

    write_from(I.get(z80), &mut buf);

    let iff1: u8 = if z80.iff1 == 0xFFFFFFFFFFFFFFFF { 0 } else { 1 };
    let iff2: u8 = if z80.iff2 { 2 } else { 0 };
    write_from(iff1 | iff2, &mut buf);

    write_from(R.get(z80) as c_long, &mut buf);

    write_from(PC.get(z80), &mut buf);
    write_from(SP.get(z80), &mut buf);
    write_from(IX.get(z80), &mut buf);
    write_from(IY.get(z80), &mut buf);

    let mut f = File::create(path)?;

    f.write_all(&buf[..])?;
    f.write_all(&z80.io.mem()[..])?;
    Ok(())
}

fn read_core<P>(path: P) -> Result<Z80<SimpleIo>, TestAgainstError>
where
    P: AsRef<Path>
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
        0x10000 + 16 + 2*4 + 2 * size_of::<c_int>() + size_of::<c_long>();

    let mut z80: Z80<SimpleIo> = Default::default();

    let mut buf: Vec<u8> = Vec::new();
    {
        let mut f = File::open(path)?;
        f.read_to_end(&mut buf)?;
    }

    if buf.len() != correct_core_size {
        return Err(
            TestAgainstError {
                msg: format!(
                    "Core file of wrong length {} (should be {})",
                    buf.len(),
                    correct_core_size)
            }
        );
    }

    let mut i: usize = 0;

    fn read_into<T>(val: &mut T, i: &mut usize, buf: &mut [u8]) {
        let size = size_of::<T>() as isize;
        unsafe {
            let valp: *mut u8 = std::mem::transmute(val);
            for j in 0..size {
                *valp.offset(j) = buf[*i];
                *i += 1;
            }
        }
    }

    fn read_register<R, T>(z: &mut Z80<SimpleIo>, reg: R, i: &mut usize, buf: &mut [u8])
    where
        R: Settable<T>,
        T: Default,
    {
        let mut t: T = Default::default();
        read_into(&mut t, i, buf);
        reg.set(z, t);
    }

    read_register(&mut z80, A, &mut i, &mut buf);
    let mut ff: c_int = 0;
    read_into(&mut ff, &mut i, &mut buf);
    F.set(&mut z80, ff as u8);
    read_register(&mut z80, B, &mut i, &mut buf);
    read_register(&mut z80, C, &mut i, &mut buf);
    read_register(&mut z80, D, &mut i, &mut buf);
    read_register(&mut z80, E, &mut i, &mut buf);
    read_register(&mut z80, H, &mut i, &mut buf);
    read_register(&mut z80, L, &mut i, &mut buf);

    read_register(&mut z80, A0, &mut i, &mut buf);
    let mut ff0: c_int = 0;
    read_into(&mut ff0, &mut i, &mut buf);
    F0.set(&mut z80, ff0 as u8);
    read_register(&mut z80, B0, &mut i, &mut buf);
    read_register(&mut z80, C0, &mut i, &mut buf);
    read_register(&mut z80, D0, &mut i, &mut buf);
    read_register(&mut z80, E0, &mut i, &mut buf);
    read_register(&mut z80, H0, &mut i, &mut buf);
    read_register(&mut z80, L0, &mut i, &mut buf);

    read_register(&mut z80, I, &mut i, &mut buf);

    let mut iff: u8 = 0;
    read_into(&mut iff, &mut i, &mut buf);
    z80.iff1 = if (iff & 1) == 0 {
        0xFFFFFFFFFFFFFFFF
    } else {
        0
    };
    z80.iff2 = (iff & 2) != 0;

    let mut rr: c_long = 0;
    read_into(&mut rr, &mut i, &mut buf);
    R.set(&mut z80, rr as u8);

    read_register(&mut z80, PC, &mut i, &mut buf);
    read_register(&mut z80, SP, &mut i, &mut buf);
    read_register(&mut z80, IX, &mut i, &mut buf);
    read_register(&mut z80, IY, &mut i, &mut buf);

    let mut mem = [0u8; 0x10000];
    mem.copy_from_slice(&buf[i..]);
    z80.io = SimpleIo { mem: mem };

    Ok(z80)
}

/// Are the registers in the two Z80s identical?
///
/// Actually, doesn't check I or R registers. Doesn't check undefined bits in F
/// or F0.
fn z80_same_state(lhs: &Z80<SimpleIo>, rhs: &Z80<SimpleIo>) -> bool  {
    for reg in [ 
        B, C, D, E, A, H, L,
        B0, C0, D0, E0, A0, L0, H0,
        IXH, IXL, IYH, IYL,
        SPH, SPL, PCH, PCL,
    ].iter() {
        if reg.get(lhs) != reg.get(rhs) {
            println!("diff register {:?}", reg);
            return false;
        }
    }

    let f_lhs = F.get(lhs);
    let f_rhs = F.get(rhs);
    if f_lhs & 0b11010111 != f_rhs & 0b11010111 {
        return false;
    }

    let f0_lhs = F0.get(lhs);
    let f0_rhs = F0.get(rhs);
    if f0_lhs & 0b11010111 != f0_rhs & 0b11010111 {
        println!("diff flags' {:b} {:b}", f0_lhs, f0_rhs);
        return false;
    }

    if lhs.io.mem()[..] != rhs.io.mem()[..] {
        for i in 0..lhs.io.mem.len() {
            if lhs.io.mem()[i] != rhs.io.mem()[i] {
                println!("diff memory at {:x}, {:x}, {:x}", i, lhs.io.mem()[i], rhs.io.mem()[i]);
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
        ([0xED, 0x6B, n, n] ; $mnemonic: ident ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ([0xED, 0x63, n, n] ; $mnemonic: ident ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};

        // avoid these two so we don't write over our own code
        ($codes: tt; lddr ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; ldir ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};

        ($codes: tt; im ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; ld ; [A, R] ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; ld ; [R, A] ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; ld ; [A, I] ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; ld ; [I, A] ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; ei ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; di ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; halt ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; pop ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; push ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; callcc ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; call ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; jr ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; jrcc ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; jp ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; jpcc ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; djnz ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; ret ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; retn ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; reti ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; retcc ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; rst ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; in_c ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; in_n ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; in_f ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; ind ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; ini ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; indr ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; inir ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; out_c ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; out_n ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; outd ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; outi ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; otir ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; otdr ; $args: tt ; $t_states: expr ; $is_undoc: expr) => {};

        ([0xDD, $code: expr] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; false ) => {
            let opcodes = vec![0xDD, $code];
            add_instruction!(opcodes, $mnemonic, $arguments);
        };
        ([0xDD, $code: expr, d] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; false ) => {
            for _ in 0..count {
                let n: u8 = r.gen();
                let opcodes = vec![0xDD, $code, n];
                add_instruction!(opcodes, $mnemonic, $arguments);
            }
        };
        ([0xDD, $code: expr, n] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; false ) => {
            for _ in 0..count {
                let n: u8 = r.gen();
                let opcodes = vec![0xDD, $code, n];
                add_instruction!(opcodes, $mnemonic, $arguments);
            }
        };
        ([0xDD, $code: expr, n, n] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; false ) => {
            for _ in 0..count {
                // avoid low addresses so we don't write over our own instruction
                let n11: u8 = r.gen();
                let n1: u8 = n11 | 0xF;
                let n2: u8 = r.gen();
                let opcodes = vec![0xDD, $code, n1, n2];
                add_instruction!(opcodes, $mnemonic, $arguments);
            }
        };
        ([0xDD, $code: expr, d, n] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; false ) => {
            for _ in 0..count {
                let n11: u8 = r.gen();
                let n1: u8 = n11 | 0xF;
                let n2: u8 = r.gen();
                let opcodes = vec![0xDD, $code, n1, n2];
                add_instruction!(opcodes, $mnemonic, $arguments);
            }
        };
        ([0xFD, $code: expr] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; false ) => {
            let opcodes = vec![0xFD, $code];
            add_instruction!(opcodes, $mnemonic, $arguments);
        };
        ([0xFD, $code: expr, d] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; false ) => {
            for _ in 0..count {
                let n: u8 = r.gen();
                let opcodes = vec![0xFD, $code, n];
                add_instruction!(opcodes, $mnemonic, $arguments);
            }
        };
        ([0xFD, $code: expr, n] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; false ) => {
            for _ in 0..count {
                let n: u8 = r.gen();
                let opcodes = vec![0xFD, $code, n];
                add_instruction!(opcodes, $mnemonic, $arguments);
            }
        };
        ([0xFD, $code: expr, n, n] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; false ) => {
            for _ in 0..count {
                let n11: u8 = r.gen();
                let n1: u8 = n11 | 0xF;
                let n2: u8 = r.gen();
                let opcodes = vec![0xFD, $code, n1, n2];
                add_instruction!(opcodes, $mnemonic, $arguments);
            }
        };
        ([0xFD, $code: expr, d, n] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; false ) => {
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
        ([0xDD, 0xCB, d, $code: expr] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; false ) => {
            let n: u8 = r.gen();
            let opcodes = vec![0xDD, 0xCB, n, $code];
            add_instruction!(opcodes, $mnemonic, $arguments);
        };
        ([0xFD, 0xCB, d, $code: expr] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; false ) => {
            for _ in 0..count {
                let n: u8 = r.gen();
                let opcodes = vec![0xFD, 0xCB, n, $code];
                add_instruction!(opcodes, $mnemonic, $arguments);
            }
        };

        ([0xCB, $code: expr] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; false ) => {
            let opcodes = vec![0xCB, $code];
            add_instruction!(opcodes, $mnemonic, $arguments);
        };
        ([0xED, $code: expr, n, n] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; false ) => {
            for _ in 0..count {
                let n11: u8 = r.gen();
                let n1: u8 = n11 | 0xF;
                let n2: u8 = r.gen();
                let opcodes = vec![0xED, $code, n1, n2];
                add_instruction!(opcodes, $mnemonic, $arguments);
            }
        };
        ([0xED, $code: expr] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; false ) => {
            let opcodes = vec![0xED, $code];
            add_instruction!(opcodes, $mnemonic, $arguments);
        };
        ([$code: expr, n] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; false ) => {
            for _ in 0..count {
                let n: u8 = r.gen();
                let opcodes = vec![$code, n];
                add_instruction!(opcodes, $mnemonic, $arguments);
            }
        };
        ([$code: expr, e] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; false ) => {
            for _ in 0..count {
                let n: u8 = r.gen();
                let opcodes = vec![$code, n];
                add_instruction!(opcodes, $mnemonic, $arguments);
            }
        };
        ([$code: expr, n, n] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; false ) => {
            for _ in 0..count {
                let n11: u8 = r.gen();
                let n1: u8 = n11 | 0xF;
                let n2: u8 = r.gen();
                let opcodes = vec![$code, n1, n2];
                add_instruction!(opcodes, $mnemonic, $arguments);
            }
        };
        ([$code: expr] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; false ) => {
            let opcodes = vec![$code];
            add_instruction!(opcodes, $mnemonic, $arguments);
        };
        ($($nothing: tt)*) => {
        }
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
    rng: &mut R
) -> Vec<InstructionSequence>
where
    R: Rng
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
    original_z80: Z80<SimpleIo>,
    sim_z80: Z80<SimpleIo>,
    attalus_z80: Z80<SimpleIo>,
}

impl std::fmt::Debug for TestFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "TestFailure \
            {{ \n\
                mnemonics: {:?}, \n\
                original_z80: \n{}\n,\n\
                sim_z80: \n{}\n,\n\
                attalus_z80: \n{}\n
            }}",
            self.mnemonics,
            self.original_z80,
            self.sim_z80,
            self.attalus_z80,
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
    rng: &mut R
) -> Result<TestResult, TestAgainstError>
where
    R: Rng
{
    use std::io::{Read, Write};
    use std::process::{Command, Stdio, ChildStdout, Child};

    use tempdir::TempDir;

    fn wait_for_prompt(stdout: &mut ChildStdout) -> Result<(), TestAgainstError> {
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
        Err(
            TestAgainstError {
                msg: "z80sim won't give a prompt".to_owned()
            }
        )
    }

    fn wait_for_exit(child: &mut Child) -> Result<(), TestAgainstError> {
        for _ in 0..50 {
            let status = child.try_wait()?;
            match status {
                Some(status) => {
                    if status.success() {
                        return Ok(());
                    } else {
                        return Err (
                            TestAgainstError {
                                msg: "exit failure from z80sim".to_owned()
                            }
                        );
                    }
                }
                _ => {}
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        child.kill()?;
        Err (
            TestAgainstError {
                msg: "z80sim would not exit".to_owned()
            }
        )
    }

    let instructions = instruction_sequence.opcodes.clone();
    for i in 0..count {
        println!("\nTest {} of \n{:}", i, instruction_sequence.mnemonics);
        let mut z80 = random_z80(&instructions[..], rng);
        PC.set(&mut z80, 0);
        
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
        Z80Interpreter {}.run(&mut attalus_z80, 1000);

        // z80sim bumps up PC even after a halt
        let pc = PC.get(&attalus_z80);
        PC.set(&mut attalus_z80, pc + 1);

        if !z80_same_state(&attalus_z80, &sim_z80) {
            return Ok(
                TestFailed(
                    TestFailure {
                        mnemonics: instruction_sequence.mnemonics.clone(),
                        original_z80: z80,
                        sim_z80: sim_z80,
                        attalus_z80: attalus_z80,
                    }
                )
            );
        }
        println!("Test passed\n");
    }
    return Ok(TestOk);
}

pub fn test_against<R>(
    count: usize,
    n_instructions: usize,
    trials: usize,
    rng: &mut R
) -> Result<TestResult, TestAgainstError>
where
    R: Rng
{
    let instructions = generate_instructions(5, rng);
    let all_instructions = generate_instructions_sequence(
        &instructions,
        count,
        n_instructions,
        rng
    );
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
    F: FromStr
{
    match s {
        Some(t) => match t.parse() {
            Ok(f) => f,
            _ => print_usage_and_exit()
        },
        _ => print_usage_and_exit()
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
        },
        Ok(TestOk) => {
            println!("All tests passed");
            return;
        },
        Err(e) => {
            eprintln!("Unable to conduct tests: {:}", e.description());
            exit(1);
        }
    }
}
