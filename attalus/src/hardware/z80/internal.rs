use std::fmt;

use hardware::memory16::Memory16;
use utilities;

use super::*;

/// Carry flag
pub const CF: u8 = 1 << 0;

/// Subtraction flag
pub const NF: u8 = 1 << 1;

/// Parity/Overflow flag
pub const PF: u8 = 1 << 2;

/// Undocumented flag
pub const XF: u8 = 1 << 3;

/// Half carry flag
pub const HF: u8 = 1 << 4;

/// Undocumented flag
pub const YF: u8 = 1 << 5;

/// Zero flag
pub const ZF: u8 = 1 << 6;

/// Sign flag
pub const SF: u8 = 1 << 7;

/// A wrapper object implementing `Display`, so you can `format` any type
/// implementing `Z80Internal`.
///
/// ```
/// use attalus::hardware::z80::{Z80Display, Z80State};
/// let z: Z80State = Default::default();
/// format!("{}", Z80Display(&z));
/// ```
pub struct Z80Display<'a>(pub &'a Z80Internal);

impl<'a> fmt::Display for Z80Display<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Reg16::*;
        use self::Reg8::*;
        write!(
            f,
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
        )
    }
}

pub trait Z80Internal {
    fn cycles(&self) -> u64;
    fn set_cycles(&mut self, u64);
    fn reg8(&self, reg8: Reg8) -> u8;
    fn set_reg8(&mut self, reg8: Reg8, x: u8);
    fn reg16(&self, reg16: Reg16) -> u16;
    fn set_reg16(&mut self, reg16: Reg16, x: u16);
    fn halted(&self) -> bool;
    fn set_halted(&mut self, bool);
    fn iff1(&self) -> bool;
    fn set_iff1(&mut self, bool);
    fn iff2(&self) -> bool;
    fn set_iff2(&mut self, bool);
    fn interrupt_mode(&self) -> InterruptMode;
    fn set_interrupt_mode(&mut self, InterruptMode);

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
    fn z80_state(&self) -> Z80State {
        let mut state = Z80State::default();
        transfer_state(&mut state, self);
        state
    }
}

/// Copy all registers and state from `source` to `dest`.
pub fn transfer_state<Z1, Z2>(dest: &mut Z1, source: &Z2)
where
    Z1: Z80Internal + ?Sized,
    Z2: Z80Internal + ?Sized,
{
    use self::Reg8::*;
    use self::Reg16::*;
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
}

pub trait Z80InternalImpler<S>
where
    S: ?Sized,
{
    fn cycles(&S) -> u64;
    fn set_cycles(&mut S, u64);
    fn reg8(&S, reg8: Reg8) -> u8;
    fn set_reg8(&mut S, reg8: Reg8, x: u8);
    fn reg16(&S, reg16: Reg16) -> u16;
    fn set_reg16(&mut S, reg16: Reg16, x: u16);
    fn halted(&S) -> bool;
    fn set_halted(&mut S, bool);
    fn iff1(&S) -> bool;
    fn set_iff1(&mut S, bool);
    fn iff2(&S) -> bool;
    fn set_iff2(&mut S, bool);
    fn interrupt_mode(&S) -> InterruptMode;
    fn set_interrupt_mode(&mut S, InterruptMode);

    /// This exists in `Z80InternalImpler` in case implers have a more efficient
    /// implementation than the default one in `Z80Internal`.
    ///
    /// But it has a default implementation in case they do not.
    #[inline]
    fn z80_state(s: &S) -> Z80State {
        // Unfortunately since we don't have a bound of `Z80Internal` on `S`, we
        // can't just call `transfer_state`.
        use self::Reg8::*;
        use self::Reg16::*;
        let mut dest = Z80State::default();
        dest.set_reg16(AF, Self::reg16(s, AF));
        dest.set_reg16(BC, Self::reg16(s, BC));
        dest.set_reg16(DE, Self::reg16(s, DE));
        dest.set_reg16(HL, Self::reg16(s, HL));
        dest.set_reg16(IX, Self::reg16(s, IX));
        dest.set_reg16(IY, Self::reg16(s, IY));
        dest.set_reg16(SP, Self::reg16(s, SP));
        dest.set_reg16(PC, Self::reg16(s, PC));
        dest.set_reg16(AF0, Self::reg16(s, AF0));
        dest.set_reg16(BC0, Self::reg16(s, BC0));
        dest.set_reg16(DE0, Self::reg16(s, DE0));
        dest.set_reg16(HL0, Self::reg16(s, HL0));
        dest.set_reg8(I, Self::reg8(s, I));
        dest.set_reg8(R, Self::reg8(s, R));
        dest.set_halted(Self::halted(s));
        dest.set_iff1(Self::iff1(s));
        dest.set_iff2(Self::iff2(s));
        dest.set_interrupt_mode(Self::interrupt_mode(s));
        dest
    }
}

