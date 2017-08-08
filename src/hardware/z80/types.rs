use std;

use bits::*;
use log::Log;

use hardware::memory_mapper::MemoryMapper;
use hardware::io::Io;

#[cfg(target_endian = "little")]
#[derive(Clone, Copy, Debug)]
pub enum Reg8 {
    C, B, E, D, F, A, L, H,
    C0, B0, E0, D0, F0, A0, L0, H0,
    IXL, IXH, IYL, IYH,
    SPL, SPH, PCL, PCH, I, R
}

#[cfg(target_endian = "big")]
#[derive(Clone, Copy, Debug)]
pub enum Reg8 {
    B, C, D, E, A, F, H, L,
    B0, C0, D0, E0, A0, F0, L0, H0,
    IXH, IXL, IYH, IYL,
    SPH, SPL, PCH, PCL, I, R
}

pub use self::Reg8::*;

#[derive(Clone, Copy, Debug)]
pub enum Reg16 {
    BC=0, DE=2, AF=4, HL=6,
    BC0=8, DE0=10, AF0=12, HL0=14,
    IX=16, IY=18,
    SP=20, PC=22
}

pub use self::Reg16::*;

#[derive(Clone, Copy, Debug)]
pub enum ConditionCode {
    NZcc, Zcc, NCcc, Ccc, POcc, PEcc, Pcc, Mcc
}

pub use self::ConditionCode::*;

pub const CF: u8 = 0;
pub const NF: u8 = 1;
pub const PF: u8 = 2;
pub const XF: u8 = 3;
pub const HF: u8 = 4;
pub const YF: u8 = 5;
pub const ZF: u8 = 6;
pub const SF: u8 = 7;

#[derive(Clone, Copy, Debug)]
pub enum InterruptMode {
    Im0, Im1, Im2
}

pub use self::InterruptMode::*;

impl Default for InterruptMode {
    fn default() -> InterruptMode { Im0 }
}

#[derive(Clone, Copy, Debug)]
pub struct Address<T>(pub T);

#[derive(Clone, Copy, Debug)]
pub struct Shift(pub Reg16, pub i8);

#[derive(Clone, Copy, Debug, Default)]
pub struct Z80Hardware {
    pub address: u16,
    pub data: u8,
    pub iff1: bool,
    pub iff2: bool,
    pub interrupt_mode: InterruptMode,
    pub registers: [u8; 26],
}

impl std::fmt::Display for Z80Hardware {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        fn get_register_value(z: &Z80Hardware, r: Reg16) -> u16 {
            let reff: &u8 = &z.registers[r as usize];
            unsafe {
                let reff2: &u16 = std::mem::transmute(reff);
                *reff2
            }
        }
        write!(
            f,
            "{{ AF = {:0>4X}, BC = {:0>4X}, DE = {:0>4X}, \
            HL = {:0>4X}, IX = {:0>4X}, IY = {:0>4X}, \
            SP = {:0>4X}, PC = {:0>4X} }}",
            get_register_value(self, AF),
            get_register_value(self, BC),
            get_register_value(self, DE),
            get_register_value(self, HL),
            get_register_value(self, IX),
            get_register_value(self, IY),
            get_register_value(self, SP),
            get_register_value(self, PC),
        )
    }
}

/// The Z80 needs access to two facilities of the larger system: IO, which in
/// this implementation it accesses through the Io trait, and memory, which in
/// this implementation it accesses through the MemoryMapper trait. In addition,
/// it needs access to a Log. All of these objects will be owned by an
/// EmulationManager. Finally, there needs to be a method to record how many
/// machine cycles have passed. All the Z80 functions will access this
/// functionality, and the Z80 itself, through this trait.
pub trait Z80: Log + Io + MemoryMapper {
    fn get_z80_hardware(&self) -> &Z80Hardware;
    fn get_mut_z80_hardware(&mut self) -> &mut Z80Hardware;
    fn advance_t_states(&mut self, count: u64);
    fn get_t_states(&self) -> u64;

    // options
    fn end_on_halt(&self) -> bool;
    fn use_r_register(&self) -> bool;
}


pub fn inc_r<Z: Z80>(z: &mut Z) {
    if z.use_r_register() {
        let r = R.get(z);
        let ir = r.wrapping_add(1) & 0x7F;
        R.set(z, ir | (r & 0x80));
    }
}

// impl std::fmt::Display for Z80 {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         fn get_register_value(z: &Z80Hardware, r: Reg16) -> u16 {
//             let reff: &u8 = &z.registers[r as usize];
//             unsafe {
//                 let reff2: &u16 = std::mem::transmute(reff);
//                 *reff2
//             }
//         }
//         write!(
//             f,
//             "{{ AF = {:0>4X}, BC = {:0>4X}, DE = {:0>4X}, \
//             HL = {:0>4X}, IX = {:0>4X}, IY = {:0>4X}, \
//             SP = {:0>4X}, PC = {:0>4X} }}",
//             self.get_z80_hardware().registers[AF],
//             self.get_z80_hardware().registers[BC],
//             self.get_z80_hardware().registers[DE],
//             self.get_z80_hardware().registers[HL],
//             self.get_z80_hardware().registers[IX],
//             self.get_z80_hardware().registers[IY],
//             self.get_z80_hardware().registers[SP],
//             self.get_z80_hardware().registers[PC],
//         )
//     }
// }

