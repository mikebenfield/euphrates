use utilities;

use hardware::memory16::Memory16;

use super::*;

use self::Reg16::*;
use self::Reg8::*;

pub struct Z80MemImpler<'a, Z: 'a + ?Sized, M: 'a + ?Sized> {
    pub z80: &'a mut Z,
    pub memory: &'a mut M,
}

pub trait Z80MemT {
    type Z80: Z80Internal + ?Sized;
    type Memory: Memory16 + ?Sized;

    fn z80(&mut self) -> &mut Self::Z80;
    fn memory(&mut self) -> &mut Self::Memory;
}

impl<'a, Z: 'a, M: 'a> Z80MemT for Z80MemImpler<'a, Z, M>
where
    Z: Z80Internal + ?Sized,
    M: Memory16 + ?Sized,
{
    type Z80 = Z;
    type Memory = M;

    #[inline(always)]
    fn z80(&mut self) -> &mut Self::Z80 {
        self.z80
    }

    #[inline(always)]
    fn memory(&mut self) -> &mut Self::Memory {
        self.memory
    }
}

/// An aspect of the Z80 that we can view, like a register or a memory address.
///
/// This trait (and `Changeable`) exists so that we may implement an instruction
/// like `ld x, y` with a single generic function, although `x` and `y` may be
/// memory addresses or registers.
pub trait Viewable<Output>: Copy {
    fn view<Z>(self, z: &mut Z) -> Output
    where
        Z: Z80MemT + ?Sized;
}

/// An aspect of the Z80 that we can change, like a register or a memory address.
///
/// This trait (and `Viewable`) exists so that we may implement an instruction
/// like `ld x, y` with a single generic function, although `x` and `y` may be
/// memory addresses or registers.
pub trait Changeable<Output>: Viewable<Output> {
    fn change<Z>(self, z: &mut Z, x: Output)
    where
        Z: Z80MemT + ?Sized;
}

impl Viewable<u8> for u8 {
    #[inline]
    fn view<Z>(self, _z: &mut Z) -> u8
    where
        Z: Z80MemT + ?Sized,
    {
        self
    }
}

impl Viewable<u16> for u16 {
    #[inline]
    fn view<Z>(self, _z: &mut Z) -> u16
    where
        Z: Z80MemT + ?Sized,
    {
        self
    }
}

impl Viewable<u8> for Reg8 {
    #[inline]
    fn view<Z>(self, z: &mut Z) -> u8
    where
        Z: Z80MemT + ?Sized,
    {
        z.z80().reg8(self)
    }
}

impl Changeable<u8> for Reg8 {
    #[inline]
    fn change<Z>(self, z: &mut Z, x: u8)
    where
        Z: Z80MemT + ?Sized,
    {
        z.z80().set_reg8(self, x);
    }
}

impl Viewable<u16> for Reg16 {
    #[inline]
    fn view<Z>(self, z: &mut Z) -> u16
    where
        Z: Z80MemT + ?Sized,
    {
        z.z80().reg16(self)
    }
}

impl Changeable<u16> for Reg16 {
    #[inline]
    fn change<Z>(self, z: &mut Z, x: u16)
    where
        Z: Z80MemT + ?Sized,
    {
        z.z80().set_reg16(self, x);
    }
}

impl Viewable<u16> for Address<Reg16> {
    #[inline]
    fn view<Z>(self, z: &mut Z) -> u16
    where
        Z: Z80MemT + ?Sized,
    {
        let addr = self.0.view(z);
        let lo = z.memory().read(addr);
        let hi = z.memory().read(addr.wrapping_add(1));
        utilities::to16(lo, hi)
    }
}

impl Changeable<u16> for Address<Reg16> {
    #[inline]
    fn change<Z>(self, z: &mut Z, x: u16)
    where
        Z: Z80MemT + ?Sized,
    {
        let addr = self.0.view(z);
        let (lo, hi) = utilities::to8(x);
        z.memory().write(addr, lo);
        z.memory().write(addr.wrapping_add(1), hi);
    }
}

impl Viewable<u8> for Address<Reg16> {
    #[inline]
    fn view<Z>(self, z: &mut Z) -> u8
    where
        Z: Z80MemT + ?Sized,
    {
        let addr = self.0.view(z);
        z.memory().read(addr)
    }
}

