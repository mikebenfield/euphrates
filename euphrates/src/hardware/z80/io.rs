use hardware::io16::Io16;
use hardware::memory16::Memory16;
use utilities;

use self::Reg16::*;
use self::Reg8::*;

use super::*;

pub struct Z80IoImpler<Z: ?Sized, M: ?Sized, I: ?Sized> {
    z80: *mut Z,
    memory: *mut M,
    io: *mut I,
}

impl<Z: ?Sized, M: ?Sized, I: ?Sized> Z80IoImpler<Z, M, I> {
    #[inline(always)]
    pub unsafe fn new(z: &mut Z, m: &mut M, i: &mut I) -> Self {
        Z80IoImpler {
            z80: z,
            memory: m,
            io: i,
        }
    }
}

pub trait Z80IoT: Z80MemT {
    type Io: Io16 + ?Sized;

    fn io(&mut self) -> &mut Self::Io;
}

impl<Z: ?Sized, M: ?Sized, I: ?Sized> Z80MemT for Z80IoImpler<Z, M, I>
where
    Z: Z80Internal,
    M: Memory16,
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

impl<Z, M, I> Z80IoT for Z80IoImpler<Z, M, I>
where
    Z: Z80Internal + ?Sized,
    M: Memory16 + ?Sized,
    I: Io16 + ?Sized,
{
    type Io = I;

    #[inline(always)]
    fn io(&mut self) -> &mut Self::Io {
        unsafe { &mut *self.io }
    }
}

use super::instruction::instruction_traits::*;

impl<U> InC<Reg8, Reg8> for U
where
    U: Z80IoT,
{
    fn in_c(&mut self, x: Reg8, y: Reg8) {
        let address_lo = y.view(self);
        let address_hi = x.view(self);
        let address = utilities::to16(address_lo, address_hi);
        let val = self.io().input(address);
        x.change(self, val);
    }
}

impl<U> InF<Reg8> for U
where
    U: Z80IoT,
{
    fn in_f(&mut self, x: Reg8) {
        in_help(self, x);
    }
}

impl<U> InN<Reg8, u8> for U
where
    U: Z80IoT,
{
    fn in_n(&mut self, x: Reg8, y: u8) {
        let address_lo = y.view(self);
        let address_hi = x.view(self);
        let address = utilities::to16(address_lo, address_hi);
        let val = self.io().input(address);
        x.change(self, val);
    }
}

impl<U> Ind for U
where
    U: Z80IoT,
{
    fn ind(&mut self) {
        let new_b = inid_help(self, 0xFFFF);
        self.z80().set_zero(new_b);
        self.z80().set_flag(NF);
    }
}

impl<U> Indr for U
where
    U: Z80IoT,
{
    fn indr(&mut self) {
        self.ind();
        if self.z80().reg16(BC) != 0 {
            let pc = self.z80().reg16(PC);
            self.z80().set_reg16(PC, pc.wrapping_sub(2));
            self.z80().inc_cycles(21);
        } else {
            self.z80().inc_cycles(16);
        }
    }
}

impl<U> Ini for U
where
    U: Z80IoT,
{
    fn ini(&mut self) {
        let new_b = inid_help(self, 1);

        self.z80().set_zero(new_b);
        self.z80().set_flag(NF);
    }
}

impl<U> Inir for U
where
    U: Z80IoT,
{
    fn inir(&mut self) {
        self.ini();
        if self.z80().reg16(BC) != 0 {
            let pc = self.z80().reg16(PC);
            self.z80().set_reg16(PC, pc.wrapping_sub(2));
            self.z80().inc_cycles(21);
        } else {
            self.z80().inc_cycles(16);
        }
    }
}

impl<U> Otdr for U
where
    U: Z80IoT,
{
    fn otdr(&mut self) {
        self.outd();
        if self.z80().reg8(B) != 0 {
            let pc = self.z80().reg16(PC);
            self.z80().set_reg16(PC, pc.wrapping_sub(2));
            self.z80().inc_cycles(21);
        } else {
            self.z80().inc_cycles(16);
        }
    }
}

impl<U> Otir for U
where
    U: Z80IoT,
{
    fn otir(&mut self) {
        self.outi();

        if self.z80().reg8(B) != 0 {
            let pc = self.z80().reg16(PC);
            self.z80().set_reg16(PC, pc.wrapping_sub(2));
            self.z80().inc_cycles(21);
        } else {
            self.z80().inc_cycles(16);
        }
    }
}

impl<U, T> OutC<Reg8, T> for U
where
    U: Z80IoT,
    T: Viewable<u8>,
{
    fn out_c(&mut self, x: Reg8, y: T) {
        let address_lo = x.view(self);
        let address_hi = B.view(self);
        let address = utilities::to16(address_lo, address_hi);
        let val = y.view(self);
        self.io().output(address, val);

        // our output may have triggered an interrupt
        self.z80().set_interrupt_status(InterruptStatus::Check);
    }
}

impl<U> OutN<u8, Reg8> for U
where
    U: Z80IoT,
{
    fn out_n(&mut self, x: u8, y: Reg8) {
        let address_lo = x.view(self);
        let address_hi = A.view(self);
        let address = utilities::to16(address_lo, address_hi);
        let val = y.view(self);
        self.io().output(address, val);

        // our output may have triggered an interrupt
        self.z80().set_interrupt_status(InterruptStatus::Check);
    }
}

impl<U> Outd for U
where
    U: Z80IoT,
{
    fn outd(&mut self) {
        outid_help(self, 0xFFFF);
        let new_b = B.view(self);

        self.z80().set_zero(new_b);
        self.z80().set_flag(NF);

        // our output may have triggered an interrupt
        self.z80().set_interrupt_status(InterruptStatus::Check);
    }
}

impl<U> Outi for U
where
    U: Z80IoT,
{
    fn outi(&mut self) {
        outid_help(self, 1);
        let new_b = B.view(self);

        self.z80().set_zero(new_b);
        self.z80().set_flag(NF);

        // our output may have triggered an interrupt
        self.z80().set_interrupt_status(InterruptStatus::Check);
    }
}
