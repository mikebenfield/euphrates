
use std;
use std::path::Path;
use std::error::Error;
use std::os::raw::{c_int, c_long};
use std::mem::size_of;

use rand::Rng;

use ::log::*;
use super::types::*;
use super::interpreter::execute_loop;
use ::hardware::memory_mapper::*;
use ::hardware::memory_mapper::implementation::*;
use ::bits::*;

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

fn random_z80_hardware<R: Rng>(rng: &mut R) -> Z80Hardware {
    Z80Hardware {
        address: 0,
        data: 0,
        iff1: false,
        iff2: false,
        interrupt_mode: Im0,
        registers: rng.gen(),
    }
}

fn random_memory<R: Rng>(mem_start: &[u8], rng: &mut R) -> [u8; 0x10000] {
    let mut result: [u8; 0x10000] = [0; 0x10000];
    for i in 0..0x10000 {
        result[i] = rng.gen();
    }
    result[0..mem_start.len()].copy_from_slice(mem_start);
    result
}

fn write_core<P: AsRef<Path>>(
    path: P,
    z80_hardware: &Z80Hardware,
    mem: &[u8; 0x10000]
) -> Result<(), TestAgainstError> {
    use std::fs::File;
    use std::io::Write;

    let z80_size: usize =
        16 + 2*4 + 2 * size_of::<c_int>() + size_of::<c_long>();

    let mut buf: Vec<u8> = Vec::with_capacity(z80_size);

    fn write_from<T>(val: &T, buf: &mut Vec<u8>) {
        let size = size_of::<T>() as isize;
        unsafe {
            let valp: *const u8 = std::mem::transmute(val);
            for j in 0..size {
                buf.push(*valp.offset(j));
            }
        }
    }

    write_from(&z80_hardware.registers[A as usize], &mut buf);
    write_from(&(z80_hardware.registers[F as usize] as c_int), &mut buf);
    write_from(&z80_hardware.registers[B as usize], &mut buf);
    write_from(&z80_hardware.registers[C as usize], &mut buf);
    write_from(&z80_hardware.registers[D as usize], &mut buf);
    write_from(&z80_hardware.registers[E as usize], &mut buf);
    write_from(&z80_hardware.registers[H as usize], &mut buf);
    write_from(&z80_hardware.registers[L as usize], &mut buf);

    write_from(&z80_hardware.registers[A0 as usize], &mut buf);
    write_from(&(z80_hardware.registers[F0 as usize] as c_int), &mut buf);
    write_from(&z80_hardware.registers[B0 as usize], &mut buf);
    write_from(&z80_hardware.registers[C0 as usize], &mut buf);
    write_from(&z80_hardware.registers[D0 as usize], &mut buf);
    write_from(&z80_hardware.registers[E0 as usize], &mut buf);
    write_from(&z80_hardware.registers[H0 as usize], &mut buf);
    write_from(&z80_hardware.registers[L0 as usize], &mut buf);

    write_from(&z80_hardware.registers[I as usize], &mut buf);

    let iff1: u8 = if z80_hardware.iff1 { 1 } else { 0 };
    let iff2: u8 = if z80_hardware.iff2 { 2 } else { 0 };
    write_from(&(iff1 | iff2), &mut buf);

    write_from(&(z80_hardware.registers[R as usize] as c_long), &mut buf);

    let pcl = z80_hardware.registers[PCL as usize];
    let pch = z80_hardware.registers[PCH as usize];
    write_from(&to16(pcl, pch), &mut buf);

    let spl = z80_hardware.registers[SPL as usize];
    let sph = z80_hardware.registers[SPH as usize];
    write_from(&to16(spl, sph), &mut buf);

    let ixl = z80_hardware.registers[IXL as usize];
    let ixh = z80_hardware.registers[IXH as usize];
    write_from(&to16(ixl, ixh), &mut buf);

    let iyl = z80_hardware.registers[IYL as usize];
    let iyh = z80_hardware.registers[IYH as usize];
    write_from(&to16(iyl, iyh), &mut buf);

    let mut f = File::create(path)?;

    f.write_all(&buf[..])?;
    f.write_all(mem)?;
    Ok(())
}

