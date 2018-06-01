use impler::{Cref, Impl, Mref, Ref};
use utilities;

use hardware::memory16::Memory16;

use super::*;

use self::Reg16::*;
use self::Reg8::*;

/// An aspect of the Z80 that we can view, like a register or a memory address.
///
/// This trait (and `Changeable`) exists so that we may implement an instruction
/// like `ld x, y` with a single generic function, although `x` and `y` may be
/// memory addresses or registers.
pub trait Viewable<Output>: Copy {
    fn view<Z>(self, z: &mut Z) -> Output
    where
        Z: ?Sized + Z80Internal + Memory16;
}

/// An aspect of the Z80 that we can change, like a register or a memory address.
///
/// This trait (and `Viewable`) exists so that we may implement an instruction
/// like `ld x, y` with a single generic function, although `x` and `y` may be
/// memory addresses or registers.
pub trait Changeable<Output>: Viewable<Output> {
    fn change<Z>(self, z: &mut Z, x: Output)
    where
        Z: ?Sized + Z80Internal + Memory16;
}

impl Viewable<u8> for u8 {
    #[inline]
    fn view<Z>(self, _z: &mut Z) -> u8
    where
        Z: ?Sized + Z80Internal + Memory16,
    {
        self
    }
}

impl Viewable<u16> for u16 {
    #[inline]
    fn view<Z>(self, _z: &mut Z) -> u16
    where
        Z: ?Sized + Z80Internal + Memory16,
    {
        self
    }
}

impl Viewable<u8> for Reg8 {
    #[inline]
    fn view<Z>(self, z: &mut Z) -> u8
    where
        Z: ?Sized + Z80Internal + Memory16,
    {
        z.reg8(self)
    }
}

impl Changeable<u8> for Reg8 {
    #[inline]
    fn change<Z>(self, z: &mut Z, x: u8)
    where
        Z: ?Sized + Z80Internal + Memory16,
    {
        z.set_reg8(self, x);
    }
}

impl Viewable<u16> for Reg16 {
    #[inline]
    fn view<Z>(self, z: &mut Z) -> u16
    where
        Z: ?Sized + Z80Internal + Memory16,
    {
        z.reg16(self)
    }
}

impl Changeable<u16> for Reg16 {
    #[inline]
    fn change<Z>(self, z: &mut Z, x: u16)
    where
        Z: ?Sized + Z80Internal + Memory16,
    {
        z.set_reg16(self, x);
    }
}

impl Viewable<u16> for Address<Reg16> {
    #[inline]
    fn view<Z>(self, z: &mut Z) -> u16
    where
        Z: ?Sized + Z80Internal + Memory16,
    {
        let addr = self.0.view(z);
        let lo = z.read(addr);
        let hi = z.read(addr.wrapping_add(1));
        utilities::to16(lo, hi)
    }
}

impl Changeable<u16> for Address<Reg16> {
    #[inline]
    fn change<Z>(self, z: &mut Z, x: u16)
    where
        Z: ?Sized + Z80Internal + Memory16,
    {
        let addr = self.0.view(z);
        let (lo, hi) = utilities::to8(x);
        z.write(addr, lo);
        z.write(addr.wrapping_add(1), hi);
    }
}

impl Viewable<u8> for Address<Reg16> {
    #[inline]
    fn view<Z>(self, z: &mut Z) -> u8
    where
        Z: ?Sized + Z80Internal + Memory16,
    {
        let addr = self.0.view(z);
        z.read(addr)
    }
}

impl Changeable<u8> for Address<Reg16> {
    #[inline]
    fn change<Z>(self, z: &mut Z, x: u8)
    where
        Z: ?Sized + Z80Internal + Memory16,
    {
        let addr = self.0.view(z);
        z.write(addr, x);
    }
}

impl Viewable<u16> for Address<u16> {
    #[inline]
    fn view<Z>(self, z: &mut Z) -> u16
    where
        Z: ?Sized + Z80Internal + Memory16,
    {
        let addr = self.0;
        let lo = z.read(addr);
        let hi = z.read(addr.wrapping_add(1));
        utilities::to16(lo, hi)
    }
}

impl Changeable<u16> for Address<u16> {
    #[inline]
    fn change<Z>(self, z: &mut Z, x: u16)
    where
        Z: ?Sized + Z80Internal + Memory16,
    {
        let addr = self.0;
        let (lo, hi) = utilities::to8(x);
        z.write(addr, lo);
        z.write(addr.wrapping_add(1), hi);
    }
}

