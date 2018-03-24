use super::{InterruptMode, Reg16, Reg8};

pub trait T {
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
}

pub trait Impler<S>
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
}

pub trait Impl {
    type Impler: Impler<Self>;
}

impl<S> T for S
where
    S: Impl,
{
    #[inline]
    fn cycles(&self) -> u64 {
        <S::Impler as Impler<Self>>::cycles(self)
    }

    #[inline]
    fn set_cycles(&mut self, x: u64) {
        <S::Impler as Impler<Self>>::set_cycles(self, x);
    }

    #[inline]
    fn reg8(&self, reg8: Reg8) -> u8 {
        <S::Impler as Impler<Self>>::reg8(self, reg8)
    }

    #[inline]
    fn set_reg8(&mut self, reg8: Reg8, x: u8) {
        <S::Impler as Impler<Self>>::set_reg8(self, reg8, x)
    }

    #[inline]
    fn reg16(&self, reg16: Reg16) -> u16 {
        <S::Impler as Impler<Self>>::reg16(self, reg16)
    }

    #[inline]
    fn set_reg16(&mut self, reg16: Reg16, x: u16) {
        <S::Impler as Impler<Self>>::set_reg16(self, reg16, x)
    }

    #[inline]
    fn halted(&self) -> bool {
        <S::Impler as Impler<Self>>::halted(&self)
    }

    #[inline]
    fn set_halted(&mut self, x: bool) {
        <S::Impler as Impler<Self>>::set_halted(self, x)
    }

    #[inline]
    fn iff1(&self) -> bool {
        <S::Impler as Impler<Self>>::iff1(self)
    }

    #[inline]
    fn set_iff1(&mut self, x: bool) {
        <S::Impler as Impler<Self>>::set_iff1(self, x)
    }

    #[inline]
    fn iff2(&self) -> bool {
        <S::Impler as Impler<Self>>::iff2(self)
    }

    #[inline]
    fn set_iff2(&mut self, x: bool) {
        <S::Impler as Impler<Self>>::set_iff2(self, x)
    }

    #[inline]
    fn interrupt_mode(&self) -> InterruptMode {
        <S::Impler as Impler<Self>>::interrupt_mode(self)
    }

    #[inline]
    fn set_interrupt_mode(&mut self, x: InterruptMode) {
        <S::Impler as Impler<Self>>::set_interrupt_mode(self, x)
    }
}
