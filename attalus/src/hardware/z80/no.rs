//! Z80 instructions that don't depend on memory or IO.

use impler::{Cref, Impl, Mref, Ref};

use super::*;

use self::Reg16::*;
use self::Reg8::*;

pub trait Z80No {
    fn adc16(&mut self, x: Reg16, y: Reg16);
    fn add16(&mut self, x: Reg16, y: Reg16);
    fn ccf(&mut self);
    fn cpl(&mut self);
    fn daa(&mut self);
    fn dec16(&mut self, x: Reg16);
    fn di(&mut self);
    fn djnz(&mut self, e: i8);

    /// Note that just calling `ei` is not sufficient to emulate the `ei`
    /// instruction.
    ///
    /// This method will set the `iff1` and `iff2` flags, as it's supposed to -
    /// but the Z80 actually does not accept interrupts until after the
    /// *following* instruction, so the emulator must do some work to make that
    /// happen.
    fn ei(&mut self);
    fn exx(&mut self);
    fn halt(&mut self);
    fn im(&mut self, x: u8);
    fn inc16(&mut self, x: Reg16);
    fn jpcc(&mut self, cc: ConditionCode, nn: u16);
    fn jr(&mut self, e: i8);
    fn jrcc(&mut self, cc: ConditionCode, e: i8);
    fn ld_ir(&mut self, x: Reg8, y: Reg8);
    fn neg(&mut self);
    fn nop(&mut self);
    fn sbc16(&mut self, x: Reg16, y: Reg16);
    fn scf(&mut self);

    /// pseudo instruction
    fn dd(&mut self);

    /// pseudo instruction
    fn fd(&mut self);

    /// pseudo instruction
    fn cb(&mut self);

    /// pseudo instruction
    fn ed(&mut self);

    /// pseudo instruction
    fn ddcb(&mut self);

    /// pseudo instruction
    fn fdcb(&mut self);
}

pub struct Z80NoImpl;

impl<T> Z80No for T
where
    T: Impl<Z80NoImpl> + ?Sized,
    T::Impler: Z80No,
{
    #[inline(always)]
    fn adc16(&mut self, x: Reg16, y: Reg16) {
        self.make_mut().adc16(x, y)
    }

    #[inline(always)]
    fn add16(&mut self, x: Reg16, y: Reg16) {
        self.make_mut().add16(x, y)
    }

    #[inline(always)]
    fn ccf(&mut self) {
        self.make_mut().ccf()
    }

    #[inline(always)]
    fn cpl(&mut self) {
        self.make_mut().cpl()
    }

    #[inline(always)]
    fn daa(&mut self) {
        self.make_mut().daa()
    }

    #[inline(always)]
    fn dec16(&mut self, x: Reg16) {
        self.make_mut().dec16(x)
    }

    #[inline(always)]
    fn di(&mut self) {
        self.make_mut().di()
    }

    #[inline(always)]
    fn djnz(&mut self, e: i8) {
        self.make_mut().djnz(e)
    }

    #[inline(always)]
    fn ei(&mut self) {
        self.make_mut().ei()
    }

    #[inline(always)]
    fn exx(&mut self) {
        self.make_mut().exx()
    }

    #[inline(always)]
    fn halt(&mut self) {
        self.make_mut().halt()
    }

    #[inline(always)]
    fn im(&mut self, x: u8) {
        self.make_mut().im(x)
    }

    #[inline(always)]
    fn inc16(&mut self, x: Reg16) {
        self.make_mut().inc16(x)
    }

    #[inline(always)]
    fn jpcc(&mut self, cc: ConditionCode, nn: u16) {
        self.make_mut().jpcc(cc, nn)
    }

    #[inline(always)]
    fn jr(&mut self, e: i8) {
        self.make_mut().jr(e)
    }

    #[inline(always)]
    fn jrcc(&mut self, cc: ConditionCode, e: i8) {
        self.make_mut().jrcc(cc, e)
    }

    #[inline(always)]
    fn ld_ir(&mut self, x: Reg8, y: Reg8) {
        self.make_mut().ld_ir(x, y)
    }

    #[inline(always)]
    fn neg(&mut self) {
        self.make_mut().neg()
    }

    #[inline(always)]
    fn nop(&mut self) {
        self.make_mut().nop()
    }

    #[inline(always)]
    fn sbc16(&mut self, x: Reg16, y: Reg16) {
        self.make_mut().sbc16(x, y)
    }

    #[inline(always)]
    fn scf(&mut self) {
        self.make_mut().scf()
    }

    #[inline(always)]
    fn dd(&mut self) {
        self.make_mut().dd()
    }

    #[inline(always)]
    fn fd(&mut self) {
        self.make_mut().fd()
    }

    #[inline(always)]
    fn cb(&mut self) {
        self.make_mut().cb()
    }

    #[inline(always)]
    fn ed(&mut self) {
        self.make_mut().ed()
    }

    #[inline(always)]
    fn ddcb(&mut self) {
        self.make_mut().ddcb()
    }

    #[inline(always)]
    fn fdcb(&mut self) {
        self.make_mut().fdcb()
    }
}

