//! Z80 instructions that don't depend on memory or IO.

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

impl<T> Z80No for T
where
    T: Z80Internal + ?Sized,
{
    fn adc16(&mut self, x: Reg16, y: Reg16) {
        let x0 = self.reg16(x);
        let y0 = self.reg16(y);
        let cf = if self.is_set_flag(CF) { 1u8 } else { 0u8 };
        let result = adc16_help(self, x0, y0, cf as u16);
        self.set_reg16(x, result);
    }

    fn add16(&mut self, x: Reg16, y: Reg16) {
        let x0 = self.reg16(x);
        let y0 = self.reg16(y);
        let result = add16_help(self, x0, y0, 0);
        self.set_reg16(x, result);
    }

    fn ccf(&mut self) {
        let cf = self.is_set_flag(CF);
        self.set_flag_by(HF, cf);
        self.set_flag_by(CF, !cf);
        self.clear_flag(NF);
    }

    fn cpl(&mut self) {
        let a = self.reg8(A);
        self.set_reg8(A, !a);
        self.set_flag(HF | NF);
    }

    fn daa(&mut self) {
        // see the table in Young
        let a = self.reg8(A);
        let cf = self.is_set_flag(CF);
        let hf = self.is_set_flag(HF);
        let nf = self.is_set_flag(NF);
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
        self.set_reg8(A, new_a);

        self.set_parity(new_a);
        self.set_zero(new_a);
        self.set_sign(new_a);
        self.set_flag_by(CF, new_cf != 0);
        self.set_flag_by(HF, new_hf != 0);
    }

    fn dec16(&mut self, x: Reg16) {
        let val = self.reg16(x);
        self.set_reg16(x, val.wrapping_sub(1));
    }

    fn di(&mut self) {
        self.set_iff1(false);
        self.set_iff2(false);
        self.set_interrupt_status(InterruptStatus::NoCheck);
    }

    fn djnz(&mut self, e: i8) {
        let b = self.reg8(B);
        let new_b = b.wrapping_sub(1);
        self.set_reg8(B, new_b);
        if new_b != 0 {
            self.jr(e);
        }
    }

    fn ei(&mut self) {
        self.set_iff1(true);
        self.set_iff2(true);
        let cycles = self.cycles().wrapping_add(4);
        self.set_interrupt_status(InterruptStatus::Ei(cycles));
    }

    fn exx(&mut self) {
        for &(reg1, reg2) in [(BC, BC0), (DE, DE0), (HL, HL0)].iter() {
            let val1 = self.reg16(reg1);
            let val2 = self.reg16(reg2);
            self.set_reg16(reg1, val2);
            self.set_reg16(reg2, val1);
        }
    }

    fn halt(&mut self) {
        self.set_prefix(Prefix::Halt);
    }

    fn im(&mut self, x: u8) {
        match x {
            0 => self.set_interrupt_mode(InterruptMode::Im0),
            1 => self.set_interrupt_mode(InterruptMode::Im1),
            2 => self.set_interrupt_mode(InterruptMode::Im2),
            _ => panic!("Z80: Invalid interrupt mode"),
        }
    }

    fn inc16(&mut self, x: Reg16) {
        let x0 = self.reg16(x);
        self.set_reg16(x, x0.wrapping_add(1));
    }

    fn jpcc(&mut self, cc: ConditionCode, nn: u16) {
        let flags = self.reg8(F);
        if cc.check(flags) {
            self.set_reg16(PC, nn);
        }
    }

    fn jr(&mut self, e: i8) {
        let pc = self.reg16(PC);
        self.set_reg16(PC, pc.wrapping_add(e as i16 as u16));
    }

    fn jrcc(&mut self, cc: ConditionCode, e: i8) {
        let flags = self.reg8(F);
        if cc.check(flags) {
            self.jr(e);
        }
    }

    fn ld_ir(&mut self, x: Reg8, y: Reg8) {
        let y0 = self.reg8(y);
        self.set_reg8(x, y0);
        let iff2 = self.iff2();
        self.set_sign(y0);
        self.set_zero(y0);
        self.clear_flag(NF | HF);
        self.set_flag_by(PF, iff2);
    }

    fn neg(&mut self) {
        let a = self.reg8(A);
        let result = sub_help(self, 0, a, 0);
        self.set_reg8(A, result);
        self.set_flag_by(PF, a == 0x80);
        self.set_flag_by(CF, a != 0);
    }

    fn nop(&mut self) {}

    fn sbc16(&mut self, x: Reg16, y: Reg16) {
        let x0 = self.reg16(x);
        let y0 = self.reg16(y);
        let cf = if self.is_set_flag(CF) { 1u8 } else { 0u8 };
        let result = adc16_help(self, x0, !y0, (1 ^ cf) as u16);
        self.set_reg16(x, result);
        let cf = self.is_set_flag(CF);
        let hf = self.is_set_flag(HF);
        self.set_flag_by(CF, !cf);
        self.set_flag_by(HF, !hf);
        self.set_flag(NF);
    }

    fn scf(&mut self) {
        self.clear_flag(HF | NF);
        self.set_flag(CF);
    }

    fn dd(&mut self) {
        self.set_prefix(Prefix::Dd);
    }

    fn fd(&mut self) {
        self.set_prefix(Prefix::Fd);
    }

    fn cb(&mut self) {
        self.set_prefix(Prefix::Cb);
    }

    fn ed(&mut self) {
        self.set_prefix(Prefix::Ed);
    }

    fn ddcb(&mut self) {
        self.set_prefix(Prefix::DdCb);
    }

    fn fdcb(&mut self) {
        self.set_prefix(Prefix::FdCb);
    }
}