impl Changeable<u8> for Address<Reg16> {
    #[inline]
    fn change<Z>(self, z: &mut Z, x: u8)
    where
        Z: Z80MemT + ?Sized,
    {
        let addr = self.0.view(z);
        z.memory().write(addr, x);
    }
}

impl Viewable<u16> for Address<u16> {
    #[inline]
    fn view<Z>(self, z: &mut Z) -> u16
    where
        Z: Z80MemT + ?Sized,
    {
        let addr = self.0;
        let lo = z.memory().read(addr);
        let hi = z.memory().read(addr.wrapping_add(1));
        utilities::to16(lo, hi)
    }
}

impl Changeable<u16> for Address<u16> {
    #[inline]
    fn change<Z>(self, z: &mut Z, x: u16)
    where
        Z: Z80MemT + ?Sized,
    {
        let addr = self.0;
        let (lo, hi) = utilities::to8(x);
        z.memory().write(addr, lo);
        z.memory().write(addr.wrapping_add(1), hi);
    }
}

impl Viewable<u8> for Address<u16> {
    #[inline]
    fn view<Z>(self, z: &mut Z) -> u8
    where
        Z: Z80MemT + ?Sized,
    {
        z.memory().read(self.0)
    }
}

impl Changeable<u8> for Address<u16> {
    #[inline]
    fn change<Z>(self, z: &mut Z, x: u8)
    where
        Z: Z80MemT + ?Sized,
    {
        z.memory().write(self.0, x)
    }
}

impl Viewable<u8> for Shift {
    #[inline]
    fn view<Z>(self, z: &mut Z) -> u8
    where
        Z: Z80MemT + ?Sized,
    {
        let addr = self.0.view(z).wrapping_add(self.1 as i16 as u16);
        Address(addr).view(z)
    }
}

impl Changeable<u8> for Shift {
    #[inline]
    fn change<Z>(self, z: &mut Z, x: u8)
    where
        Z: Z80MemT + ?Sized,
    {
        let addr = self.0.view(z).wrapping_add(self.1 as i16 as u16);
        Address(addr).change(z, x);
    }
}

impl Viewable<bool> for ConditionCode {
    #[inline]
    fn view<Z>(self, z: &mut Z) -> bool
    where
        Z: Z80MemT + ?Sized,
    {
        let f = z.z80().reg8(Reg8::F);
        self.check(f)
    }
}

/// Z80 instructions that require `Memory16`.
pub trait Z80Mem {
    fn adc<T>(&mut self, x: Reg8, y: T)
    where
        T: Viewable<u8>;

    fn add<T>(&mut self, x: Reg8, y: T)
    where
        T: Viewable<u8>;

    fn and<T>(&mut self, x: T)
    where
        T: Viewable<u8>;

    fn bit<T>(&mut self, x: u8, y: T)
    where
        T: Viewable<u8>;

    fn call(&mut self, x: u16);

    fn callcc(&mut self, x: ConditionCode, y: u16);

    fn cp<T>(&mut self, x: T)
    where
        T: Viewable<u8>;

    fn cpd(&mut self);

    fn cpdr(&mut self);

    fn cpi(&mut self);

    fn cpir(&mut self);

    fn dec<T>(&mut self, x: T)
    where
        T: Changeable<u8>;

    fn ex<T>(&mut self, x: T, y: Reg16)
    where
        T: Changeable<u16>;

    fn inc<T>(&mut self, x: T)
    where
        T: Changeable<u8>;

    fn jp<T>(&mut self, x: T)
    where
        T: Viewable<u16>;

    fn ld<T1, T2>(&mut self, x: T1, y: T2)
    where
        T1: Changeable<u8>,
        T2: Viewable<u8>;

    fn ld16<T1, T2>(&mut self, x: T1, y: T2)
    where
        T1: Changeable<u16>,
        T2: Viewable<u16>;

    fn ldd(&mut self);

    fn lddr(&mut self);

    fn ldi(&mut self);

    fn ldir(&mut self);

    fn or<T>(&mut self, x: T)
    where
        T: Viewable<u8>;

    fn pop(&mut self, x: Reg16);

    fn push(&mut self, x: Reg16);

    fn res<T>(&mut self, x: u8, y: T)
    where
        T: Changeable<u8>;

    fn res_store<T>(&mut self, x: u8, y: T, w: Reg8)
    where
        T: Changeable<u8>;

    fn ret(&mut self);

    fn retcc(&mut self, x: ConditionCode);

    fn reti(&mut self);

    fn retn(&mut self);