fn read_core<P: AsRef<Path>>(path: P) -> Result<(Z80Hardware, [u8; 0x10000]), TestAgainstError> {
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

    let mut z80_hardware: Z80Hardware = Default::default();

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

    read_into(&mut z80_hardware.registers[A as usize], &mut i, &mut buf);
    let mut ff: c_int = 0;
    read_into(&mut ff, &mut i, &mut buf);
    z80_hardware.registers[F as usize] = ff as u8;
    read_into(&mut z80_hardware.registers[B as usize], &mut i, &mut buf);
    read_into(&mut z80_hardware.registers[C as usize], &mut i, &mut buf);
    read_into(&mut z80_hardware.registers[D as usize], &mut i, &mut buf);
    read_into(&mut z80_hardware.registers[E as usize], &mut i, &mut buf);
    read_into(&mut z80_hardware.registers[H as usize], &mut i, &mut buf);
    read_into(&mut z80_hardware.registers[L as usize], &mut i, &mut buf);

    read_into(&mut z80_hardware.registers[A0 as usize], &mut i, &mut buf);
    let mut ff0: c_int = 0;
    read_into(&mut ff0, &mut i, &mut buf);
    z80_hardware.registers[F0 as usize] = ff0 as u8;
    read_into(&mut z80_hardware.registers[B0 as usize], &mut i, &mut buf);
    read_into(&mut z80_hardware.registers[C0 as usize], &mut i, &mut buf);
    read_into(&mut z80_hardware.registers[D0 as usize], &mut i, &mut buf);
    read_into(&mut z80_hardware.registers[E0 as usize], &mut i, &mut buf);
    read_into(&mut z80_hardware.registers[H0 as usize], &mut i, &mut buf);
    read_into(&mut z80_hardware.registers[L0 as usize], &mut i, &mut buf);

    read_into(&mut z80_hardware.registers[I as usize], &mut i, &mut buf);

    let mut iff: u8 = 0;
    read_into(&mut iff, &mut i, &mut buf);
    z80_hardware.iff1 = (iff & 1) != 0;
    z80_hardware.iff2 = (iff & 2) != 0;

    let mut rr: c_long = 0;
    read_into(&mut rr, &mut i, &mut buf);
    z80_hardware.registers[R as usize] = rr as u8;

    let mut pc = 0u16;
    read_into(&mut pc, &mut i, &mut buf);
    z80_hardware.registers[PCL as usize] = (pc & 0xFF) as u8;
    z80_hardware.registers[PCH as usize] = (pc >> 8) as u8;

    let mut sp = 0u16;
    read_into(&mut sp, &mut i, &mut buf);
    z80_hardware.registers[SPL as usize] = (sp & 0xFF) as u8;
    z80_hardware.registers[SPH as usize] = (sp >> 8) as u8;

    let mut ix = 0u16;
    read_into(&mut ix, &mut i, &mut buf);
    z80_hardware.registers[IXL as usize] = (ix & 0xFF) as u8;
    z80_hardware.registers[IXH as usize] = (ix >> 8) as u8;

    let mut iy = 0u16;
    read_into(&mut iy, &mut i, &mut buf);
    z80_hardware.registers[IYL as usize] = (iy & 0xFF) as u8;
    z80_hardware.registers[IYH as usize] = (iy >> 8) as u8;

    let mut mem = [0u8; 0x10000];
    mem.copy_from_slice(&buf[i..]);

    Ok((z80_hardware, mem))
}

/// Doesn't check I or R registers. Doesn't check undefined bits in F or F0
fn z80_same_state(lhs: &Z80Hardware, rhs: &Z80Hardware) -> bool  {
    for reg in [ 
        B, C, D, E, A, F, H, L,
        B0, C0, D0, E0, A0, F0, L0, H0,
        IXH, IXL, IYH, IYL,
        SPH, SPL, PCH, PCL,
    ].iter() {
        if lhs.registers[*reg as usize] != rhs.registers[*reg as usize] {
            return false;
        }
    }

    let f_lhs = lhs.registers[F as usize];
    let f_rhs = rhs.registers[F as usize];
    if f_lhs & 0b11010111 != f_rhs & 0b11010111 {
        return false;
    }

    let f0_lhs = lhs.registers[F0 as usize];
    let f0_rhs = rhs.registers[F0 as usize];
    if f0_lhs & 0b11010111 != f0_rhs & 0b11010111 {
        return false;
    }

    true
}

#[derive(Clone, Default, Debug)]
struct InstructionSequence {
    opcodes: Vec<u8>,
    mnemonics: String,
}

