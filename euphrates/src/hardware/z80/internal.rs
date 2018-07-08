use std::fmt;

use impler::Impl;

use super::*;

/// Carry flag
pub const CF: u8 = 0x01;

/// Subtraction flag
pub const NF: u8 = 0x02;

/// Parity/Overflow flag
pub const PF: u8 = 0x04;

/// Undocumented flag
pub const XF: u8 = 0x08;

/// Half carry flag
pub const HF: u8 = 0x10;

/// Undocumented flag
pub const YF: u8 = 0x20;

/// Zero flag
pub const ZF: u8 = 0x40;

/// Sign flag
pub const SF: u8 = 0x80;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Prefix {
    NoPrefix,
    Cb,
    Ed,
    Dd,
    Fd,
    DdCb,
    FdCb,
    Halt,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum InterruptStatus {
    /// If we're in the middle of emulation, no need to check interrupts before
    /// the next instruction.
    NoCheck,

    /// Check interrupts before executing another instruction.
    Check,

    /// The last instruction executed was Ei, and this is how many cycles were
    /// on the clock when it finished. Don't check interrupts now, but do check
    /// them after the next instruction.
    Ei(u64),
}

impl Default for InterruptStatus {
    #[inline]
    fn default() -> Self {
        InterruptStatus::NoCheck
    }
}

/// A wrapper object implementing `Display`, so you can `format` any type
/// implementing `Z80Internal`.
///
/// ```
/// use euphrates::hardware::z80::{Z80Display, Z80State};
/// let z: Z80State = Default::default();
/// format!("{}", Z80Display(&z));
/// ```
pub struct Z80Display<'a>(pub &'a dyn Z80Internal);

impl<'a> fmt::Display for Z80Display<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Reg16::*;
        use self::Reg8::*;
        format_args!(
            "SZYHXPNC  A   BC   DE   HL   IX   IY   SP   PC\n\
             {:0>8b} {:0>2X} {:0>4X} {:0>4X} {:0>4X} {:0>4X} {:0>4X} {:0>4X} {:0>4X}\n",
            self.0.reg8(F),
            self.0.reg8(A),
            self.0.reg16(BC),
            self.0.reg16(DE),
            self.0.reg16(HL),
            self.0.reg16(IX),
            self.0.reg16(IY),
            self.0.reg16(SP),
            self.0.reg16(PC),
        ).fmt(f)
    }
}

pub trait Z80Internal {
    fn cycles(&self) -> u64;
    fn set_cycles(&mut self, _: u64);
    fn reg8(&self, reg8: Reg8) -> u8;
    fn set_reg8(&mut self, reg8: Reg8, x: u8);
    fn reg16(&self, reg16: Reg16) -> u16;
    fn set_reg16(&mut self, reg16: Reg16, x: u16);
    fn halted(&self) -> bool;
    fn set_halted(&mut self, _: bool);
    fn iff1(&self) -> bool;
    fn set_iff1(&mut self, _: bool);
    fn iff2(&self) -> bool;
    fn set_iff2(&mut self, _: bool);
    fn interrupt_mode(&self) -> InterruptMode;
    fn set_interrupt_mode(&mut self, _: InterruptMode);

    fn prefix(&self) -> Prefix;
    fn set_prefix(&mut self, prefix: Prefix);

    fn interrupt_status(&self) -> InterruptStatus;
    fn set_interrupt_status(&mut self, interrupt_status: InterruptStatus);

    /// Increment the Z80's `cycles` by `x`.
    #[inline]
    fn inc_cycles(&mut self, x: u64) {
        let c = self.cycles();
        self.set_cycles(c + x);
    }

    /// Increment the lower 7 bits of the R register by the given amount,
    /// maintaining the high bit.
    #[inline]
    fn inc_r(&mut self, x: u8) {
        let r = self.reg8(Reg8::R);
        let ir = r.wrapping_add(x) & 0x7F;
        self.set_reg8(Reg8::R, ir | (r & 0x80));
    }

    /// Are these bits set in the F register?
    #[inline]
    fn is_set_flag(&self, x: u8) -> bool {
        (self.reg8(Reg8::F) & x) == x
    }

    /// Set the bits in the F register that are set in `x`.
    #[inline]
    fn set_flag(&mut self, x: u8) {
        let f = self.reg8(Reg8::F);
        self.set_reg8(Reg8::F, f | x);
    }