pub struct Z80NoImpler<T: ?Sized>(Ref<T>);

impl<T: ?Sized> Z80NoImpler<T> {
    #[inline(always)]
    pub fn new<'a>(t: &'a T) -> Cref<'a, Self> {
        Cref::Own(Z80NoImpler(unsafe { Ref::new(t) }))
    }

    #[inline(always)]
    pub fn new_mut<'a>(t: &'a mut T) -> Mref<'a, Self> {
        Mref::Own(Z80NoImpler(unsafe { Ref::new_mut(t) }))
    }
}

impl<T> Z80No for Z80NoImpler<T>
where
    T: Z80Internal + ?Sized,
{
    fn adc16(&mut self, x: Reg16, y: Reg16) {
        let z = self.0.mut_0();
        let x0 = z.reg16(x);
        let y0 = z.reg16(y);
        let cf = if z.is_set_flag(CF) { 1u8 } else { 0u8 };
        let result = adc16_help(z, x0, y0, cf as u16);
        z.set_reg16(x, result);
    }

    fn add16(&mut self, x: Reg16, y: Reg16) {
        let z = self.0.mut_0();
        let x0 = z.reg16(x);
        let y0 = z.reg16(y);
        let result = add16_help(z, x0, y0, 0);
        z.set_reg16(x, result);
    }

    fn ccf(&mut self) {
        let z = self.0.mut_0();
        let cf = z.is_set_flag(CF);
        z.set_flag_by(HF, cf);
        z.set_flag_by(CF, !cf);
        z.clear_flag(NF);
    }

    fn cpl(&mut self) {
        let z = self.0.mut_0();
        let a = z.reg8(A);
        z.set_reg8(A, !a);
        z.set_flag(HF | NF);
    }

    fn daa(&mut self) {
        let z = self.0.mut_0();
        // see the table in Young
        let a = z.reg8(A);
        let cf = z.is_set_flag(CF);
        let hf = z.is_set_flag(HF);
        let nf = z.is_set_flag(NF);
        let diff = match (cf, a >> 4, hf, a & 0xF) {
            (false, 0...9, false, 0...9) => 0,
            (false, 0...9, true, 0...9) => 0x6,
            (false, 0...8, _, 0xA...0xF) => 0x6,
            (false, 0xA...0xF, false, 0...9) => 0x60,
            (true, _, false, 0...9) => 0x60,
            _ => 0x66,
        };

        let new_cf = match (cf, a >> 4, a & 0xF) {
            (false, 0...9, 0...9) => 0,
            (false, 0...8, 0xA...0xF) => 0,
            _ => 1,
        };

        let new_hf = match (nf, hf, a & 0xF) {
            (false, _, 0xA...0xF) => 1,
            (true, true, 0...5) => 1,
            _ => 0,
        };

        let new_a = if nf {
            a.wrapping_sub(diff)
        } else {
            a.wrapping_add(diff)
        };
        z.set_reg8(A, new_a);

        z.set_parity(new_a);
        z.set_zero(new_a);
        z.set_sign(new_a);
        z.set_flag_by(CF, new_cf != 0);
        z.set_flag_by(HF, new_hf != 0);
    }

    fn dec16(&mut self, x: Reg16) {
        let z = self.0.mut_0();
        let val = z.reg16(x);
        z.set_reg16(x, val.wrapping_sub(1));
    }

    fn di(&mut self) {
        let z = self.0.mut_0();
        z.set_iff1(false);
        z.set_iff2(false);
        z.set_interrupt_status(InterruptStatus::NoCheck);
    }

    fn djnz(&mut self, e: i8) {
        let b = self.0.mut_0().reg8(B);
        let new_b = b.wrapping_sub(1);
        self.0.mut_0().set_reg8(B, new_b);
        if new_b != 0 {
            self.jr(e);
        }
    }

    fn ei(&mut self) {
        let z = self.0.mut_0();
        z.set_iff1(true);
        z.set_iff2(true);
        let cycles = z.cycles().wrapping_add(4);
        z.set_interrupt_status(InterruptStatus::Ei(cycles));
    }

    fn exx(&mut self) {
        let z = self.0.mut_0();
        for &(reg1, reg2) in [(BC, BC0), (DE, DE0), (HL, HL0)].iter() {
            let val1 = z.reg16(reg1);
            let val2 = z.reg16(reg2);
            z.set_reg16(reg1, val2);
            z.set_reg16(reg2, val1);
        }
    }

    fn halt(&mut self) {
        let z = self.0.mut_0();
        z.set_prefix(Prefix::Halt);
    }

    fn im(&mut self, x: u8) {
        let z = self.0.mut_0();

        match x {
            0 => z.set_interrupt_mode(InterruptMode::Im0),
            1 => z.set_interrupt_mode(InterruptMode::Im1),
            2 => z.set_interrupt_mode(InterruptMode::Im2),
            _ => panic!("Z80: Invalid interrupt mode"),
        }
    }

    fn inc16(&mut self, x: Reg16) {
        let z = self.0.mut_0();
        let x0 = z.reg16(x);
        z.set_reg16(x, x0.wrapping_add(1));
    }

    fn jpcc(&mut self, cc: ConditionCode, nn: u16) {
        let z = self.0.mut_0();
        let flags = z.reg8(F);
        if cc.check(flags) {
            z.set_reg16(PC, nn);
        }
    }

    fn jr(&mut self, e: i8) {
        let z = self.0.mut_0();
        let pc = z.reg16(PC);
        z.set_reg16(PC, pc.wrapping_add(e as i16 as u16));
    }

    fn jrcc(&mut self, cc: ConditionCode, e: i8) {
        let flags = self.0.mut_0().reg8(F);
        if cc.check(flags) {
            self.jr(e);
        }
    }

    fn ld_ir(&mut self, x: Reg8, y: Reg8) {
        let z = self.0.mut_0();
        let y0 = z.reg8(y);
        z.set_reg8(x, y0);
        let iff2 = z.iff2();
        z.set_sign(y0);
        z.set_zero(y0);
        z.clear_flag(NF | HF);
        z.set_flag_by(PF, iff2);
    }

    fn neg(&mut self) {
        let z = self.0.mut_0();
        let a = z.reg8(A);
        let result = sub_help(z, 0, a, 0);
        z.set_reg8(A, result);
        z.set_flag_by(PF, a == 0x80);
        z.set_flag_by(CF, a != 0);
    }

    fn nop(&mut self) {}

    fn sbc16(&mut self, x: Reg16, y: Reg16) {
        let z = self.0.mut_0();
        let x0 = z.reg16(x);
        let y0 = z.reg16(y);
        let cf = if z.is_set_flag(CF) { 1u8 } else { 0u8 };
        let result = adc16_help(z, x0, !y0, (1 ^ cf) as u16);
        z.set_reg16(x, result);
        let cf = z.is_set_flag(CF);
        let hf = z.is_set_flag(HF);
        z.set_flag_by(CF, !cf);
        z.set_flag_by(HF, !hf);
        z.set_flag(NF);
    }

    fn scf(&mut self) {
        let z = self.0.mut_0();
        z.clear_flag(HF | NF);
        z.set_flag(CF);
    }

    fn dd(&mut self) {
        let z = self.0.mut_0();
        z.set_prefix(Prefix::Dd);
    }

    fn fd(&mut self) {
        let z = self.0.mut_0();
        z.set_prefix(Prefix::Fd);
    }

    fn cb(&mut self) {
        let z = self.0.mut_0();
        z.set_prefix(Prefix::Cb);
    }

    fn ed(&mut self) {
        let z = self.0.mut_0();
        z.set_prefix(Prefix::Ed);
    }

    fn ddcb(&mut self) {
        let z = self.0.mut_0();
        z.set_prefix(Prefix::DdCb);
    }

    fn fdcb(&mut self) {
        let z = self.0.mut_0();
        z.set_prefix(Prefix::FdCb);
    }
}

impl Impl<Z80NoImpl> for Z80State {
    type Impler = Z80NoImpler<Self>;

    #[inline(always)]
    fn make<'a>(&'a self) -> Cref<'a, Self::Impler> {
        Z80NoImpler::new(self)
    }

    #[inline(always)]
    fn make_mut<'a>(&'a mut self) -> Mref<'a, Self::Impler> {
        Z80NoImpler::new_mut(self)
    }
}