/// count is the number of instructions to generate for each instruction representing
/// multiple instructions
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
        ($codes: tt; ld ; [A, R] ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; ld ; [R, A] ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; ld ; [A, I] ; $t_states: expr ; $is_undoc: expr) => {};
        ($codes: tt; ld ; [I, A] ; $t_states: expr ; $is_undoc: expr) => {};
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
        // ([0xDD, $code: expr] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; $is_undoc: expr ) => {
        //     let opcodes = vec![0xDD, $code];
        //     add_instruction!(opcodes, $mnemonic, $arguments);
        // };
        // ([0xDD, $code: expr, d] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; $is_undoc: expr ) => {
        //     for _ in 0..count {
        //         let n: u8 = r.gen();
        //         let opcodes = vec![0xDD, $code, n];
        //         add_instruction!(opcodes, $mnemonic, $arguments);
        //     }
        // };
        // ([0xDD, $code: expr, n] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; $is_undoc: expr ) => {
        //     for _ in 0..count {
        //         let n: u8 = r.gen();
        //         let opcodes = vec![0xDD, $code, n];
        //         add_instruction!(opcodes, $mnemonic, $arguments);
        //     }
        // };
        // ([0xDD, $code: expr, n, n] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; $is_undoc: expr ) => {
        //     for _ in 0..count {
        //         let n1: u8 = r.gen();
        //         let n2: u8 = r.gen();
        //         let opcodes = vec![0xDD, $code, n1, n2];
        //         add_instruction!(opcodes, $mnemonic, $arguments);
        //     }
        // };
        // ([0xDD, $code: expr, d, n] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; $is_undoc: expr ) => {
        //     for _ in 0..count {
        //         let n1: u8 = r.gen();
        //         let n2: u8 = r.gen();
        //         let opcodes = vec![0xDD, $code, n1, n2];
        //         add_instruction!(opcodes, $mnemonic, $arguments);
        //     }
        // };
        // ([0xFD, $code: expr] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; $is_undoc: expr ) => {
        //     let opcodes = vec![0xFD, $code];
        //     add_instruction!(opcodes, $mnemonic, $arguments);
        // };
        // ([0xFD, $code: expr, d] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; $is_undoc: expr ) => {
        //     for _ in 0..count {
        //         let n: u8 = r.gen();
        //         let opcodes = vec![0xFD, $code, n];
        //         add_instruction!(opcodes, $mnemonic, $arguments);
        //     }
        // };
        // ([0xFD, $code: expr, n] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; $is_undoc: expr ) => {
        //     for _ in 0..count {
        //         let n: u8 = r.gen();
        //         let opcodes = vec![0xFD, $code, n];
        //         add_instruction!(opcodes, $mnemonic, $arguments);
        //     }
        // };
        // ([0xFD, $code: expr, n, n] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; $is_undoc: expr ) => {
        //     for _ in 0..count {
        //         let n1: u8 = r.gen();
        //         let n2: u8 = r.gen();
        //         let opcodes = vec![0xFD, $code, n1, n2];
        //         add_instruction!(opcodes, $mnemonic, $arguments);
        //     }
        // };
        // ([0xFD, $code: expr, d, n] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; $is_undoc: expr ) => {
        //     for _ in 0..count {
        //         let n1: u8 = r.gen();
        //         let n2: u8 = r.gen();
        //         let opcodes = vec![0xFD, $code, n1, n2];
        //         add_instruction!(opcodes, $mnemonic, $arguments);
        //     }
        // };
        // ([0xDD, 0xCB, d, $code: expr] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; $is_undoc: expr ) => {
        //     let n: u8 = r.gen();
        //     let opcodes = vec![0xDD, 0xCB, n, $code];
        //     add_instruction!(opcodes, $mnemonic, $arguments);
        // };
        // ([0xFD, 0xCB, d, $code: expr] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; $is_undoc: expr ) => {
        //     for _ in 0..count {
        //         let n: u8 = r.gen();
        //         let opcodes = vec![0xFD, 0xCB, n, $code];
        //         add_instruction!(opcodes, $mnemonic, $arguments);
        //     }
        // };
        // ([0xCB, $code: expr] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; $is_undoc: expr ) => {
        //     let opcodes = vec![0xCB, $code];
        //     add_instruction!(opcodes, $mnemonic, $arguments);
        // };
        // ([0xED, $code: expr, n, n] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; $is_undoc: expr ) => {
        //     for _ in 0..count {
        //         let n1: u8 = r.gen();
        //         let n2: u8 = r.gen();
        //         let opcodes = vec![0xED, $code, n1, n2];
        //         add_instruction!(opcodes, $mnemonic, $arguments);
        //     }
        // };
        // ([0xED, $code: expr] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; $is_undoc: expr ) => {
        //     let opcodes = vec![0xED, $code];
        //     add_instruction!(opcodes, $mnemonic, $arguments);
        // };
        // ([$code: expr, n] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; $is_undoc: expr ) => {
        //     for _ in 0..count {
        //         let n: u8 = r.gen();
        //         let opcodes = vec![$code, n];
        //         add_instruction!(opcodes, $mnemonic, $arguments);
        //     }
        // };
        // ([$code: expr, e] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; $is_undoc: expr ) => {
        //     for _ in 0..count {
        //         let n: u8 = r.gen();
        //         let opcodes = vec![$code, n];
        //         add_instruction!(opcodes, $mnemonic, $arguments);
        //     }
        // };
        // ([$code: expr, n, n] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; $is_undoc: expr ) => {
        //     for _ in 0..count {
        //         let n1: u8 = r.gen();
        //         let n2: u8 = r.gen();
        //         let opcodes = vec![$code, n1, n2];
        //         add_instruction!(opcodes, $mnemonic, $arguments);
        //     }
        // };
        ([$code: expr] ; $mnemonic: ident ; $arguments: tt ; $t_states: expr ; $is_undoc: expr ) => {
            let opcodes = vec![$code];
            add_instruction!(opcodes, $mnemonic, $arguments);
        };
        ($($nothing: tt)*) => {

        }
    }

    process_instructions!(process_instruction, d, e, n, nn);

    instructions
}