pub trait Z80InternalImpl {
    type Impler: Z80InternalImpler<Self>;
}

impl<S> Z80Internal for S
where
    S: Z80InternalImpl + ?Sized,
{
    #[inline]
    fn cycles(&self) -> u64 {
        S::Impler::cycles(self)
    }

    #[inline]
    fn set_cycles(&mut self, x: u64) {
        S::Impler::set_cycles(self, x);
    }

    #[inline]
    fn reg8(&self, reg8: Reg8) -> u8 {
        S::Impler::reg8(self, reg8)
    }

    #[inline]
    fn set_reg8(&mut self, reg8: Reg8, x: u8) {
        S::Impler::set_reg8(self, reg8, x)
    }

    #[inline]
    fn reg16(&self, reg16: Reg16) -> u16 {
        S::Impler::reg16(self, reg16)
    }

    #[inline]
    fn set_reg16(&mut self, reg16: Reg16, x: u16) {
        S::Impler::set_reg16(self, reg16, x)
    }

    #[inline]
    fn halted(&self) -> bool {
        S::Impler::halted(&self)
    }

    #[inline]
    fn set_halted(&mut self, x: bool) {
        S::Impler::set_halted(self, x)
    }

    #[inline]
    fn iff1(&self) -> bool {
        S::Impler::iff1(self)
    }

    #[inline]
    fn set_iff1(&mut self, x: bool) {
        S::Impler::set_iff1(self, x)
    }

    #[inline]
    fn iff2(&self) -> bool {
        S::Impler::iff2(self)
    }

    #[inline]
    fn set_iff2(&mut self, x: bool) {
        S::Impler::set_iff2(self, x)
    }

    #[inline]
    fn interrupt_mode(&self) -> InterruptMode {
        S::Impler::interrupt_mode(self)
    }

    #[inline]
    fn set_interrupt_mode(&mut self, x: InterruptMode) {
        S::Impler::set_interrupt_mode(self, x)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Address<T>(pub T);

impl fmt::Display for Address<Reg16> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = format!("({})", self.0);
        f.pad(&s)
    }
}

impl fmt::Display for Address<u16> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = format!("({:0<4X})", self.0);
        f.pad(&s)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Shift(pub Reg16, pub i8);

impl fmt::Display for Shift {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = format!("({}{:<+07X})", self.0, self.1);
        f.pad(&s)
    }
}

/// An aspect of the Z80 that we can view, like a register or a memory address.
///
/// This trait (and `Changeable`) exists so that we may implement an instruction
/// like `ld x, y` with a single generic function, although `x` and `y` may be
/// memory addresses or registers.
pub trait Viewable<Output, Z: ?Sized>: Copy {
    fn view(self, z: &mut Z) -> Output;
}

/// An aspect of the Z80 that we can change, like a register or a memory address.
///
/// This trait (and `Viewable`) exists so that we may implement an instruction
/// like `ld x, y` with a single generic function, although `x` and `y` may be
/// memory addresses or registers.
pub trait Changeable<Output, Z: ?Sized>: Viewable<Output, Z> {
    fn change(self, z: &mut Z, x: Output);
}

impl<Z: ?Sized> Viewable<u8, Z> for u8 {
    #[inline]
    fn view(self, _z: &mut Z) -> u8 {
        self
    }
}

impl<Z: ?Sized> Viewable<u16, Z> for u16 {
    #[inline]
    fn view(self, _z: &mut Z) -> u16 {
        self
    }
}