    fn rl<T>(&mut self, x: T)
    where
        T: Changeable<u8>;

    fn rl_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>;

    fn rla(&mut self);

    fn rlc<T>(&mut self, x: T)
    where
        T: Changeable<u8>;

    fn rlc_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>;

    fn rlca(&mut self);

    fn rld(&mut self);

    fn rr<T>(&mut self, x: T)
    where
        T: Changeable<u8>;

    fn rr_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>;

    fn rra(&mut self);

    fn rrc<T>(&mut self, x: T)
    where
        T: Changeable<u8>;

    fn rrc_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>;

    fn rrca(&mut self);

    fn rrd(&mut self);

    fn rst(&mut self, x: u16);

    fn sbc<T>(&mut self, x: Reg8, y: T)
    where
        T: Viewable<u8>;

    fn set<T>(&mut self, x: u8, y: T)
    where
        T: Changeable<u8>;

    fn set_store<T>(&mut self, x: u8, y: T, w: Reg8)
    where
        T: Changeable<u8>;

    fn sla<T>(&mut self, x: T)
    where
        T: Changeable<u8>;

    fn sla_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>;

    fn sll<T>(&mut self, x: T)
    where
        T: Changeable<u8>;

    fn sll_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>;

    fn sra<T>(&mut self, x: T)
    where
        T: Changeable<u8>;

    fn sra_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>;

    fn srl<T>(&mut self, x: T)
    where
        T: Changeable<u8>;

    fn srl_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>;

    fn sub<T>(&mut self, x: Reg8, y: T)
    where
        T: Viewable<u8>;

    fn xor<T>(&mut self, x: T)
    where
        T: Viewable<u8>;
}