fn generate_instructions_sequence<R: Rng>(instructions: &[InstructionSequence], count: usize, size: usize, rng: &mut R) -> Vec<InstructionSequence> {
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

#[derive(Copy, Clone, Debug)]
struct Z80X {
    z80_hardware: Z80Hardware,
    memory_map_hw: SimpleMemoryMapperHardware,
}

impl Log for Z80X {
    fn log_minor0(&mut self, s: String) {}
    fn log_major0(&mut self, s: String) {}
    fn log_fault0(&mut self, s: String) {}
    fn does_log_minor(&self) -> bool { false }
    fn does_log_major(&self) -> bool { false }
    fn does_log_fault(&self) -> bool { false }
    fn check_fault(&self) -> Option<String> { None }
}

impl MemoryMapper0 for Z80X {
    type Hardware = SimpleMemoryMapperHardware;
    fn get_memory_mapper_hardware(&self) -> &SimpleMemoryMapperHardware {
        &self.memory_map_hw
    }
    fn get_mut_memory_mapper_hardware(&mut self) -> &mut SimpleMemoryMapperHardware {
        &mut self.memory_map_hw
    }
}

impl ::hardware::io::Io for Z80X {
    fn get_io_hardware(&self) -> &::hardware::io::IoHardware {
        unimplemented!()
    }
    fn get_mut_io_hardware(&mut self) -> &mut ::hardware::io::IoHardware {
        unimplemented!()
    }
}

impl ::hardware::irq::Irq for Z80X {
	fn request_maskable_interrupt(&mut self) -> bool {
        unimplemented!()
    }
    fn request_nonmaskable_interrupt(&mut self) {
        unimplemented!()
    }
}

impl ::hardware::vdp::Vdp for Z80X {
    fn get_vdp_hardware(&self) -> &::hardware::vdp::VdpHardware {
        unimplemented!()
    }
    fn get_mut_vdp_hardware(&mut self) -> &mut ::hardware::vdp::VdpHardware {
        unimplemented!()
    }
}

impl Z80 for Z80X {
    fn get_z80_hardware(&self) -> &Z80Hardware { &self.z80_hardware }
    fn get_mut_z80_hardware(&mut self) -> &mut Z80Hardware { &mut self.z80_hardware }
    fn advance_t_states(&mut self, _: u64) {}
    fn get_t_states(&self) -> u64 { 0 }
    fn end_on_halt(&self) -> bool { true }
    fn use_r_register(&self) -> bool { false }
}

pub struct TestFailure {
    mnemonics: String,
    original_hw: Z80Hardware,
    original_mem: [u8; 0x10000],
    z80sim_hw: Z80Hardware,
    z80sim_mem: [u8; 0x10000],
    attalus_hw: Z80Hardware,
    attalus_mem: [u8; 0x10000],
}

impl Clone for TestFailure {
    fn clone(&self) -> TestFailure {
        TestFailure {
            mnemonics: self.mnemonics.clone(),
            original_hw: self.original_hw,
            original_mem: self.original_mem,
            z80sim_hw: self.z80sim_hw,
            z80sim_mem: self.z80sim_mem,
            attalus_hw: self.attalus_hw,
            attalus_mem: self.attalus_mem,
        }
    }
}

impl std::fmt::Debug for TestFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "TestFailure \
            {{ \n\
                mnemonics: {:?}, \n\
                original_hw: {:?}, \noriginal_mem: {:?} (...), \n\
                z80sim_hw: {:?}, \nz80sim_mem: {:?} (...), \n\
                attalus_hw: {:?}, \nattalus_mem: {:?} (...) \n\
            }}",
            self.mnemonics,
            self.original_hw,
            &self.original_mem[0..64],
            self.z80sim_hw,
            &self.z80sim_mem[0..64],
            self.attalus_hw,
            &self.attalus_mem[0..64],
        )
    }
}