    /// Clear the bits in the F register that are set in `x`.
    #[inline]
    fn clear_flag(&mut self, x: u8) {
        let f = self.reg8(Reg8::F);
        self.set_reg8(Reg8::F, f & !x);
    }

    /// Set or clear the bits in the F register that are set in `x` according to
    /// whether `y` is true.
    #[inline]
    fn set_flag_by(&mut self, x: u8, y: bool) {
        if y {
            self.set_flag(x);
        } else {
            self.clear_flag(x);
        }
    }

    /// Set or clear the SF flag according to whether the sign bit of `x` is set.
    #[inline]
    fn set_sign(&mut self, x: u8) {
        if x & 0x80 != 0 {
            self.set_flag(SF);
        } else {
            self.clear_flag(SF);
        }
    }

    /// Set or clear the ZF flag according to whether `x` is zero.
    #[inline]
    fn set_zero(&mut self, x: u8) {
        if x == 0 {
            self.set_flag(ZF);
        } else {
            self.clear_flag(ZF);
        }
    }

    /// Set or clear the PF flag according to whether `x` has even parity.
    #[inline]
    fn set_parity(&mut self, x: u8) {
        if x.count_ones() % 2 == 0 {
            self.set_flag(PF);
        } else {
            self.clear_flag(PF);
        }
    }

    #[inline]
    fn state(&self) -> Z80State {
        let mut state = Z80State::default();
        transfer_state(&mut state, self);
        state
    }
}

/// Transfer all registers from `source` into `dest`.
pub fn transfer_state<Z1, Z2>(dest: &mut Z1, source: &Z2)
where
    Z1: Z80Internal + ?Sized,
    Z2: Z80Internal + ?Sized,
{
    use self::Reg16::*;
    use self::Reg8::*;
    dest.set_reg16(AF, source.reg16(AF));
    dest.set_reg16(BC, source.reg16(BC));
    dest.set_reg16(DE, source.reg16(DE));
    dest.set_reg16(HL, source.reg16(HL));
    dest.set_reg16(IX, source.reg16(IX));
    dest.set_reg16(IY, source.reg16(IY));
    dest.set_reg16(SP, source.reg16(SP));
    dest.set_reg16(PC, source.reg16(PC));
    dest.set_reg16(AF0, source.reg16(AF0));
    dest.set_reg16(BC0, source.reg16(BC0));
    dest.set_reg16(DE0, source.reg16(DE0));
    dest.set_reg16(HL0, source.reg16(HL0));
    dest.set_reg8(I, source.reg8(I));
    dest.set_reg8(R, source.reg8(R));
    dest.set_halted(source.halted());
    dest.set_iff1(source.iff1());
    dest.set_iff2(source.iff2());
    dest.set_interrupt_mode(source.interrupt_mode());
    dest.set_prefix(source.prefix());
    dest.set_interrupt_status(source.interrupt_status());
}

pub struct Z80InternalImpl;

