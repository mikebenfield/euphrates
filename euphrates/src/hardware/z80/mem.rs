use utilities;

use hardware::memory16::Memory16;

use super::instruction::instruction_traits::*;
use super::*;

use self::Reg16::*;
use self::Reg8::*;

pub struct Z80MemImpler<Z: ?Sized, M: ?Sized> {
    z80: *mut Z,
    memory: *mut M,
}

impl<Z: ?Sized, M: ?Sized> Z80MemImpler<Z, M> {
    #[inline(always)]
    pub unsafe fn new(z80: &mut Z, memory: &mut M) -> Self {
        Z80MemImpler { z80, memory }
    }
}

pub trait Z80MemT {
    type Z80: Z80Internal + ?Sized;
    type Memory: Memory16 + ?Sized;

    fn z80(&mut self) -> &mut Self::Z80;
    fn memory(&mut self) -> &mut Self::Memory;
}

impl<Z, M> Z80MemT for Z80MemImpler<Z, M>
where
    Z: Z80Internal + ?Sized,
    M: Memory16 + ?Sized,
{
    type Z80 = Z;
    type Memory = M;

    #[inline(always)]
    fn z80(&mut self) -> &mut Self::Z80 {
        unsafe { &mut *self.z80 }
    }

    #[inline(always)]
    fn memory(&mut self) -> &mut Self::Memory {
        unsafe { &mut *self.memory }
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

impl<Z, T> Adc<Reg8, T> for Z
where
    Z: Z80MemT + ?Sized,
    T: Viewable<u8>,
{
    fn adc(&mut self, x: Reg8, y: T) {
        let cf = if self.z80().is_set_flag(CF) { 1u8 } else { 0u8 };
        let a = x.view(self);
        let y0 = y.view(self);
        let result = add_help(self.z80(), a, y0, cf);
        x.change(self, result);
    }
}

impl<Z, T> Add<Reg8, T> for Z
where
    Z: Z80MemT + ?Sized,
    T: Viewable<u8>,
{
    fn add(&mut self, x: Reg8, y: T) {
        let x0 = x.view(self);
        let y0 = y.view(self);
        let result = add_help(self.z80(), x0, y0, 0);
        x.change(self, result);
    }
}

impl<Z, T> And<T> for Z
where
    Z: Z80MemT + ?Sized,
    T: Viewable<u8>,
{
    fn and(&mut self, x: T) {
        let result = x.view(self) & A.view(self);
        andor_help(self.z80(), result);
        self.z80().set_flag(HF);
    }
}

impl<Z, T> Bit<u8, T> for Z
where
    Z: Z80MemT + ?Sized,
    T: Viewable<u8>,
{
    fn bit(&mut self, x: u8, y: T) {
        let y0 = y.view(self);
        let bitflag = 1 << x;
        let y_contains = y0 & bitflag != 0;

        self.z80().set_flag_by(ZF | PF, !y_contains);
        self.z80().set_flag(HF);
        self.z80().clear_flag(NF);
        self.z80().set_flag_by(SF, x == 7 && y_contains);
    }
}

impl<Z> Call<u16> for Z
where
    Z: Z80MemT + ?Sized,
{
    fn call(&mut self, x: u16) {
        let pch = PCH.view(self);
        let pcl = PCL.view(self);
        let sp = SP.view(self);
        Address(sp.wrapping_sub(1)).change(self, pch);
        Address(sp.wrapping_sub(2)).change(self, pcl);
        SP.change(self, sp.wrapping_sub(2));
        PC.change(self, x);
    }
}

impl<Z> Callcc<ConditionCode, u16> for Z
where
    Z: Z80MemT + ?Sized,
{
    fn callcc(&mut self, x: ConditionCode, y: u16) {
        if x.view(self) {
            <Self as Call<u16>>::call(self, y);
            self.z80().inc_cycles(17);
        } else {
            self.z80().inc_cycles(10);
        }
    }
}

impl<Z, T> Cp<T> for Z
where
    Z: Z80MemT + ?Sized,
    T: Viewable<u8>,
{
    fn cp(&mut self, x: T) {
        let x0 = x.view(self);
        let a = A.view(self);
        // cp is like a subtraction whose result we ignore
        sub_help(self.z80(), a, x0, 0);
    }
}

impl<Z> Cpd for Z
where
    Z: Z80MemT + ?Sized,
{
    fn cpd(&mut self) {
        cpid(self, 0xFFFF);
    }
}

impl<Z> Cpdr for Z
where
    Z: Z80MemT + ?Sized,
{
    fn cpdr(&mut self) {
        self.cpd();

        if self.z80().reg16(BC) != 0 && !self.z80().is_set_flag(ZF) {
            let pc = self.z80().reg16(PC);
            self.z80().set_reg16(PC, pc.wrapping_sub(2));
            self.z80().inc_cycles(21);
        } else {
            self.z80().inc_cycles(16);
        }
    }
}

impl<Z> Cpi for Z
where
    Z: Z80MemT + ?Sized,
{
    fn cpi(&mut self) {
        cpid(self, 1);
    }
}

impl<Z> Cpir for Z
where
    Z: Z80MemT + ?Sized,
{
    fn cpir(&mut self) {
        self.cpi();

        if self.z80().reg16(BC) != 0 && !self.z80().is_set_flag(ZF) {
            let pc = self.z80().reg16(PC);
            self.z80().set_reg16(PC, pc.wrapping_sub(2));
            self.z80().inc_cycles(21);
        } else {
            self.z80().inc_cycles(16);
        }
    }
}

impl<Z, T> Dec<T> for Z
where
    Z: Z80MemT + ?Sized,
    T: Changeable<u8>,
{
    fn dec(&mut self, x: T) {
        let x0 = x.view(self);
        let result = x0.wrapping_sub(1);
        x.change(self, result);
        self.z80().set_zero(result);
        self.z80().set_sign(result);
        self.z80().set_flag_by(HF, x0 & 0xF == 0);
        self.z80().set_flag_by(PF, x0 == 0x80);
        self.z80().set_flag(NF);
    }
}

impl<Z, T> Ex<T, Reg16> for Z
where
    Z: Z80MemT + ?Sized,
    T: Changeable<u16>,
{
    fn ex(&mut self, x: T, y: Reg16) {
        let val1 = x.view(self);
        let val2 = y.view(self);
        x.change(self, val2);
        y.change(self, val1);
    }
}

impl<Z, T> Inc<T> for Z
where
    Z: Z80MemT + ?Sized,
    T: Changeable<u8>,
{
    fn inc(&mut self, x: T) {
        let x0 = x.view(self);
        let result = x0.wrapping_add(1);
        x.change(self, result);
        self.z80().set_zero(result);
        self.z80().set_sign(result);
        self.z80().set_flag_by(HF, x0 & 0xF == 0xF);
        self.z80().set_flag_by(PF, x0 == 0x7F);
        self.z80().clear_flag(NF);
    }
}

impl<Z, T> Jp<T> for Z
where
    Z: Z80MemT + ?Sized,
    T: Viewable<u16>,
{
    fn jp(&mut self, x: T) {
        let addr = x.view(self);
        self.z80().set_reg16(PC, addr);
    }
}

impl<Z, T1, T2> Ld<T1, T2> for Z
where
    Z: Z80MemT + ?Sized,
    T1: Changeable<u8>,
    T2: Viewable<u8>,
{
    fn ld(&mut self, x: T1, y: T2) {
        let val = y.view(self);
        x.change(self, val);
    }
}

impl<Z, T1, T2> Ld16<T1, T2> for Z
where
    Z: Z80MemT + ?Sized,
    T1: Changeable<u16>,
    T2: Viewable<u16>,
{
    fn ld16(&mut self, x: T1, y: T2) {
        let val = y.view(self);
        x.change(self, val);
    }
}

impl<Z> Ldd for Z
where
    Z: Z80MemT + ?Sized,
{
    fn ldd(&mut self) {
        ldid(self, 0xFFFF);
    }
}

impl<Z> Lddr for Z
where
    Z: Z80MemT + ?Sized,
{
    fn lddr(&mut self) {
        self.ldd();

        if self.z80().reg16(BC) != 0 {
            let pc = self.z80().reg16(PC);
            self.z80().set_reg16(PC, pc.wrapping_sub(2));
            self.z80().inc_cycles(21);
        } else {
            self.z80().inc_cycles(16);
        }
    }
}

impl<Z> Ldi for Z
where
    Z: Z80MemT + ?Sized,
{
    fn ldi(&mut self) {
        ldid(self, 1);
    }
}

impl<Z> Ldir for Z
where
    Z: Z80MemT + ?Sized,
{
    fn ldir(&mut self) {
        self.ldi();

        if self.z80().reg16(BC) != 0 {
            let pc = self.z80().reg16(PC);
            self.z80().set_reg16(PC, pc.wrapping_sub(2));
            self.z80().inc_cycles(21);
        } else {
            self.z80().inc_cycles(16);
        }
    }
}

impl<Z, T> Or<T> for Z
where
    Z: Z80MemT + ?Sized,
    T: Viewable<u8>,
{
    fn or(&mut self, x: T) {
        let result = x.view(self) | A.view(self);
        andor_help(self.z80(), result);
    }
}

impl<Z> Pop<Reg16> for Z
where
    Z: Z80MemT + ?Sized,
{
    fn pop(&mut self, x: Reg16) {
        let sp = SP.view(self);
        let lo = Address(sp).view(self);
        let hi = Address(sp.wrapping_add(1)).view(self);
        x.change(self, utilities::to16(lo, hi));
        SP.change(self, sp.wrapping_add(2));
    }
}

impl<Z> Push<Reg16> for Z
where
    Z: Z80MemT + ?Sized,
{
    fn push(&mut self, x: Reg16) {
        let (lo, hi) = utilities::to8(x.view(self));
        let sp = SP.view(self);
        Address(sp.wrapping_sub(1)).change(self, hi);
        Address(sp.wrapping_sub(2)).change(self, lo);
        SP.change(self, sp.wrapping_sub(2));
    }
}

impl<Z, T> Res<u8, T> for Z
where
    Z: Z80MemT + ?Sized,
    T: Changeable<u8>,
{
    fn res(&mut self, x: u8, y: T) {
        let mut y0 = y.view(self);
        utilities::clear_bit(&mut y0, x);
        y.change(self, y0);
    }
}

impl<Z, T> ResStore<u8, T, Reg8> for Z
where
    Z: Z80MemT + ?Sized,
    T: Changeable<u8>,
{
    fn res_store(&mut self, x: u8, y: T, w: Reg8) {
        self.res(x, y);

        let y0 = y.view(self);
        w.change(self, y0);
    }
}

impl<Z> Ret for Z
where
    Z: Z80MemT + ?Sized,
{
    fn ret(&mut self) {
        let sp = SP.view(self);
        let n1 = Address(sp).view(self);
        PCL.change(self, n1);
        let n2 = Address(sp.wrapping_add(1)).view(self);
        PCH.change(self, n2);
        SP.change(self, sp.wrapping_add(2));
    }
}

impl<Z> Retcc<ConditionCode> for Z
where
    Z: Z80MemT + ?Sized,
{
    fn retcc(&mut self, x: ConditionCode) {
        if x.view(self) {
            self.ret();
            self.z80().inc_cycles(11);
        } else {
            self.z80().inc_cycles(5);
        }
    }
}

impl<Z> Reti for Z
where
    Z: Z80MemT + ?Sized,
{
    fn reti(&mut self) {
        self.retn();
    }
}

impl<Z> Retn for Z
where
    Z: Z80MemT + ?Sized,
{
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
}

impl<Z, T> Rl<T> for Z
where
    Z: Z80MemT + ?Sized,
    T: Changeable<u8>,
{
    fn rl(&mut self, x: T) {
        rl(self, x)
    }
}

impl<Z, T> RlStore<T, Reg8> for Z
where
    Z: Z80MemT + ?Sized,
    T: Changeable<u8>,
{
    fn rl_store(&mut self, x: T, y: Reg8) {
        rl_store(self, x, y)
    }
}

impl<Z> Rla for Z
where
    Z: Z80MemT + ?Sized,
{
    fn rla(&mut self) {
        rla(self)
    }
}

impl<Z, T> Rlc<T> for Z
where
    Z: Z80MemT + ?Sized,
    T: Changeable<u8>,
{
    fn rlc(&mut self, x: T) {
        rlc(self, x)
    }
}

impl<Z, T> RlcStore<T, Reg8> for Z
where
    Z: Z80MemT + ?Sized,
    T: Changeable<u8>,
{
    fn rlc_store(&mut self, x: T, y: Reg8) {
        rlc_store(self, x, y)
    }
}

impl<Z> Rlca for Z
where
    Z: Z80MemT + ?Sized,
{
    fn rlca(&mut self) {
        rlca(self)
    }
}

impl<Z> Rld for Z
where
    Z: Z80MemT + ?Sized,
{
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
}

impl<Z, T> Rr<T> for Z
where
    Z: Z80MemT + ?Sized,
    T: Changeable<u8>,
{
    fn rr(&mut self, x: T) {
        rr(self, x)
    }
}

impl<Z, T> RrStore<T, Reg8> for Z
where
    Z: Z80MemT + ?Sized,
    T: Changeable<u8>,
{
    fn rr_store(&mut self, x: T, y: Reg8) {
        rr_store(self, x, y)
    }
}

impl<Z> Rra for Z
where
    Z: Z80MemT + ?Sized,
{
    fn rra(&mut self) {
        rra(self)
    }
}

impl<Z, T> Rrc<T> for Z
where
    Z: Z80MemT + ?Sized,
    T: Changeable<u8>,
{
    fn rrc(&mut self, x: T) {
        rrc(self, x)
    }
}

impl<Z, T> RrcStore<T, Reg8> for Z
where
    Z: Z80MemT + ?Sized,
    T: Changeable<u8>,
{
    fn rrc_store(&mut self, x: T, y: Reg8) {
        rrc_store(self, x, y)
    }
}

impl<Z> Rrca for Z
where
    Z: Z80MemT + ?Sized,
{
    fn rrca(&mut self) {
        rrca(self)
    }
}

impl<Z> Rrd for Z
where
    Z: Z80MemT + ?Sized,
{
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
}

impl<Z> Rst<u8> for Z
where
    Z: Z80MemT + ?Sized,
{
    fn rst(&mut self, x: u8) {
        self.rst(x as u16);
    }
}

impl<Z> Rst<u16> for Z
where
    Z: Z80MemT + ?Sized,
{
    fn rst(&mut self, x: u16) {
        let sp = SP.view(self);
        let pch = PCH.view(self);
        let pcl = PCL.view(self);
        Address(sp.wrapping_sub(1)).change(self, pch);
        Address(sp.wrapping_sub(2)).change(self, pcl);
        SP.change(self, sp.wrapping_sub(2));
        PC.change(self, x as u16);
    }
}

impl<Z, T> Sbc<Reg8, T> for Z
where
    Z: Z80MemT + ?Sized,
    T: Viewable<u8>,
{
    fn sbc(&mut self, x: Reg8, y: T) {
        let cf = if self.z80().is_set_flag(CF) { 1u8 } else { 0u8 };
        let x0 = x.view(self);
        let y0 = y.view(self);
        let result = sub_help(self.z80(), x0, y0, cf);
        x.change(self, result);
    }
}

impl<Z, T> Set<u8, T> for Z
where
    Z: Z80MemT + ?Sized,
    T: Changeable<u8>,
{
    fn set(&mut self, x: u8, y: T) {
        let mut y0 = y.view(self);
        utilities::set_bit(&mut y0, x);
        y.change(self, y0);
    }
}

impl<Z, T> SetStore<u8, T, Reg8> for Z
where
    Z: Z80MemT + ?Sized,
    T: Changeable<u8>,
{
    fn set_store(&mut self, x: u8, y: T, w: Reg8) {
        self.set(x, w);

        let y0 = y.view(self);
        w.change(self, y0);
    }
}

impl<Z, T> Sla<T> for Z
where
    Z: Z80MemT + ?Sized,
    T: Changeable<u8>,
{
    fn sla(&mut self, x: T) {
        sla(self, x)
    }
}

impl<Z, T> SlaStore<T, Reg8> for Z
where
    Z: Z80MemT + ?Sized,
    T: Changeable<u8>,
{
    fn sla_store(&mut self, x: T, y: Reg8) {
        sla_store(self, x, y)
    }
}

impl<Z, T> Sll<T> for Z
where
    Z: Z80MemT + ?Sized,
    T: Changeable<u8>,
{
    fn sll(&mut self, x: T) {
        sll(self, x)
    }
}

impl<Z, T> SllStore<T, Reg8> for Z
where
    Z: Z80MemT + ?Sized,
    T: Changeable<u8>,
{
    fn sll_store(&mut self, x: T, y: Reg8) {
        sll_store(self, x, y)
    }
}

impl<Z, T> Sra<T> for Z
where
    Z: Z80MemT + ?Sized,
    T: Changeable<u8>,
{
    fn sra(&mut self, x: T) {
        sra(self, x)
    }
}

impl<Z, T> SraStore<T, Reg8> for Z
where
    Z: Z80MemT + ?Sized,
    T: Changeable<u8>,
{
    fn sra_store(&mut self, x: T, y: Reg8) {
        sra_store(self, x, y)
    }
}

impl<Z, T> Srl<T> for Z
where
    Z: Z80MemT + ?Sized,
    T: Changeable<u8>,
{
    fn srl(&mut self, x: T) {
        srl(self, x)
    }
}

impl<Z, T> SrlStore<T, Reg8> for Z
where
    Z: Z80MemT + ?Sized,
    T: Changeable<u8>,
{
    fn srl_store(&mut self, x: T, y: Reg8) {
        srl_store(self, x, y)
    }
}

impl<Z, T> Sub<Reg8, T> for Z
where
    Z: Z80MemT + ?Sized,
    T: Viewable<u8>,
{
    fn sub(&mut self, x: Reg8, y: T) {
        let a = x.view(self);
        let y0 = y.view(self);
        let result = sub_help(self.z80(), a, y0, 0);
        x.change(self, result);
    }
}

impl<Z, T> Xor<T> for Z
where
    Z: Z80MemT + ?Sized,
    T: Viewable<u8>,
{
    fn xor(&mut self, x: T) {
        let result = x.view(self) ^ A.view(self);
        andor_help(self.z80(), result);
    }
}