#[derive(Clone, Debug)]
pub enum TestResult {
    TestOk,
    TestFailed(TestFailure),
}

use self::TestResult::*;

fn test_instruction_sequence<R: Rng>(
    instruction_sequence: &InstructionSequence,
    count: usize,
    rng: &mut R
) -> Result<TestResult, TestAgainstError> {
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
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        child.kill()?;
        Err (
            TestAgainstError {
                msg: "z80sim would not exit".to_owned()
            }
        )
    }

    let mut instructions = instruction_sequence.opcodes.clone();
    instructions.push(0x76); // HALT
    for _ in 0..count {
        let mem = random_memory(&instructions[..], rng);
        let mut z80_hardware = random_z80_hardware(rng);
        z80_hardware.registers[PCL as usize] = 0;
        z80_hardware.registers[PCH as usize] = 0;
        
        let dir = TempDir::new("attalus_tmp")?;
        let file_path = dir.path().join("core.z80");
        write_core(&file_path, &z80_hardware, &mem);
        Command::new("ls")
            .current_dir(dir.path())
            .arg("-l")
            .arg("core.z80")
            .spawn()?;
        std::thread::sleep(std::time::Duration::from_millis(2));
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
            child_stdin.write_all("quit\n".as_bytes())?;
        }
        wait_for_exit(&mut child)?;
        let (z80sim_hw, z80sim_mem) = read_core(&file_path)?;
        let mut z80x = Z80X {
            z80_hardware: z80_hardware.clone(),
            memory_map_hw: SimpleMemoryMapperHardware {
                mem: mem
            }
        };
        execute_loop(&mut z80x, 1);
        let attalus_hw = z80x.z80_hardware;
        let attalus_mem = z80x.memory_map_hw.mem;
        let attalus_mem_slice: &[u8] = &attalus_mem;
        let z80sim_mem_slice: &[u8] = &z80sim_mem;
        if !z80_same_state(&attalus_hw, &z80sim_hw) || &attalus_mem_slice != &z80sim_mem_slice {
            return Ok(
                TestFailed(
                    TestFailure {
                        mnemonics: instruction_sequence.mnemonics.clone(),
                        original_hw: z80_hardware,
                        original_mem: mem,
                        z80sim_hw: z80sim_hw,
                        z80sim_mem: z80sim_mem,
                        attalus_hw: attalus_hw,
                        attalus_mem: attalus_mem,
                    }
                )
            );
        }
    }
    return Ok(TestOk);
}

pub fn test_against<R: Rng>(count: usize, depth: usize, rng: &mut R) -> Result<TestResult, TestAgainstError> {
    let instructions = generate_instructions(10, rng);
    let all_instructions = generate_instructions_sequence(&instructions, count, depth, rng);
    for inst in all_instructions.iter() {
        match test_instruction_sequence(inst, 3, rng)? {
            TestOk => continue,
            x => return Ok(x),
        }
    }
    return Ok(TestOk);
}
