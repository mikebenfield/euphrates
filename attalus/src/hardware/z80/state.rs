use std::fmt;
use std::mem::transmute;

use super::*;
use super::Reg16::*;
use super::Reg8::*;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[repr(C)]
pub struct Z80State {
    pub cycles: u64,
    pub registers: [u16; 13],
    pub halted: bool,
    pub iff1: bool,
    pub iff2: bool,
    pub interrupt_mode: InterruptMode,
}

impl fmt::Display for Z80State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "SZYHXPNC  A   BC   DE   HL   IX   IY   SP   PC\n\
             {:0>8b} {:0>2X} {:0>4X} {:0>4X} {:0>4X} {:0>4X} {:0>4X} {:0>4X} {:0>4X}\n",
            self.reg8(F),
            self.reg8(A),
            self.reg16(BC),
            self.reg16(DE),
            self.reg16(HL),
            self.reg16(IX),
            self.reg16(IY),
            self.reg16(SP),
            self.reg16(PC),
        )
    }
}

impl Z80Internal for Z80State {
    #[inline]
    fn cycles(&self) -> u64 {
        self.cycles
    }

    #[inline]
    fn set_cycles(&mut self, x: u64) {
        self.cycles = x;
    }

    #[inline]
    fn reg8(&self, reg8: Reg8) -> u8 {
        let byte_array: &[u8; 26] = unsafe { transmute(&self.registers) };
        unsafe { *byte_array.get_unchecked(reg8 as usize) }
    }

    #[inline]
    fn set_reg8(&mut self, reg8: Reg8, x: u8) {
        let byte_array: &mut [u8; 26] = unsafe { transmute(&mut self.registers) };
        unsafe {
            *byte_array.get_unchecked_mut(reg8 as usize) = x;
        }
    }

    #[inline]
    fn reg16(&self, reg16: Reg16) -> u16 {
        unsafe { *self.registers.get_unchecked(reg16 as usize) }
    }

    #[inline]
    fn set_reg16(&mut self, reg16: Reg16, x: u16) {
        unsafe { *self.registers.get_unchecked_mut(reg16 as usize) = x }
    }

    #[inline]
    fn halted(&self) -> bool {
        self.halted
    }

    #[inline]
    fn set_halted(&mut self, x: bool) {
        self.halted = x;
    }

    #[inline]
    fn iff1(&self) -> bool {
        self.iff1
    }

    #[inline]
    fn set_iff1(&mut self, x: bool) {
        self.iff1 = x;
    }

    #[inline]
    fn iff2(&self) -> bool {
        self.iff2
    }

    #[inline]
    fn set_iff2(&mut self, x: bool) {
        self.iff2 = x;
    }

    #[inline]
    fn interrupt_mode(&self) -> InterruptMode {
        self.interrupt_mode
    }

    #[inline]
    fn set_interrupt_mode(&mut self, x: InterruptMode) {
        self.interrupt_mode = x;
    }
}

impl<S> Z80InternalImpler<S> for Z80State
where
    S: ?Sized + AsRef<Z80State> + AsMut<Z80State>,
{
    #[inline]
    fn cycles(s: &S) -> u64 {
        s.as_ref().cycles()
    }

    #[inline]
    fn set_cycles(s: &mut S, x: u64) {
        s.as_mut().set_cycles(x)
    }

    #[inline]
    fn reg8(s: &S, reg8: Reg8) -> u8 {
        s.as_ref().reg8(reg8)
    }

    #[inline]
    fn set_reg8(s: &mut S, reg8: Reg8, x: u8) {
        s.as_mut().set_reg8(reg8, x)
    }

    #[inline]
    fn reg16(s: &S, reg16: Reg16) -> u16 {
        s.as_ref().reg16(reg16)
    }

    #[inline]
    fn set_reg16(s: &mut S, reg16: Reg16, x: u16) {
        s.as_mut().set_reg16(reg16, x)
    }

    #[inline]
    fn halted(s: &S) -> bool {
        s.as_ref().halted
    }

    #[inline]
    fn set_halted(s: &mut S, x: bool) {
        s.as_mut().set_halted(x)
    }

    #[inline]
    fn iff1(s: &S) -> bool {
        s.as_ref().iff1()
    }

    #[inline]
    fn set_iff1(s: &mut S, x: bool) {
        s.as_mut().set_iff1(x)
    }

    #[inline]
    fn iff2(s: &S) -> bool {
        s.as_ref().iff2()
    }

    #[inline]
    fn set_iff2(s: &mut S, x: bool) {
        s.as_mut().set_iff2(x)
    }

    #[inline]
    fn interrupt_mode(s: &S) -> InterruptMode {
        s.as_ref().interrupt_mode()
    }

    #[inline]
    fn set_interrupt_mode(s: &mut S, x: InterruptMode) {
        s.as_mut().set_interrupt_mode(x)
    }

    #[inline]
    fn z80_state(s: &S) -> Z80State {
        *s.as_ref()
    }
}

pub trait Savable {
    fn save(&self) -> Z80State;
}

pub trait Restorable {
    fn restore(&Z80State) -> Self;
}

impl Savable for Z80State {
    fn save(&self) -> Z80State {
        *self
    }
}

impl Restorable for Z80State {
    fn restore(t: &Z80State) -> Self {
        *t
    }
}
