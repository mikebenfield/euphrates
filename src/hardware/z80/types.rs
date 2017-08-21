// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use std;

use ::bits::*;

use ::hardware::memory_map::MemoryMap;
use ::hardware::io::Io;

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
    BC, DE, AF, HL,
    BC0, DE0, AF0, HL0,
    IX, IY,
    SP, PC
}

pub use self::Reg16::*;

#[derive(Clone, Copy, Debug)]
pub enum ConditionCode {
    NZcc, Zcc, NCcc, Ccc, POcc, PEcc, Pcc, Mcc
}

pub use self::ConditionCode::*;

// #[derive(Clone, Copy, Debug, Default)]
// pub enum Flag {
//     C, N, P, X, H, Y, Z80, S
// }

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
pub struct Z80<I: Io> {
    pub io: I,
    pub halted: bool,
    pub cycles: u64,
    pub address: u16,
    pub data: u8,

    /// Represents the iff1 flag, determining whether maskable interrupts are
    /// accepted.
    ///
    /// The Z80 `ei` instruction is supposed to set iff1, but then interrupts
    /// aren't supposed to actually be accepted until after the following
    /// instruction. To emulate this, my `ei` implementation sets the `iff1`
    /// field to the current value of `cycles`. Then when an interrupt is
    /// desired, the function `maskable_interrupt` first checks to see if
    /// `cycles` is larger than `iff1`.
    pub iff1: u64,
    pub iff2: bool,
    pub interrupt_mode: InterruptMode,
    registers: [u16; 13],
}

impl<I> Z80<I>
where I: Io {
    pub fn new(io: I) -> Z80<I> {
        let mut registers = [0u16; 13];
        // according to Young 2.4 these are the power on defaults
        registers[AF as usize] = 0xFFFF;
        registers[SP as usize] = 0xFFFF;
        Z80 {
            io: io,
            halted: false,
            cycles: 0,
            address: 0,
            data: 0,
            iff1: 0xFFFFFFFFFFFFFFFF,
            iff2: false,
            interrupt_mode: Im0,
            registers: registers,
        }
    }
}

impl<I: Io> std::fmt::Display for Z80<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "SZYHXPNC  A   BC   DE   HL   IX   IY   SP   PC\n\
             {:0>8b} {:0>2X} {:0>4X} {:0>4X} {:0>4X} {:0>4X} {:0>4X} {:0>4X} {:0>4X}\n",
            F.get(self),
            A.get(self),
            BC.get(self),
            DE.get(self),
            HL.get(self),
            IX.get(self),
            IY.get(self),
            SP.get(self),
            PC.get(self)
        )
    }
}

pub trait Z80Emulator<I: Io> {
    fn run(&mut self, &mut Z80<I>, cycles: u64);
}

pub fn inc_r<I: Io>(z: &mut Z80<I>) {
    let r = R.get(z);
    let ir = r.wrapping_add(1) & 0x7F;
    R.set(z, ir | (r & 0x80));
}

pub trait Gettable<Output>: std::fmt::Debug + Copy {
    fn get<I: Io>(self, z: &Z80<I>) -> Output;
}

pub trait Settable<Output>: Gettable<Output> {
    fn set<I: Io>(self, z: &mut Z80<I>, x: Output);
}

impl Gettable<u8> for u8 {
    fn get<I: Io>(self, _: &Z80<I>) -> u8 {
        self
    }
}

impl Gettable<u16> for u16 {
    fn get<I: Io>(self, _: &Z80<I>) -> u16 {
        self
    }
}

impl Gettable<u8> for Reg8 {
    fn get<I: Io>(self, z: &Z80<I>) -> u8 {
        let byte_array: &[u8; 26] =
            unsafe {
                std::mem::transmute(&z.registers)
            };
        byte_array[self as usize]
    }
}

impl Settable<u8> for Reg8 {
    fn set<I: Io>(self, z: &mut Z80<I>, x: u8) {
        let byte_array: &mut [u8; 26] =
            unsafe {
                std::mem::transmute(&mut z.registers)
            };
        byte_array[self as usize] = x
    }
}

impl Gettable<u16> for Reg16 {
    fn get<I: Io>(self, z: &Z80<I>) -> u16 {
        z.registers[self as usize]
    }
}

impl Settable<u16> for Reg16 {
    fn set<I: Io>(self, z: &mut Z80<I>, x: u16) {
        z.registers[self as usize] = x
    }
}

impl Gettable<u16> for Address<Reg16> {
    fn get<I: Io>(self, z: &Z80<I>) -> u16 {
        let addr = self.0.get(z);
        let lo = z.io.mem().read(addr);
        let hi = z.io.mem().read(addr.wrapping_add(1));
        to16(lo, hi)
    }
}

impl Settable<u16> for Address<Reg16> {
    fn set<I: Io>(self, z: &mut Z80<I>, x: u16) {
        let addr = self.0.get(z);
        let (lo, hi) = to8(x);
        z.io.mem_mut().write(addr, lo);
        z.io.mem_mut().write(addr.wrapping_add(1), hi);
    }
}

impl Gettable<u8> for Address<Reg16> {
    fn get<I: Io>(self, z: &Z80<I>) -> u8 {
        let addr = self.0.get(z);
        z.io.mem().read(addr)
    }
}

impl Settable<u8> for Address<Reg16> {
    fn set<I: Io>(self, z: &mut Z80<I>, x: u8) {
        let addr = self.0.get(z);
        z.io.mem_mut().write(addr, x);
    }
}

impl Gettable<u16> for Address<u16> {
    fn get<I: Io>(self, z: &Z80<I>) -> u16 {
        let addr = self.0;
        let lo = z.io.mem().read(addr);
        let hi = z.io.mem().read(addr + 1);
        to16(lo, hi)
    }
}

impl Settable<u16> for Address<u16> {
    fn set<I: Io>(self, z: &mut Z80<I>, x: u16) {
        let addr = self.0;
        let (lo, hi) = to8(x);
        z.io.mem_mut().write(addr, lo);
        z.io.mem_mut().write(addr + 1, hi);
    }
}

impl Gettable<u8> for Address<u16> {
    fn get<I: Io>(self, z: &Z80<I>) -> u8 {
        z.io.mem().read(self.0)
    }
}

impl Settable<u8> for Address<u16> {
    fn set<I: Io>(self, z: &mut Z80<I>, x: u8) {
        z.io.mem_mut().write(self.0, x);
    }
}

impl Gettable<u8> for Shift {
    fn get<I: Io>(self, z: &Z80<I>) -> u8 {
        let addr = self.0.get(z).wrapping_add(self.1 as i16 as u16);
        Address(addr).get(z)
    }
}

impl Settable<u8> for Shift {
    fn set<I: Io>(self, z: &mut Z80<I>, x: u8) {
        let addr = self.0.get(z).wrapping_add(self.1 as i16 as u16);
        Address(addr).set(z, x);
    }
}

impl Gettable<bool> for ConditionCode {
    fn get<I: Io>(self, z: &Z80<I>) -> bool {
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