impl<U> Z80Mem for U
where
    U: Z80MemT + ?Sized,
{
    fn adc<T>(&mut self, x: Reg8, y: T)
    where
        T: Viewable<u8>,
    {
        let cf = if self.z80().is_set_flag(CF) { 1u8 } else { 0u8 };
        let a = x.view(self);
        let y0 = y.view(self);
        let result = add_help(self.z80(), a, y0, cf);
        x.change(self, result);
    }

    fn add<T>(&mut self, x: Reg8, y: T)
    where
        T: Viewable<u8>,
    {
        let x0 = x.view(self);
        let y0 = y.view(self);
        let result = add_help(self.z80(), x0, y0, 0);
        x.change(self, result);
    }

    fn and<T>(&mut self, x: T)
    where
        T: Viewable<u8>,
    {
        let result = x.view(self) & A.view(self);
        andor_help(self.z80(), result);
        self.z80().set_flag(HF);
    }

    fn bit<T>(&mut self, x: u8, y: T)
    where
        T: Viewable<u8>,
    {
        let y0 = y.view(self);
        let bitflag = 1 << x;
        let y_contains = y0 & bitflag != 0;

        self.z80().set_flag_by(ZF | PF, !y_contains);
        self.z80().set_flag(HF);
        self.z80().clear_flag(NF);
        self.z80().set_flag_by(SF, x == 7 && y_contains);
    }

    fn call(&mut self, x: u16) {
        let pch = PCH.view(self);
        let pcl = PCL.view(self);
        let sp = SP.view(self);
        Address(sp.wrapping_sub(1)).change(self, pch);
        Address(sp.wrapping_sub(2)).change(self, pcl);
        SP.change(self, sp.wrapping_sub(2));
        PC.change(self, x);
    }

    fn callcc(&mut self, x: ConditionCode, y: u16) {
        if x.view(self) {
            self.call(y);
            self.z80().inc_cycles(17);
        } else {
            self.z80().inc_cycles(10);
        }
    }

    fn cp<T>(&mut self, x: T)
    where
        T: Viewable<u8>,
    {
        let x0 = x.view(self);
        let a = A.view(self);
        // cp is like a subtraction whose result we ignore
        sub_help(self.z80(), a, x0, 0);
    }

    fn cpd(&mut self) {
        cpid(self, 0xFFFF);
    }

    fn cpdr(&mut self) {
        self.cpd();

        if self.z80().reg16(BC) != 0 && !self.z80().is_set_flag(ZF) {
            let pc = self.z80().reg16(PC);
            self.z80().set_reg16(PC, pc.wrapping_sub(2));
        }
    }

    fn cpi(&mut self) {
        cpid(self, 1);
    }

    fn cpir(&mut self) {
        self.cpi();

        if self.z80().reg16(BC) != 0 && !self.z80().is_set_flag(ZF) {
            let pc = self.z80().reg16(PC);
            self.z80().set_reg16(PC, pc.wrapping_sub(2));
        }
    }

    fn dec<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        let x0 = x.view(self);
        let result = x0.wrapping_sub(1);
        x.change(self, result);
        self.z80().set_zero(result);
        self.z80().set_sign(result);
        self.z80().set_flag_by(HF, x0 & 0xF == 0);
        self.z80().set_flag_by(PF, x0 == 0x80);
        self.z80().set_flag(NF);
    }

    fn ex<T>(&mut self, x: T, y: Reg16)
    where
        T: Changeable<u16>,
    {
        let val1 = x.view(self);
        let val2 = y.view(self);
        x.change(self, val2);
        y.change(self, val1);
    }

    fn inc<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        let x0 = x.view(self);
        let result = x0.wrapping_add(1);
        x.change(self, result);
        self.z80().set_zero(result);
        self.z80().set_sign(result);
        self.z80().set_flag_by(HF, x0 & 0xF == 0xF);
        self.z80().set_flag_by(PF, x0 == 0x7F);
        self.z80().clear_flag(NF);
    }

    fn jp<T>(&mut self, x: T)
    where
        T: Viewable<u16>,
    {
        let addr = x.view(self);
        self.z80().set_reg16(PC, addr);
    }

    fn ld<T1, T2>(&mut self, x: T1, y: T2)
    where
        T1: Changeable<u8>,
        T2: Viewable<u8>,
    {
        let val = y.view(self);
        x.change(self, val);
    }

    fn ld16<T1, T2>(&mut self, x: T1, y: T2)
    where
        T1: Changeable<u16>,
        T2: Viewable<u16>,
    {
        let val = y.view(self);
        x.change(self, val);
    }

    fn ldd(&mut self) {
        ldid(self, 0xFFFF);
    }

    fn lddr(&mut self) {
        self.ldd();

        if self.z80().reg16(BC) != 0 {
            let pc = self.z80().reg16(PC);
            self.z80().set_reg16(PC, pc.wrapping_sub(2));
        }
    }

    fn ldi(&mut self) {
        ldid(self, 1);
    }

    fn ldir(&mut self) {
        self.ldi();

        if self.z80().reg16(BC) != 0 {
            let pc = self.z80().reg16(PC);
            self.z80().set_reg16(PC, pc.wrapping_sub(2));
        }
    }

    fn or<T>(&mut self, x: T)
    where
        T: Viewable<u8>,
    {
        let result = x.view(self) | A.view(self);
        andor_help(self.z80(), result);
    }

    fn pop(&mut self, x: Reg16) {
        let sp = SP.view(self);
        let lo = Address(sp).view(self);
        let hi = Address(sp.wrapping_add(1)).view(self);
        x.change(self, utilities::to16(lo, hi));
        SP.change(self, sp.wrapping_add(2));
    }

    fn push(&mut self, x: Reg16) {
        let (lo, hi) = utilities::to8(x.view(self));
        let sp = SP.view(self);
        Address(sp.wrapping_sub(1)).change(self, hi);
        Address(sp.wrapping_sub(2)).change(self, lo);
        SP.change(self, sp.wrapping_sub(2));
    }

    fn res<T>(&mut self, x: u8, y: T)
    where
        T: Changeable<u8>,
    {
        let mut y0 = y.view(self);
        utilities::clear_bit(&mut y0, x);
        y.change(self, y0);
    }

    fn res_store<T>(&mut self, x: u8, y: T, w: Reg8)
    where
        T: Changeable<u8>,
    {
        self.res(x, y);

        let y0 = y.view(self);
        w.change(self, y0);
    }

    fn ret(&mut self) {
        let sp = SP.view(self);
        let n1 = Address(sp).view(self);
        PCL.change(self, n1);
        let n2 = Address(sp.wrapping_add(1)).view(self);
        PCH.change(self, n2);
        SP.change(self, sp.wrapping_add(2));
    }

    fn retcc(&mut self, x: ConditionCode) {
        if x.view(self) {
            self.ret();
            self.z80().inc_cycles(11);
        } else {
            self.z80().inc_cycles(5);
        }
    }

    fn reti(&mut self) {
        self.retn()
    }

    fn retn(&mut self) {
        let iff2 = self.z80().iff2();
        self.z80().set_iff1(iff2);
        if iff2 {
            self.z80().set_interrupt_status(InterruptStatus::Check);
        }

        let sp = SP.view(self);
        let pcl = Address(sp).view(self);
        let pch = Address(sp.wrapping_add(1)).view(self);
        PCL.change(self, pcl);
        PCH.change(self, pch);
        SP.change(self, sp.wrapping_add(2));
    }

    fn rl<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        rl(self, x)
    }

    fn rl_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>,
    {
        rl_store(self, x, y)
    }

    fn rla(&mut self) {
        rla(self)
    }

    fn rlc<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        rlc(self, x)
    }

    fn rlc_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>,
    {
        rlc_store(self, x, y)
    }

    fn rlca(&mut self) {
        rlca(self)
    }

    fn rld(&mut self) {
        let hl: u8 = Address(HL).view(self);
        let hl_lo: u8 = 0xF & hl;
        let hl_hi: u8 = 0xF0 & hl;
        let a_lo = 0xF & A.view(self);
        let a_hi = 0xF0 & A.view(self);
        Address(HL).change(self, hl_lo << 4 | a_lo);
        A.change(self, hl_hi >> 4 | a_hi);
        let a = A.view(self);

        self.z80().set_parity(a);
        self.z80().set_sign(a);
        self.z80().set_zero(a);
        self.z80().clear_flag(HF | NF);
    }

    fn rr<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        rr(self, x)
    }

    fn rr_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>,
    {
        rr_store(self, x, y)
    }

    fn rra(&mut self) {
        rra(self)
    }

    fn rrc<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        rrc(self, x)
    }

    fn rrc_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>,
    {
        rrc_store(self, x, y)
    }

    fn rrca(&mut self) {
        rrca(self)
    }

    fn rrd(&mut self) {
        let hl: u8 = Address(HL).view(self);
        let hl_lo: u8 = 0xF & hl;
        let hl_hi: u8 = 0xF0 & hl;
        let a_lo = 0xF & A.view(self);
        let a_hi = 0xF0 & A.view(self);
        Address(HL).change(self, a_lo << 4 | hl_hi >> 4);
        A.change(self, hl_lo | a_hi);
        let a = A.view(self);

        self.z80().set_parity(a);
        self.z80().set_sign(a);
        self.z80().set_zero(a);
        self.z80().clear_flag(HF | NF);
    }

    fn rst(&mut self, x: u16) {
        let sp = SP.view(self);
        let pch = PCH.view(self);
        let pcl = PCL.view(self);
        Address(sp.wrapping_sub(1)).change(self, pch);
        Address(sp.wrapping_sub(2)).change(self, pcl);
        SP.change(self, sp.wrapping_sub(2));
        PC.change(self, x);
    }

    fn sbc<T>(&mut self, x: Reg8, y: T)
    where
        T: Viewable<u8>,
    {
        let cf = if self.z80().is_set_flag(CF) { 1u8 } else { 0u8 };
        let x0 = x.view(self);
        let y0 = y.view(self);
        let result = sub_help(self.z80(), x0, y0, cf);
        x.change(self, result);
    }

    fn set<T>(&mut self, x: u8, y: T)
    where
        T: Changeable<u8>,
    {
        let mut y0 = y.view(self);
        utilities::set_bit(&mut y0, x);
        y.change(self, y0);
    }

    fn set_store<T>(&mut self, x: u8, y: T, w: Reg8)
    where
        T: Changeable<u8>,
    {
        self.set(x, w);

        let y0 = y.view(self);
        w.change(self, y0);
    }

    fn sla<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        sla(self, x)
    }

    fn sla_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>,
    {
        sla_store(self, x, y)
    }

    fn sll<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        sll(self, x)
    }

    fn sll_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>,
    {
        sll_store(self, x, y)
    }

    fn sra<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        sra(self, x)
    }

    fn sra_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>,
    {
        sra_store(self, x, y)
    }

    fn srl<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        srl(self, x)
    }

    fn srl_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>,
    {
        srl_store(self, x, y)
    }

    fn sub<T>(&mut self, x: Reg8, y: T)
    where
        T: Viewable<u8>,
    {
        let a = x.view(self);
        let y0 = y.view(self);
        let result = sub_help(self.z80(), a, y0, 0);
        x.change(self, result);
    }

    fn xor<T>(&mut self, x: T)
    where
        T: Viewable<u8>,
    {
        let result = x.view(self) ^ A.view(self);
        andor_help(self.z80(), result);
    }
}
