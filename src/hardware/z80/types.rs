// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use std;

use ::bits::*;

use ::message::{Receiver, Sender};
use ::hardware::memory_map::MemoryMap;
use ::hardware::io::Io;

#[cfg(target_endian = "little")]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Reg8 {
    C, B, E, D, F, A, L, H,
    C0, B0, E0, D0, F0, A0, L0, H0,
    IXL, IXH, IYL, IYH,
    SPL, SPH, PCL, PCH, I, R
}

#[cfg(target_endian = "big")]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Reg8 {
    B, C, D, E, A, F, H, L,
    B0, C0, D0, E0, A0, F0, H0, L0,
    IXH, IXL, IYH, IYL,
    SPH, SPL, PCH, PCL, I, R
}

pub use self::Reg8::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Reg16 {
    BC, DE, AF, HL,
    BC0, DE0, AF0, HL0,
    IX, IY,
    SP, PC
}

pub use self::Reg16::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ConditionCode {
    NZcc, Zcc, NCcc, Ccc, POcc, PEcc, Pcc, Mcc
}

pub use self::ConditionCode::*;

bitflags! {
    pub struct Flags: u8 {
        const CF = 1 << 0;
        const NF = 1 << 1;
        const PF = 1 << 2;
        const XF = 1 << 3;
        const HF = 1 << 4;
        const YF = 1 << 5;
        const ZF = 1 << 6;
        const SF = 1 << 7;
    }
}

impl Flags {
    pub fn set_sign(&mut self, x: u8)
    {
        self.set(SF, x & 0x80 != 0);
    }

    pub fn set_zero(&mut self, x: u8)
    {
        self.set(ZF, x == 0);
    }

    pub fn set_parity(&mut self, x: u8)
    {
        let parity = x.count_ones() % 2 == 0;
        self.set(PF, parity);
    }

}

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

#[derive(Clone, Copy, Debug)]
pub struct Z80<I> {
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
    /// `cycles` is larger than `iff1`. `di` sets `iff1` to 0xFFFFFFFFFFFFFFFF.
    pub iff1: u64,
    pub iff2: bool,
    pub interrupt_mode: InterruptMode,
    pub registers: [u16; 13],
    id: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Opcode {
    OneByte([u8; 1]),
    TwoBytes([u8; 2]),
    ThreeBytes([u8; 3]),
    FourBytes([u8; 4]),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Z80Message {
    Reg8Changed {
        register: Reg8,
        old_value: u8,
        new_value: u8,
    },

    Reg16Changed {
        register: Reg16,
        old_value: u16,
        new_value: u16,
    },

    InstructionAtPc(u16),
    InstructionOpcode(Opcode),

    MaskableInterruptDenied,
    MaskableInterruptAllowed,
    NonmaskableInterrupt,
}

impl<I> Sender for Z80<I>
{
    type Message = Z80Message;

    fn id(&self) -> u32 { self.id }

    fn set_id(&mut self, id: u32) { self.id = id; }
}

impl<I: Default> Default for Z80<I> {
    fn default() -> Z80<I> {
        let io: I = I::default();
        Z80::new(io)
    }
}

impl<I> Z80<I> {
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
            id: 0,
        }
    }

    pub fn toggle_flags(&mut self, flags: Flags) {
        let mut f = Flags::from_bits_truncate(self.get_reg8(F));
        f.toggle(flags);
        self.set_reg8(F, f.bits());
    }

    pub fn get_reg8(&self, reg8: Reg8) -> u8 {
        let byte_array: &[u8; 26] =
            unsafe {
                std::mem::transmute(&self.registers)
            };
        byte_array[reg8 as usize]
    }

    pub fn set_reg8(&mut self, reg8: Reg8, x: u8) {
        let byte_array: &mut [u8; 26] =
            unsafe {
                std::mem::transmute(&mut self.registers)
            };
        byte_array[reg8 as usize] = x
    }

    pub fn get_reg16(&self, reg16: Reg16) -> u16 {
        self.registers[reg16 as usize]
    }

    pub fn set_reg16(&mut self, reg16: Reg16, x: u16) {
        self.registers[reg16 as usize] = x;
    }

    pub fn flags(&self) -> Flags {
        Flags::from_bits_truncate(self.get_reg8(F))
    }

    pub fn set_flags(&mut self, f: Flags) {
        self.set_reg8(F, f.bits());
    }
}

impl<I> std::fmt::Display for Z80<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "SZYHXPNC  A   BC   DE   HL   IX   IY   SP   PC\n\
             {:0>8b} {:0>2X} {:0>4X} {:0>4X} {:0>4X} {:0>4X} {:0>4X} {:0>4X} {:0>4X}\n",
            self.get_reg8(F),
            self.get_reg8(A),
            self.get_reg16(BC),
            self.get_reg16(DE),
            self.get_reg16(HL),
            self.get_reg16(IX),
            self.get_reg16(IY),
            self.get_reg16(SP),
            self.get_reg16(PC),
        )
    }
}

pub trait Z80Emulator<I, R>
where
    I: Io<R>,
    R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
{
    fn run(&mut self, receiver: &mut R, &mut Z80<I>, cycles: u64);
}

pub fn inc_r<I>(z: &mut Z80<I>) {
    let r = z.get_reg8(R);
    let ir = r.wrapping_add(1) & 0x7F;
    z.set_reg8(R, ir | (r & 0x80));
}

pub trait Gettable<Output>: std::fmt::Debug + Copy {
    fn get<I, R>(self, receiver: &mut R, z: &Z80<I>) -> Output
    where
        I: Io<R>,
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>;
}