impl Viewable<u8> for Address<u16> {
    #[inline]
    fn view<Z>(self, z: &mut Z) -> u8
    where
        Z: ?Sized + Z80Internal + Memory16,
    {
        z.read(self.0)
    }
}

impl Changeable<u8> for Address<u16> {
    #[inline]
    fn change<Z>(self, z: &mut Z, x: u8)
    where
        Z: ?Sized + Z80Internal + Memory16,
    {
        z.write(self.0, x);
    }
}

impl Viewable<u8> for Shift {
    #[inline]
    fn view<Z>(self, z: &mut Z) -> u8
    where
        Z: ?Sized + Z80Internal + Memory16,
    {
        let addr = self.0.view(z).wrapping_add(self.1 as i16 as u16);
        Address(addr).view(z)
    }
}

impl Changeable<u8> for Shift {
    #[inline]
    fn change<Z>(self, z: &mut Z, x: u8)
    where
        Z: ?Sized + Z80Internal + Memory16,
    {
        let addr = self.0.view(z).wrapping_add(self.1 as i16 as u16);
        Address(addr).change(z, x);
    }
}

impl Viewable<bool> for ConditionCode {
    #[inline]
    fn view<Z>(self, z: &mut Z) -> bool
    where
        Z: ?Sized + Z80Internal + Memory16,
    {
        let f = z.reg8(Reg8::F);
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

pub struct Z80MemImpl;

impl<U> Z80Mem for U
where
    U: Impl<Z80MemImpl> + ?Sized,
    U::Impler: Z80Mem,
{
    #[inline]
    fn adc<T>(&mut self, x: Reg8, y: T)
    where
        T: Viewable<u8>,
    {
        self.make_mut().adc(x, y)
    }

    #[inline]
    fn add<T>(&mut self, x: Reg8, y: T)
    where
        T: Viewable<u8>,
    {
        self.make_mut().add(x, y)
    }

    #[inline]
    fn and<T>(&mut self, x: T)
    where
        T: Viewable<u8>,
    {
        self.make_mut().and(x)
    }

    #[inline]
    fn bit<T>(&mut self, x: u8, y: T)
    where
        T: Viewable<u8>,
    {
        self.make_mut().bit(x, y)
    }

    #[inline]
    fn call(&mut self, x: u16) {
        self.make_mut().call(x)
    }

    #[inline]
    fn callcc(&mut self, x: ConditionCode, y: u16) {
        self.make_mut().callcc(x, y)
    }

    #[inline]
    fn cp<T>(&mut self, x: T)
    where
        T: Viewable<u8>,
    {
        self.make_mut().cp(x)
    }

    #[inline]
    fn cpd(&mut self) {
        self.make_mut().cpd()
    }

    #[inline]
    fn cpdr(&mut self) {
        self.make_mut().cpdr()
    }

    #[inline]
    fn cpi(&mut self) {
        self.make_mut().cpi()
    }

    #[inline]
    fn cpir(&mut self) {
        self.make_mut().cpir()
    }

    #[inline]
    fn dec<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        self.make_mut().dec(x)
    }

    #[inline]
    fn ex<T>(&mut self, x: T, y: Reg16)
    where
        T: Changeable<u16>,
    {
        self.make_mut().ex(x, y)
    }

    #[inline]
    fn inc<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        self.make_mut().inc(x)
    }

    #[inline]
    fn jp<T>(&mut self, x: T)
    where
        T: Viewable<u16>,
    {
        self.make_mut().jp(x)
    }

    #[inline]
    fn ld<T1, T2>(&mut self, x: T1, y: T2)
    where
        T1: Changeable<u8>,
        T2: Viewable<u8>,
    {
        self.make_mut().ld(x, y)
    }

    #[inline]
    fn ld16<T1, T2>(&mut self, x: T1, y: T2)
    where
        T1: Changeable<u16>,
        T2: Viewable<u16>,
    {
        self.make_mut().ld16(x, y)
    }

    #[inline]
    fn ldd(&mut self) {
        self.make_mut().ldd()
    }

    #[inline]
    fn lddr(&mut self) {
        self.make_mut().lddr()
    }

    #[inline]
    fn ldi(&mut self) {
        self.make_mut().ldi()
    }

    #[inline]
    fn ldir(&mut self) {
        self.make_mut().ldir()
    }