impl<Z> Viewable<u8, Z> for Reg8
where
    Z: Z80Internal + ?Sized,
{
    #[inline]
    fn view(self, z: &mut Z) -> u8 {
        z.reg8(self)
    }
}

impl<Z> Changeable<u8, Z> for Reg8
where
    Z: Z80Internal + ?Sized,
{
    #[inline]
    fn change(self, z: &mut Z, x: u8) {
        z.set_reg8(self, x);
    }
}

impl<Z> Viewable<u16, Z> for Reg16
where
    Z: Z80Internal + ?Sized,
{
    #[inline]
    fn view(self, z: &mut Z) -> u16 {
        z.reg16(self)
    }
}

impl<Z> Changeable<u16, Z> for Reg16
where
    Z: Z80Internal + ?Sized,
{
    #[inline]
    fn change(self, z: &mut Z, x: u16) {
        z.set_reg16(self, x);
    }
}

impl<Z> Viewable<u16, Z> for Address<Reg16>
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    #[inline]
    fn view(self, z: &mut Z) -> u16 {
        let addr = self.0.view(z);
        let lo = z.read(addr);
        let hi = z.read(addr.wrapping_add(1));
        utilities::to16(lo, hi)
    }
}

impl<Z> Changeable<u16, Z> for Address<Reg16>
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    #[inline]
    fn change(self, z: &mut Z, x: u16) {
        let addr = self.0.view(z);
        let (lo, hi) = utilities::to8(x);
        z.write(addr, lo);
        z.write(addr.wrapping_add(1), hi);
    }
}

impl<Z> Viewable<u8, Z> for Address<Reg16>
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    #[inline]
    fn view(self, z: &mut Z) -> u8 {
        let addr = self.0.view(z);
        z.read(addr)
    }
}

impl<Z> Changeable<u8, Z> for Address<Reg16>
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    #[inline]
    fn change(self, z: &mut Z, x: u8) {
        let addr = self.0.view(z);
        z.write(addr, x);
    }
}

impl<Z> Viewable<u16, Z> for Address<u16>
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    #[inline]
    fn view(self, z: &mut Z) -> u16 {
        let addr = self.0;
        let lo = z.read(addr);
        let hi = z.read(addr.wrapping_add(1));
        utilities::to16(lo, hi)
    }
}

impl<Z> Changeable<u16, Z> for Address<u16>
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    #[inline]
    fn change(self, z: &mut Z, x: u16) {
        let addr = self.0;
        let (lo, hi) = utilities::to8(x);
        z.write(addr, lo);
        z.write(addr.wrapping_add(1), hi);
    }
}

impl<Z> Viewable<u8, Z> for Address<u16>
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    #[inline]
    fn view(self, z: &mut Z) -> u8 {
        z.read(self.0)
    }
}

impl<Z> Changeable<u8, Z> for Address<u16>
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    #[inline]
    fn change(self, z: &mut Z, x: u8) {
        z.write(self.0, x);
    }
}

impl<Z> Viewable<u8, Z> for Shift
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    #[inline]
    fn view(self, z: &mut Z) -> u8 {
        let addr = self.0.view(z).wrapping_add(self.1 as i16 as u16);
        Address(addr).view(z)
    }
}

impl<Z> Changeable<u8, Z> for Shift
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    #[inline]
    fn change(self, z: &mut Z, x: u8) {
        let addr = self.0.view(z).wrapping_add(self.1 as i16 as u16);
        Address(addr).change(z, x);
    }
}

impl<Z> Viewable<bool, Z> for ConditionCode
where
    Z: Z80Internal + ?Sized,
{
    #[inline]
    fn view(self, z: &mut Z) -> bool {
        let f = z.reg8(Reg8::F);
        match self {
            ConditionCode::NZcc => f & ZF == 0,
            ConditionCode::Zcc => f & ZF != 0,
            ConditionCode::NCcc => f & CF == 0,
            ConditionCode::Ccc => f & CF != 0,
            ConditionCode::POcc => f & PF == 0,
            ConditionCode::PEcc => f & PF != 0,
            ConditionCode::Pcc => f & SF == 0,
            ConditionCode::Mcc => f & SF != 0,
        }
    }
}