pub trait Gettable<Output>: std::fmt::Debug + Copy {
    fn get<Z: Z80>(self, z: &mut Z) -> Output;
}

pub trait Settable<Output>: Gettable<Output> {
    fn set<Z: Z80>(self, z: &mut Z, x: Output);
}

impl Gettable<u8> for u8 {
    fn get<Z: Z80>(self, _: &mut Z) -> u8 {
        self
    }
}

impl Gettable<u16> for u16 {
    fn get<Z: Z80>(self, _: &mut Z) -> u16 {
        self
    }
}

impl Gettable<u8> for Reg8 {
    fn get<Z: Z80>(self, z: &mut Z) -> u8 {
        z.get_z80_hardware().registers[self as usize]
    }
}

impl Settable<u8> for Reg8 {
    fn set<Z: Z80>(self, z: &mut Z, x: u8) {
        z.get_mut_z80_hardware().registers[self as usize] = x
    }
}

impl Gettable<u16> for Reg16 {
    fn get<Z: Z80>(self, z: &mut Z) -> u16 {
        let reff: &u8 = &z.get_z80_hardware().registers[self as usize];
        unsafe {
            let reff2: &u16 = std::mem::transmute(reff);
            *reff2
        }
    }
}

impl Settable<u16> for Reg16 {
    fn set<Z: Z80>(self, z: &mut Z, x: u16) {
        let reff: &mut u8 = &mut z.get_mut_z80_hardware().registers[self as usize];
        unsafe {
            let reff2: &mut u16 = std::mem::transmute(reff);
            *reff2 = x;
        }
    }
}

impl Gettable<u16> for Address<Reg16> {
    fn get<Z: Z80>(self, z: &mut Z) -> u16 {
        let addr = self.0.get(z);
        let lo = z.read(addr);
        let hi = z.read(addr + 1);
        to16(lo, hi)
    }
}

impl Settable<u16> for Address<Reg16> {
    fn set<Z: Z80>(self, z: &mut Z, x: u16) {
        let addr = self.0.get(z);
        let (lo, hi) = to8(x);
        z.write(addr, lo);
        z.write(addr + 1, hi);
    }
}

impl Gettable<u8> for Address<Reg16> {
    fn get<Z: Z80>(self, z: &mut Z) -> u8 {
        let addr = self.0.get(z);
        z.read(addr)
    }
}

impl Settable<u8> for Address<Reg16> {
    fn set<Z: Z80>(self, z: &mut Z, x: u8) {
        let addr = self.0.get(z);
        z.write(addr, x);
    }
}

impl Gettable<u16> for Address<u16> {
    fn get<Z: Z80>(self, z: &mut Z) -> u16 {
        let addr = self.0;
        let lo = z.read(addr);
        let hi = z.read(addr + 1);
        to16(lo, hi)
    }
}

impl Settable<u16> for Address<u16> {
    fn set<Z: Z80>(self, z: &mut Z, x: u16) {
        let addr = self.0;
        let (lo, hi) = to8(x);
        z.write(addr, lo);
        z.write(addr + 1, hi);
    }
}

impl Gettable<u8> for Address<u16> {
    fn get<Z: Z80>(self, z: &mut Z) -> u8 {
        z.read(self.0)
    }
}

impl Settable<u8> for Address<u16> {
    fn set<Z: Z80>(self, z: &mut Z, x: u8) {
        z.write(self.0, x);
    }
}

impl Gettable<u8> for Shift {
    fn get<Z: Z80>(self, z: &mut Z) -> u8 {
        let addr = self.0.get(z).wrapping_add(self.1 as i16 as u16);
        Address(addr).get(z)
    }
}

impl Settable<u8> for Shift {
    fn set<Z: Z80>(self, z: &mut Z, x: u8) {
        let addr = self.0.get(z).wrapping_add(self.1 as i16 as u16);
        Address(addr).set(z, x);
    }
}

impl Gettable<bool> for ConditionCode {
    fn get<Z: Z80>(self, z: &mut Z) -> bool {
        let f = F.get(z);
        match self {
            NZcc => f & (1 << ZF) == 0,
            Zcc  => f & (1 << ZF) != 0,
            NCcc => f & (1 << CF) == 0,
            Ccc => f & (1 << CF) != 0,
            POcc => f & (1 << PF) == 0,
            PEcc => f & (1 << PF) != 0,
            Pcc => f & (1 << SF) == 0,
            Mcc => f & (1 << SF) != 0,
        }
    }
}