impl<T> Z80Internal for T
where
    T: Impl<Z80InternalImpl> + ?Sized,
    T::Impler: Z80Internal,
{
    #[inline]
    fn cycles(&self) -> u64 {
        self.make().cycles()
    }

    #[inline]
    fn set_cycles(&mut self, x: u64) {
        self.make_mut().set_cycles(x)
    }

    #[inline]
    fn reg8(&self, reg8: Reg8) -> u8 {
        self.make().reg8(reg8)
    }

    #[inline]
    fn set_reg8(&mut self, reg8: Reg8, x: u8) {
        self.make_mut().set_reg8(reg8, x)
    }

    #[inline]
    fn reg16(&self, reg16: Reg16) -> u16 {
        self.make().reg16(reg16)
    }

    #[inline]
    fn set_reg16(&mut self, reg16: Reg16, x: u16) {
        self.make_mut().set_reg16(reg16, x)
    }

    #[inline]
    fn halted(&self) -> bool {
        self.make().halted()
    }

    #[inline]
    fn set_halted(&mut self, x: bool) {
        self.make_mut().set_halted(x)
    }

    #[inline]
    fn iff1(&self) -> bool {
        self.make().iff1()
    }

    #[inline]
    fn set_iff1(&mut self, x: bool) {
        self.make_mut().set_iff1(x)
    }

    #[inline]
    fn iff2(&self) -> bool {
        self.make().iff2()
    }

    #[inline]
    fn set_iff2(&mut self, x: bool) {
        self.make_mut().set_iff2(x)
    }

    #[inline]
    fn interrupt_mode(&self) -> InterruptMode {
        self.make().interrupt_mode()
    }

    #[inline]
    fn set_interrupt_mode(&mut self, x: InterruptMode) {
        self.make_mut().set_interrupt_mode(x)
    }

    #[inline]
    fn prefix(&self) -> Prefix {
        self.make().prefix()
    }

    #[inline]
    fn set_prefix(&mut self, prefix: Prefix) {
        self.make_mut().set_prefix(prefix)
    }

    #[inline]
    fn interrupt_status(&self) -> InterruptStatus {
        self.make().interrupt_status()
    }

    #[inline]
    fn set_interrupt_status(&mut self, interrupt_status: InterruptStatus) {
        self.make_mut().set_interrupt_status(interrupt_status)
    }

    #[inline]
    fn inc_cycles(&mut self, x: u64) {
        self.make_mut().inc_cycles(x)
    }

    #[inline]
    fn inc_r(&mut self, x: u8) {
        self.make_mut().inc_r(x)
    }

    #[inline]
    fn is_set_flag(&self, x: u8) -> bool {
        self.make().is_set_flag(x)
    }

    #[inline]
    fn set_flag(&mut self, x: u8) {
        self.make_mut().set_flag(x)
    }

    #[inline]
    fn clear_flag(&mut self, x: u8) {
        self.make_mut().clear_flag(x)
    }

    #[inline]
    fn set_flag_by(&mut self, x: u8, y: bool) {
        self.make_mut().set_flag_by(x, y)
    }

    #[inline]
    fn set_sign(&mut self, x: u8) {
        self.make_mut().set_sign(x)
    }

    #[inline]
    fn set_zero(&mut self, x: u8) {
        self.make_mut().set_zero(x)
    }

    #[inline]
    fn set_parity(&mut self, x: u8) {
        self.make_mut().set_parity(x)
    }

    #[inline]
    fn state(&self) -> Z80State {
        self.make().state()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Z80State {
    pub cycles: u64,
    pub registers: [u16; 13],
    pub halted: bool,
    /// iff1 is bit 0; iff2 is bit 1
    pub iff: u8,
    pub prefix: Prefix,
    pub interrupt_status: InterruptStatus,
    pub interrupt_mode: InterruptMode,
}

impl Default for Z80State {
    fn default() -> Self {
        let mut z80 = Z80State {
            cycles: 0,
            registers: Default::default(),
            halted: false,
            iff: 0,
            prefix: Prefix::NoPrefix,
            interrupt_status: Default::default(),
            interrupt_mode: Default::default(),
        };
        z80.set_reg16(Reg16::IX, 0xFFFF);
        z80.set_reg16(Reg16::IY, 0xFFFF);
        z80
    }
}

impl fmt::Display for Z80State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Z80Display(self).fmt(f)
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
        use std::mem::transmute;
        let byte_array: &[u8; 26] = unsafe { transmute(&self.registers) };
        unsafe { *byte_array.get_unchecked(reg8 as usize) }
    }

    #[inline]
    fn set_reg8(&mut self, reg8: Reg8, x: u8) {
        use std::mem::transmute;
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
        self.iff & 1 != 0
    }

    #[inline]
    fn set_iff1(&mut self, x: bool) {
        if x {
            self.iff |= 1
        } else {
            self.iff &= 0xFE
        }
    }

    #[inline]
    fn iff2(&self) -> bool {
        self.iff & 2 != 0
    }

    #[inline]
    fn set_iff2(&mut self, x: bool) {
        if x {
            self.iff |= 2
        } else {
            self.iff &= 0xFD
        }
    }

    #[inline]
    fn interrupt_mode(&self) -> InterruptMode {
        self.interrupt_mode
    }

    #[inline]
    fn set_interrupt_mode(&mut self, x: InterruptMode) {
        self.interrupt_mode = x;
    }

    #[inline]
    fn prefix(&self) -> Prefix {
        self.prefix
    }

    #[inline]
    fn set_prefix(&mut self, prefix: Prefix) {
        self.prefix = prefix
    }

    #[inline]
    fn interrupt_status(&self) -> InterruptStatus {
        self.interrupt_status
    }

    #[inline]
    fn set_interrupt_status(&mut self, interrupt_status: InterruptStatus) {
        self.interrupt_status = interrupt_status
    }

    #[inline]
    fn state(&self) -> Z80State {
        self.clone()
    }
}
