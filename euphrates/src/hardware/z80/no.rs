//! Z80 instructions that don't depend on memory or IO.

use super::*;

use super::instruction::instruction_traits::*;

use self::Reg16::*;
use self::Reg8::*;

pub struct Z80NoImpler<Z: ?Sized>(*mut Z);

impl<Z: ?Sized> Z80NoImpler<Z> {
    /// Caller's responsibility to make sure the result doesn't live longer than
    /// the reference
    #[inline(always)]
    pub unsafe fn new(z: &mut Z) -> Self {
        Z80NoImpler(z)
    }

    #[inline(always)]
    fn z80(&mut self) -> &mut Z {
        unsafe { &mut *self.0 }
    }
}

impl<Z> Adc16<Reg16, Reg16> for Z80NoImpler<Z>
where
    Z: Z80Internal + ?Sized,
{
    fn adc16(&mut self, x: Reg16, y: Reg16) {
        let x0 = self.z80().reg16(x);
        let y0 = self.z80().reg16(y);
        let cf = if self.z80().is_set_flag(CF) { 1u8 } else { 0u8 };
        let result = adc16_help(self.z80(), x0, y0, cf as u16);
        self.z80().set_reg16(x, result);
    }
}

impl<Z> Add16<Reg16, Reg16> for Z80NoImpler<Z>
where
    Z: Z80Internal + ?Sized,
{
    fn add16(&mut self, x: Reg16, y: Reg16) {
        let x0 = self.z80().reg16(x);
        let y0 = self.z80().reg16(y);
        let result = add16_help(self.z80(), x0, y0, 0);
        self.z80().set_reg16(x, result);
    }
}

impl<Z> Ccf for Z80NoImpler<Z>
where
    Z: Z80Internal + ?Sized,
{
    fn ccf(&mut self) {
        let cf = self.z80().is_set_flag(CF);
        self.z80().set_flag_by(HF, cf);
        self.z80().set_flag_by(CF, !cf);
        self.z80().clear_flag(NF);
    }
}

impl<Z> Cpl for Z80NoImpler<Z>
where
    Z: Z80Internal + ?Sized,
{
    fn cpl(&mut self) {
        let a = self.z80().reg8(A);
        self.z80().set_reg8(A, !a);
        self.z80().set_flag(HF | NF);
    }
}

impl<Z> Daa for Z80NoImpler<Z>
where
    Z: Z80Internal + ?Sized,
{
    fn daa(&mut self) {
        // see the table in Young
        let a = self.z80().reg8(A);
        let cf = self.z80().is_set_flag(CF);
        let hf = self.z80().is_set_flag(HF);
        let nf = self.z80().is_set_flag(NF);
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
        self.z80().set_reg8(A, new_a);

        self.z80().set_parity(new_a);
        self.z80().set_zero(new_a);
        self.z80().set_sign(new_a);
        self.z80().set_flag_by(CF, new_cf != 0);
        self.z80().set_flag_by(HF, new_hf != 0);
    }
}

impl<Z> Dec16<Reg16> for Z80NoImpler<Z>
where
    Z: Z80Internal + ?Sized,
{
    fn dec16(&mut self, x: Reg16) {
        let val = self.z80().reg16(x);
        self.z80().set_reg16(x, val.wrapping_sub(1));
    }
}

impl<Z> Di for Z80NoImpler<Z>
where
    Z: Z80Internal + ?Sized,
{
    fn di(&mut self) {
        self.z80().set_iff1(false);
        self.z80().set_iff2(false);
        self.z80().set_interrupt_status(InterruptStatus::NoCheck);
    }
}

impl<Z> Djnz<i8> for Z80NoImpler<Z>
where
    Z: Z80Internal + ?Sized,
{
    fn djnz(&mut self, e: i8) {
        let b = self.z80().reg8(B);
        let new_b = b.wrapping_sub(1);
        self.z80().set_reg8(B, new_b);
        if new_b != 0 {
            self.z80().inc_cycles(13);
            self.jr(e);
        } else {
            self.z80().inc_cycles(8);
        }
    }
}

impl<Z> Ei for Z80NoImpler<Z>
where
    Z: Z80Internal + ?Sized,
{
    fn ei(&mut self) {
        self.z80().set_iff1(true);
        self.z80().set_iff2(true);
        let cycles = self.z80().cycles().wrapping_add(4);
        self.z80().set_interrupt_status(InterruptStatus::Ei(cycles));
    }
}

impl<Z> Exx for Z80NoImpler<Z>
where
    Z: Z80Internal + ?Sized,
{
    fn exx(&mut self) {
        for &(reg1, reg2) in [(BC, BC0), (DE, DE0), (HL, HL0)].iter() {
            let val1 = self.z80().reg16(reg1);
            let val2 = self.z80().reg16(reg2);
            self.z80().set_reg16(reg1, val2);
            self.z80().set_reg16(reg2, val1);
        }
    }
}

impl<Z> Halt for Z80NoImpler<Z>
where
    Z: Z80Internal + ?Sized,
{
    fn halt(&mut self) {
        self.z80().set_prefix(Prefix::Halt);
    }
}