    #[inline]
    fn or<T>(&mut self, x: T)
    where
        T: Viewable<u8>,
    {
        self.make_mut().or(x)
    }

    #[inline]
    fn pop(&mut self, x: Reg16) {
        self.make_mut().pop(x)
    }

    #[inline]
    fn push(&mut self, x: Reg16) {
        self.make_mut().push(x)
    }

    #[inline]
    fn res<T>(&mut self, x: u8, y: T)
    where
        T: Changeable<u8>,
    {
        self.make_mut().res(x, y)
    }

    #[inline]
    fn res_store<T>(&mut self, x: u8, y: T, w: Reg8)
    where
        T: Changeable<u8>,
    {
        self.make_mut().res_store(x, y, w)
    }

    #[inline]
    fn ret(&mut self) {
        self.make_mut().ret()
    }

    #[inline]
    fn retcc(&mut self, x: ConditionCode) {
        self.make_mut().retcc(x)
    }

    #[inline]
    fn reti(&mut self) {
        self.make_mut().reti()
    }

    #[inline]
    fn retn(&mut self) {
        self.make_mut().retn()
    }

    #[inline]
    fn rl<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        self.make_mut().rl(x)
    }

    #[inline]
    fn rl_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>,
    {
        self.make_mut().rl_store(x, y)
    }

    #[inline]
    fn rla(&mut self) {
        self.make_mut().rla()
    }

    #[inline]
    fn rlc<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        self.make_mut().rlc(x)
    }

    #[inline]
    fn rlc_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>,
    {
        self.make_mut().rlc_store(x, y)
    }

    #[inline]
    fn rlca(&mut self) {
        self.make_mut().rlca()
    }

    #[inline]
    fn rld(&mut self) {
        self.make_mut().rld()
    }

    #[inline]
    fn rr<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        self.make_mut().rr(x)
    }

    #[inline]
    fn rr_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>,
    {
        self.make_mut().rr_store(x, y)
    }

    #[inline]
    fn rra(&mut self) {
        self.make_mut().rra()
    }

    #[inline]
    fn rrc<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        self.make_mut().rrc(x)
    }

    #[inline]
    fn rrc_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>,
    {
        self.make_mut().rrc_store(x, y)
    }

    #[inline]
    fn rrca(&mut self) {
        self.make_mut().rrca()
    }

    #[inline]
    fn rrd(&mut self) {
        self.make_mut().rrd()
    }

    #[inline]
    fn rst(&mut self, x: u16) {
        self.make_mut().rst(x)
    }

    #[inline]
    fn sbc<T>(&mut self, x: Reg8, y: T)
    where
        T: Viewable<u8>,
    {
        self.make_mut().sbc(x, y)
    }

    #[inline]
    fn set<T>(&mut self, x: u8, y: T)
    where
        T: Changeable<u8>,
    {
        self.make_mut().set(x, y)
    }

    #[inline]
    fn set_store<T>(&mut self, x: u8, y: T, w: Reg8)
    where
        T: Changeable<u8>,
    {
        self.make_mut().set_store(x, y, w)
    }

    #[inline]
    fn sla<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        self.make_mut().sla(x)
    }

    #[inline]
    fn sla_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>,
    {
        self.make_mut().sla_store(x, y)
    }

    #[inline]
    fn sll<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        self.make_mut().sll(x)
    }

    #[inline]
    fn sll_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>,
    {
        self.make_mut().sll_store(x, y)
    }

    #[inline]
    fn sra<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        self.make_mut().sra(x)
    }

    #[inline]
    fn sra_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>,
    {
        self.make_mut().sra_store(x, y)
    }

    #[inline]
    fn srl<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        self.make_mut().srl(x)
    }

    #[inline]
    fn srl_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>,
    {
        self.make_mut().srl_store(x, y)
    }

    #[inline]
    fn sub<T>(&mut self, x: Reg8, y: T)
    where
        T: Viewable<u8>,
    {
        self.make_mut().sub(x, y)
    }

    #[inline]
    fn xor<T>(&mut self, x: T)
    where
        T: Viewable<u8>,
    {
        self.make_mut().xor(x)
    }
}

pub struct Z80MemImpler<T: ?Sized>(Ref<T>);

impl<T: ?Sized> Z80MemImpler<T> {
    #[inline(always)]
    pub fn new<'a>(t: &'a T) -> Cref<'a, Self> {
        Cref::Own(Z80MemImpler(unsafe { Ref::new(t) }))
    }

    #[inline(always)]
    pub fn new_mut<'a>(t: &'a mut T) -> Mref<'a, Self> {
        Mref::Own(Z80MemImpler(unsafe { Ref::new_mut(t) }))
    }
}

