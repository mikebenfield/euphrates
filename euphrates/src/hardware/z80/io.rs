use hardware::io16::Io16;
use hardware::memory16::Memory16;
use utilities;

use self::Reg16::*;
use self::Reg8::*;

use super::*;

/// Z80 instructions that require `Memory16` and `Io`.
pub trait Z80Io {
    fn in_c(&mut self, x: Reg8, y: Reg8);

    fn in_f(&mut self, x: Reg8);

    fn in_n(&mut self, x: Reg8, y: u8);

    fn ind(&mut self);

    fn indr(&mut self);

    fn ini(&mut self);

    fn inir(&mut self);

    fn otdr(&mut self);

    fn otir(&mut self);

    fn out_c<T>(&mut self, x: Reg8, y: T)
    where
        T: Viewable<u8>;

    fn out_n(&mut self, x: u8, y: Reg8);

    fn outd(&mut self);

    fn outi(&mut self);
}

pub struct Z80IoImpler<'a, Z: 'a + ?Sized, M: 'a + ?Sized, I: 'a + ?Sized> {
    pub z80: &'a mut Z,
    pub memory: &'a mut M,
    pub io: &'a mut I,
}

pub trait Z80IoT: Z80MemT {
    type Io: Io16 + ?Sized;

    fn io(&mut self) -> &mut Self::Io;
}

impl<'a, Z: 'a + ?Sized, M: 'a + ?Sized, I: 'a + ?Sized> Z80MemT for Z80IoImpler<'a, Z, M, I>
where
    Z: Z80Internal,
    M: Memory16,
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

impl<'a, Z: 'a, M: 'a, I: 'a> Z80IoT for Z80IoImpler<'a, Z, M, I>
where
    Z: Z80Internal + ?Sized,
    M: Memory16 + ?Sized,
    I: Io16 + ?Sized,
{
    type Io = I;

    #[inline(always)]
    fn io(&mut self) -> &mut Self::Io {
        self.io
    }
}

impl<U> Z80Io for U
where
    U: Z80IoT + ?Sized,
{
    fn in_c(&mut self, x: Reg8, y: Reg8) {
        let address_lo = y.view(self);
        let address_hi = x.view(self);
        let address = utilities::to16(address_lo, address_hi);
        let val = self.io().input(address);
        x.change(self, val);
    }

    fn in_f(&mut self, x: Reg8) {
        in_help(self, x);
    }

    fn in_n(&mut self, x: Reg8, y: u8) {
        let address_lo = y.view(self);
        let address_hi = x.view(self);
        let address = utilities::to16(address_lo, address_hi);
        let val = self.io().input(address);
        x.change(self, val);
    }

    fn ind(&mut self) {
        let new_b = inid_help(self, 0xFFFF);
        self.z80().set_zero(new_b);
        self.z80().set_flag(NF);
    }

    fn indr(&mut self) {
        self.ind();
        if self.z80().reg16(BC) != 0 {
            let pc = self.z80().reg16(PC);
            self.z80().set_reg16(PC, pc.wrapping_sub(2));
        }
    }

    fn ini(&mut self) {
        let new_b = inid_help(self, 1);

        self.z80().set_zero(new_b);
        self.z80().set_flag(NF);
    }

    fn inir(&mut self) {
        self.ini();
        if self.z80().reg16(BC) != 0 {
            let pc = self.z80().reg16(PC);
            self.z80().set_reg16(PC, pc.wrapping_sub(2));
        }
    }

    fn otdr(&mut self) {
        self.outd();
        if self.z80().reg8(B) != 0 {
            let pc = self.z80().reg16(PC);
            self.z80().set_reg16(PC, pc.wrapping_sub(2));
        }
    }

    fn otir(&mut self) {
        self.outi();

        if self.z80().reg8(B) != 0 {
            let pc = self.z80().reg16(PC);
            self.z80().set_reg16(PC, pc.wrapping_sub(2));
        }
    }

    fn out_c<T>(&mut self, x: Reg8, y: T)
    where
        T: Viewable<u8>,
    {
        let address_lo = x.view(self);
        let address_hi = B.view(self);
        let address = utilities::to16(address_lo, address_hi);
        let val = y.view(self);
        self.io().output(address, val);

        // our output may have triggered an interrupt
        self.z80().set_interrupt_status(InterruptStatus::Check);
    }

    fn out_n(&mut self, x: u8, y: Reg8) {
        let address_lo = x.view(self);
        let address_hi = A.view(self);
        let address = utilities::to16(address_lo, address_hi);
        let val = y.view(self);
        self.io().output(address, val);

        // our output may have triggered an interrupt
        self.z80().set_interrupt_status(InterruptStatus::Check);
    }

    fn outd(&mut self) {
        outid_help(self, 0xFFFF);
        let new_b = B.view(self);

        self.z80().set_zero(new_b);
        self.z80().set_flag(NF);

        // our output may have triggered an interrupt
        self.z80().set_interrupt_status(InterruptStatus::Check);
    }

    fn outi(&mut self) {
        outid_help(self, 1);
        let new_b = B.view(self);

        self.z80().set_zero(new_b);
        self.z80().set_flag(NF);

        // our output may have triggered an interrupt
        self.z80().set_interrupt_status(InterruptStatus::Check);
    }
}