impl<Z> Im<u8> for Z80NoImpler<Z>
where
    Z: Z80Internal + ?Sized,
{
    fn im(&mut self, x: u8) {
        match x {
            0 => self.z80().set_interrupt_mode(InterruptMode::Im0),
            1 => self.z80().set_interrupt_mode(InterruptMode::Im1),
            2 => self.z80().set_interrupt_mode(InterruptMode::Im2),
            _ => panic!("Z80: Invalid interrupt mode"),
        }
    }
}

impl<Z> Inc16<Reg16> for Z80NoImpler<Z>
where
    Z: Z80Internal + ?Sized,
{
    fn inc16(&mut self, x: Reg16) {
        let x0 = self.z80().reg16(x);
        self.z80().set_reg16(x, x0.wrapping_add(1));
    }
}

impl<Z> Jpcc<ConditionCode, u16> for Z80NoImpler<Z>
where
    Z: Z80Internal + ?Sized,
{
    fn jpcc(&mut self, cc: ConditionCode, nn: u16) {
        let flags = self.z80().reg8(F);
        if cc.check(flags) {
            self.z80().set_reg16(PC, nn);
        }
    }
}

impl<Z> Jr<i8> for Z80NoImpler<Z>
where
    Z: Z80Internal + ?Sized,
{
    fn jr(&mut self, e: i8) {
        let pc = self.z80().reg16(PC);
        self.z80().set_reg16(PC, pc.wrapping_add(e as i16 as u16));
    }
}

impl<Z> Jrcc<ConditionCode, i8> for Z80NoImpler<Z>
where
    Z: Z80Internal + ?Sized,
{
    fn jrcc(&mut self, cc: ConditionCode, e: i8) {
        let flags = self.z80().reg8(F);
        if cc.check(flags) {
            self.z80().inc_cycles(12);
            self.jr(e);
        } else {
            self.z80().inc_cycles(7);
        }
    }
}

impl<Z> LdIr<Reg8, Reg8> for Z80NoImpler<Z>
where
    Z: Z80Internal + ?Sized,
{
    fn ld_ir(&mut self, x: Reg8, y: Reg8) {
        let y0 = self.z80().reg8(y);
        self.z80().set_reg8(x, y0);
        let iff2 = self.z80().iff2();
        self.z80().set_sign(y0);
        self.z80().set_zero(y0);
        self.z80().clear_flag(NF | HF);
        self.z80().set_flag_by(PF, iff2);
    }
}

impl<Z> Neg for Z80NoImpler<Z>
where
    Z: Z80Internal + ?Sized,
{
    fn neg(&mut self) {
        let a = self.z80().reg8(A);
        let result = sub_help(self.z80(), 0, a, 0);
        self.z80().set_reg8(A, result);
        self.z80().set_flag_by(PF, a == 0x80);
        self.z80().set_flag_by(CF, a != 0);
    }
}

impl<Z> Nop for Z80NoImpler<Z>
where
    Z: Z80Internal + ?Sized,
{
    fn nop(&mut self) {}
}

impl<Z> Sbc16<Reg16, Reg16> for Z80NoImpler<Z>
where
    Z: Z80Internal + ?Sized,
{
    fn sbc16(&mut self, x: Reg16, y: Reg16) {
        let x0 = self.z80().reg16(x);
        let y0 = self.z80().reg16(y);
        let cf = if self.z80().is_set_flag(CF) { 1u8 } else { 0u8 };
        let result = adc16_help(self.z80(), x0, !y0, (1 ^ cf) as u16);
        self.z80().set_reg16(x, result);
        let cf = self.z80().is_set_flag(CF);
        let hf = self.z80().is_set_flag(HF);
        self.z80().set_flag_by(CF, !cf);
        self.z80().set_flag_by(HF, !hf);
        self.z80().set_flag(NF);
    }
}

impl<Z> Scf for Z80NoImpler<Z>
where
    Z: Z80Internal + ?Sized,
{
    fn scf(&mut self) {
        self.z80().clear_flag(HF | NF);
        self.z80().set_flag(CF);
    }
}

impl<Z> Dd for Z80NoImpler<Z>
where
    Z: Z80Internal + ?Sized,
{
    fn dd(&mut self) {
        self.z80().set_prefix(Prefix::Dd);
    }
}

impl<Z> Fd for Z80NoImpler<Z>
where
    Z: Z80Internal + ?Sized,
{
    fn fd(&mut self) {
        self.z80().set_prefix(Prefix::Fd);
    }
}

impl<Z> Cb for Z80NoImpler<Z>
where
    Z: Z80Internal + ?Sized,
{
    fn cb(&mut self) {
        self.z80().set_prefix(Prefix::Cb);
    }
}

impl<Z> Ed for Z80NoImpler<Z>
where
    Z: Z80Internal + ?Sized,
{
    fn ed(&mut self) {
        self.z80().set_prefix(Prefix::Ed);
    }
}

impl<Z> Ddcb for Z80NoImpler<Z>
where
    Z: Z80Internal + ?Sized,
{
    fn ddcb(&mut self) {
        self.z80().set_prefix(Prefix::DdCb);
    }
}

impl<Z> Fdcb for Z80NoImpler<Z>
where
    Z: Z80Internal + ?Sized,
{
    fn fdcb(&mut self) {
        self.z80().set_prefix(Prefix::FdCb);
    }
}