pub trait Settable<Output>: Gettable<Output> {
    fn set<I, R>(self, receiver: &mut R, z: &mut Z80<I>, x: Output)
    where
        I: Io<R>,
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>;
}

impl Gettable<u8> for u8 {
    fn get<I, R>(self, _receiver: &mut R, _: &Z80<I>) -> u8
    where
        I: Io<R>,
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
    {
        self
    }
}

impl Gettable<u16> for u16 {
    fn get<I, R>(self, _receiver: &mut R, _: &Z80<I>) -> u16
    where
        I: Io<R>,
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
    {
        self
    }
}

impl Gettable<u8> for Reg8 {
    fn get<I, R>(self, _receiver: &mut R, z: &Z80<I>) -> u8
    where
        I: Io<R>,
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
    {
        z.get_reg8(self)
    }
}

impl Settable<u8> for Reg8 {
    fn set<I, R>(self, receiver: &mut R, z: &mut Z80<I>, x: u8)
    where
        I: Io<R>,
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
    {
        receiver.receive(
            z.id(),
            Z80Message::Reg8Changed{
                register: self,
                old_value: z.get_reg8(self),
                new_value: x,
            }
        );
        z.set_reg8(self, x);
    }
}

impl Gettable<u16> for Reg16 {
    fn get<I, R>(self, _receiver: &mut R, z: &Z80<I>) -> u16
    where
        I: Io<R>,
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
    {
        z.get_reg16(self)
    }
}

impl Settable<u16> for Reg16 {
    fn set<I, R>(self, receiver: &mut R, z: &mut Z80<I>, x: u16)
    where
        I: Io<R>,
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
    {
        receiver.receive(
            z.id(),
            Z80Message::Reg16Changed{
                register: self,
                old_value: z.get_reg16(self),
                new_value: x,
            }
        );
        z.set_reg16(self, x);
    }
}

impl Gettable<u16> for Address<Reg16> {
    fn get<I, R>(self, receiver: &mut R, z: &Z80<I>) -> u16
    where
        I: Io<R>,
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
    {
        let addr = self.0.get(receiver, z);
        let lo = z.io.mem().read(receiver, addr);
        let hi = z.io.mem().read(receiver, addr.wrapping_add(1));
        to16(lo, hi)
    }
}

impl Settable<u16> for Address<Reg16> {
    fn set<I, R>(self, receiver: &mut R, z: &mut Z80<I>, x: u16)
    where
        I: Io<R>,
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
    {
        let addr = self.0.get(receiver, z);
        let (lo, hi) = to8(x);
        z.io.mem_mut().write(receiver, addr, lo);
        z.io.mem_mut().write(receiver, addr.wrapping_add(1), hi);
    }
}

impl Gettable<u8> for Address<Reg16> {
    fn get<I, R>(self, receiver: &mut R, z: &Z80<I>) -> u8
    where
        I: Io<R>,
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
    {
        let addr = self.0.get(receiver, z);
        z.io.mem().read(receiver, addr)
    }
}

impl Settable<u8> for Address<Reg16> {
    fn set<I, R>(self, receiver: &mut R, z: &mut Z80<I>, x: u8)
    where
        I: Io<R>,
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
    {
        let addr = self.0.get(receiver, z);
        z.io.mem_mut().write(receiver, addr, x);
    }
}

impl Gettable<u16> for Address<u16> {
    fn get<I, R>(self, receiver: &mut R, z: &Z80<I>) -> u16
    where
        I: Io<R>,
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
    {
        let addr = self.0;
        let lo = z.io.mem().read(receiver, addr);
        let hi = z.io.mem().read(receiver, addr.wrapping_add(1));
        to16(lo, hi)
    }
}

impl Settable<u16> for Address<u16> {
    fn set<I, R>(self, receiver: &mut R, z: &mut Z80<I>, x: u16)
    where
        I: Io<R>,
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
    {
        let addr = self.0;
        let (lo, hi) = to8(x);
        z.io.mem_mut().write(receiver, addr, lo);
        z.io.mem_mut().write(receiver, addr.wrapping_add(1), hi);
    }
}

impl Gettable<u8> for Address<u16> {
    fn get<I, R>(self, receiver: &mut R, z: &Z80<I>) -> u8
    where
        I: Io<R>,
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
    {
        z.io.mem().read(receiver, self.0)
    }
}

impl Settable<u8> for Address<u16> {
    fn set<I, R>(self, receiver: &mut R, z: &mut Z80<I>, x: u8)
    where
        I: Io<R>,
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
    {
        z.io.mem_mut().write(receiver, self.0, x);
    }
}

impl Gettable<u8> for Shift {
    fn get<I, R>(self, receiver: &mut R, z: &Z80<I>) -> u8
    where
        I: Io<R>,
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
    {
        let addr = self.0.get(receiver, z).wrapping_add(self.1 as i16 as u16);
        Address(addr).get(receiver, z)
    }
}

impl Settable<u8> for Shift {
    fn set<I, R>(self, receiver: &mut R, z: &mut Z80<I>, x: u8)
    where
        I: Io<R>,
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
    {
        let addr = self.0.get(receiver, z).wrapping_add(self.1 as i16 as u16);
        Address(addr).set(receiver, z, x);
    }
}

impl Gettable<bool> for ConditionCode {
    fn get<I, R>(self, _receiver: &mut R, z: &Z80<I>) -> bool
    where
        I: Io<R>,
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
    {
        let f = z.flags();
        match self {
            NZcc => !f.contains(ZF),
            Zcc => f.contains(ZF),
            NCcc => !f.contains(CF),
            Ccc => f.contains(CF),
            POcc => !f.contains(PF),
            PEcc => f.contains(PF),
            Pcc => !f.contains(SF),
            Mcc => f.contains(SF),
        }
    }
}