impl<U> Z80Mem for Z80MemImpler<U>
where
    U: Z80Internal + Memory16 + ?Sized,
{
    fn adc<T>(&mut self, x: Reg8, y: T)
    where
        T: Viewable<u8>,
    {
        let z = self.0.mut_0();
        let cf = if z.is_set_flag(CF) { 1u8 } else { 0u8 };
        let a = x.view(z);
        let y0 = y.view(z);
        let result = add_help(z, a, y0, cf);
        x.change(z, result);
    }

    fn add<T>(&mut self, x: Reg8, y: T)
    where
        T: Viewable<u8>,
    {
        let z = self.0.mut_0();
        let x0 = x.view(z);
        let y0 = y.view(z);
        let result = add_help(z, x0, y0, 0);
        x.change(z, result);
    }

    fn and<T>(&mut self, x: T)
    where
        T: Viewable<u8>,
    {
        let z = self.0.mut_0();
        let result = x.view(z) & A.view(z);
        andor_help(z, result);
        z.set_flag(HF);
    }

    fn bit<T>(&mut self, x: u8, y: T)
    where
        T: Viewable<u8>,
    {
        let z = self.0.mut_0();
        let y0 = y.view(z);
        let bitflag = 1 << x;
        let y_contains = y0 & bitflag != 0;

        z.set_flag_by(ZF | PF, !y_contains);
        z.set_flag(HF);
        z.clear_flag(NF);
        z.set_flag_by(SF, x == 7 && y_contains);
    }

    fn call(&mut self, x: u16) {
        let z = self.0.mut_0();
        let pch = PCH.view(z);
        let pcl = PCL.view(z);
        let sp = SP.view(z);
        Address(sp.wrapping_sub(1)).change(z, pch);
        Address(sp.wrapping_sub(2)).change(z, pcl);
        SP.change(z, sp.wrapping_sub(2));
        PC.change(z, x);
    }

    fn callcc(&mut self, x: ConditionCode, y: u16) {
        if x.view(self.0.mut_0()) {
            self.call(y);
            self.0.mut_0().inc_cycles(17);
        } else {
            self.0.mut_0().inc_cycles(10);
        }
    }

    fn cp<T>(&mut self, x: T)
    where
        T: Viewable<u8>,
    {
        let z = self.0.mut_0();
        let x0 = x.view(z);
        let a = A.view(z);
        // cp is like a subtraction whose result we ignore
        sub_help(z, a, x0, 0);
    }

    fn cpd(&mut self) {
        cpid(self.0.mut_0(), 0xFFFF);
    }

    fn cpdr(&mut self) {
        self.cpd();
        let z = self.0.mut_0();
        if z.reg16(BC) != 0 && !z.is_set_flag(ZF) {
            let pc = z.reg16(PC);
            z.set_reg16(PC, pc.wrapping_sub(2));
        }
    }

    fn cpi(&mut self) {
        cpid(self.0.mut_0(), 1);
    }

    fn cpir(&mut self) {
        self.cpi();
        let z = self.0.mut_0();
        if z.reg16(BC) != 0 && !z.is_set_flag(ZF) {
            let pc = z.reg16(PC);
            z.set_reg16(PC, pc.wrapping_sub(2));
        }
    }

    fn dec<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        let z = self.0.mut_0();
        let x0 = x.view(z);
        let result = x0.wrapping_sub(1);
        x.change(z, result);
        z.set_zero(result);
        z.set_sign(result);
        z.set_flag_by(HF, x0 & 0xF == 0);
        z.set_flag_by(PF, x0 == 0x80);
        z.set_flag(NF);
    }

    fn ex<T>(&mut self, x: T, y: Reg16)
    where
        T: Changeable<u16>,
    {
        let z = self.0.mut_0();
        let val1 = x.view(z);
        let val2 = y.view(z);
        x.change(z, val2);
        y.change(z, val1);
    }

    fn inc<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        let z = self.0.mut_0();
        let x0 = x.view(z);
        let result = x0.wrapping_add(1);
        x.change(z, result);
        z.set_zero(result);
        z.set_sign(result);
        z.set_flag_by(HF, x0 & 0xF == 0xF);
        z.set_flag_by(PF, x0 == 0x7F);
        z.clear_flag(NF);
    }

    fn jp<T>(&mut self, x: T)
    where
        T: Viewable<u16>,
    {
        let z = self.0.mut_0();
        let addr = x.view(z);
        z.set_reg16(PC, addr);
    }

    fn ld<T1, T2>(&mut self, x: T1, y: T2)
    where
        T1: Changeable<u8>,
        T2: Viewable<u8>,
    {
        let z = self.0.mut_0();
        let val = y.view(z);
        x.change(z, val);
    }

    fn ld16<T1, T2>(&mut self, x: T1, y: T2)
    where
        T1: Changeable<u16>,
        T2: Viewable<u16>,
    {
        let z = self.0.mut_0();
        let val = y.view(z);
        x.change(z, val);
    }

    fn ldd(&mut self) {
        ldid(self.0.mut_0(), 0xFFFF);
    }

    fn lddr(&mut self) {
        self.ldd();
        let z = self.0.mut_0();
        if z.reg16(BC) != 0 {
            let pc = z.reg16(PC);
            z.set_reg16(PC, pc.wrapping_sub(2));
        }
    }

    fn ldi(&mut self) {
        ldid(self.0.mut_0(), 1);
    }

    fn ldir(&mut self) {
        self.ldi();
        let z = self.0.mut_0();
        if z.reg16(BC) != 0 {
            let pc = z.reg16(PC);
            z.set_reg16(PC, pc.wrapping_sub(2));
        }
    }

    fn or<T>(&mut self, x: T)
    where
        T: Viewable<u8>,
    {
        let z = self.0.mut_0();
        let result = x.view(z) | A.view(z);
        andor_help(z, result);
    }

    fn pop(&mut self, x: Reg16) {
        let z = self.0.mut_0();
        let sp = SP.view(z);
        let lo = Address(sp).view(z);
        let hi = Address(sp.wrapping_add(1)).view(z);
        x.change(z, utilities::to16(lo, hi));
        SP.change(z, sp.wrapping_add(2));
    }

    fn push(&mut self, x: Reg16) {
        let z = self.0.mut_0();
        let (lo, hi) = utilities::to8(x.view(z));
        let sp = SP.view(z);
        Address(sp.wrapping_sub(1)).change(z, hi);
        Address(sp.wrapping_sub(2)).change(z, lo);
        SP.change(z, sp.wrapping_sub(2));
    }

    fn res<T>(&mut self, x: u8, y: T)
    where
        T: Changeable<u8>,
    {
        let z = self.0.mut_0();
        let mut y0 = y.view(z);
        utilities::clear_bit(&mut y0, x);
        y.change(z, y0);
    }

    fn res_store<T>(&mut self, x: u8, y: T, w: Reg8)
    where
        T: Changeable<u8>,
    {
        self.res(x, y);
        let z = self.0.mut_0();
        let y0 = y.view(z);
        w.change(z, y0);
    }

    fn ret(&mut self) {
        let z = self.0.mut_0();
        let sp = SP.view(z);
        let n1 = Address(sp).view(z);
        PCL.change(z, n1);
        let n2 = Address(sp.wrapping_add(1)).view(z);
        PCH.change(z, n2);
        SP.change(z, sp.wrapping_add(2));
    }

    fn retcc(&mut self, x: ConditionCode) {
        if x.view(self.0.mut_0()) {
            self.ret();
            self.0.mut_0().inc_cycles(11);
        } else {
            self.0.mut_0().inc_cycles(5);
        }
    }

    fn reti(&mut self) {
        self.retn()
    }

    fn retn(&mut self) {
        let z = self.0.mut_0();
        let iff2 = z.iff2();
        z.set_iff1(iff2);
        if iff2 {
            z.set_interrupt_status(InterruptStatus::Check);
        }

        let sp = SP.view(z);
        let pcl = Address(sp).view(z);
        let pch = Address(sp.wrapping_add(1)).view(z);
        PCL.change(z, pcl);
        PCH.change(z, pch);
        SP.change(z, sp.wrapping_add(2));
    }

    fn rl<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        rl(self.0.mut_0(), x)
    }

    fn rl_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>,
    {
        rl_store(self.0.mut_0(), x, y)
    }

    fn rla(&mut self) {
        rla(self.0.mut_0())
    }

    fn rlc<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        rlc(self.0.mut_0(), x)
    }

    fn rlc_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>,
    {
        rlc_store(self.0.mut_0(), x, y)
    }

    fn rlca(&mut self) {
        rlca(self.0.mut_0())
    }

    fn rld(&mut self) {
        let z = self.0.mut_0();
        let hl: u8 = Address(HL).view(z);
        let hl_lo: u8 = 0xF & hl;
        let hl_hi: u8 = 0xF0 & hl;
        let a_lo = 0xF & A.view(z);
        let a_hi = 0xF0 & A.view(z);
        Address(HL).change(z, hl_lo << 4 | a_lo);
        A.change(z, hl_hi >> 4 | a_hi);
        let a = A.view(z);

        z.set_parity(a);
        z.set_sign(a);
        z.set_zero(a);
        z.clear_flag(HF | NF);
    }

    fn rr<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        rr(self.0.mut_0(), x)
    }

    fn rr_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>,
    {
        rr_store(self.0.mut_0(), x, y)
    }

    fn rra(&mut self) {
        rra(self.0.mut_0())
    }

    fn rrc<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        rrc(self.0.mut_0(), x)
    }

    fn rrc_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>,
    {
        rrc_store(self.0.mut_0(), x, y)
    }

    fn rrca(&mut self) {
        rrca(self.0.mut_0())
    }

    fn rrd(&mut self) {
        let z = self.0.mut_0();
        let hl: u8 = Address(HL).view(z);
        let hl_lo: u8 = 0xF & hl;
        let hl_hi: u8 = 0xF0 & hl;
        let a_lo = 0xF & A.view(z);
        let a_hi = 0xF0 & A.view(z);
        Address(HL).change(z, a_lo << 4 | hl_hi >> 4);
        A.change(z, hl_lo | a_hi);
        let a = A.view(z);

        z.set_parity(a);
        z.set_sign(a);
        z.set_zero(a);
        z.clear_flag(HF | NF);
    }

    fn rst(&mut self, x: u16) {
        let z = self.0.mut_0();
        let sp = SP.view(z);
        let pch = PCH.view(z);
        let pcl = PCL.view(z);
        Address(sp.wrapping_sub(1)).change(z, pch);
        Address(sp.wrapping_sub(2)).change(z, pcl);
        SP.change(z, sp.wrapping_sub(2));
        PC.change(z, x);
    }

    fn sbc<T>(&mut self, x: Reg8, y: T)
    where
        T: Viewable<u8>,
    {
        let z = self.0.mut_0();
        let cf = if z.is_set_flag(CF) { 1u8 } else { 0u8 };
        let x0 = x.view(z);
        let y0 = y.view(z);
        let result = sub_help(z, x0, y0, cf);
        x.change(z, result);
    }

    fn set<T>(&mut self, x: u8, y: T)
    where
        T: Changeable<u8>,
    {
        let z = self.0.mut_0();
        let mut y0 = y.view(z);
        utilities::set_bit(&mut y0, x);
        y.change(z, y0);
    }

    fn set_store<T>(&mut self, x: u8, y: T, w: Reg8)
    where
        T: Changeable<u8>,
    {
        self.set(x, w);
        let z = self.0.mut_0();
        let y0 = y.view(z);
        w.change(z, y0);
    }

    fn sla<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        sla(self.0.mut_0(), x)
    }

    fn sla_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>,
    {
        sla_store(self.0.mut_0(), x, y)
    }

    fn sll<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        sll(self.0.mut_0(), x)
    }

    fn sll_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>,
    {
        sll_store(self.0.mut_0(), x, y)
    }

    fn sra<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        sra(self.0.mut_0(), x)
    }

    fn sra_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>,
    {
        sra_store(self.0.mut_0(), x, y)
    }

    fn srl<T>(&mut self, x: T)
    where
        T: Changeable<u8>,
    {
        srl(self.0.mut_0(), x)
    }

    fn srl_store<T>(&mut self, x: T, y: Reg8)
    where
        T: Changeable<u8>,
    {
        srl_store(self.0.mut_0(), x, y)
    }

    fn sub<T>(&mut self, x: Reg8, y: T)
    where
        T: Viewable<u8>,
    {
        let z = self.0.mut_0();
        let a = x.view(z);
        let y0 = y.view(z);
        let result = sub_help(z, a, y0, 0);
        x.change(z, result);
    }

    fn xor<T>(&mut self, x: T)
    where
        T: Viewable<u8>,
    {
        let z = self.0.mut_0();
        let result = x.view(z) ^ A.view(z);
        andor_help(z, result);
    }
}
