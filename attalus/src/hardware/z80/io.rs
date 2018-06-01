use impler::{Cref, Impl, Mref, Ref};
use utilities;

use hardware::io16::Io16;
use hardware::memory16::Memory16;

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

pub struct Z80IoImpl;

impl<U> Z80Io for U
where
    U: Impl<Z80IoImpl> + ?Sized,
    U::Impler: Z80Io,
{
    #[inline]
    fn in_c(&mut self, x: Reg8, y: Reg8) {
        self.make_mut().in_c(x, y)
    }

    #[inline]
    fn in_f(&mut self, x: Reg8) {
        self.make_mut().in_f(x)
    }

    #[inline]
    fn in_n(&mut self, x: Reg8, y: u8) {
        self.make_mut().in_n(x, y)
    }

    #[inline]
    fn ind(&mut self) {
        self.make_mut().ind()
    }

    #[inline]
    fn indr(&mut self) {
        self.make_mut().indr()
    }

    #[inline]
    fn ini(&mut self) {
        self.make_mut().ini()
    }

    #[inline]
    fn inir(&mut self) {
        self.make_mut().inir()
    }

    #[inline]
    fn otdr(&mut self) {
        self.make_mut().otdr()
    }

    #[inline]
    fn otir(&mut self) {
        self.make_mut().otir()
    }

    #[inline]
    fn out_c<T>(&mut self, x: Reg8, y: T)
    where
        T: Viewable<u8>,
    {
        self.make_mut().out_c(x, y)
    }

    #[inline]
    fn out_n(&mut self, x: u8, y: Reg8) {
        self.make_mut().out_n(x, y)
    }

    #[inline]
    fn outd(&mut self) {
        self.make_mut().outd()
    }

    #[inline]
    fn outi(&mut self) {
        self.make_mut().outi()
    }
}

pub struct Z80IoImpler<T: ?Sized>(Ref<T>);

impl<T: ?Sized> Z80IoImpler<T> {
    #[inline(always)]
    pub fn new<'a>(t: &'a T) -> Cref<'a, Self> {
        Cref::Own(Z80IoImpler(unsafe { Ref::new(t) }))
    }

    #[inline(always)]
    pub fn new_mut<'a>(t: &'a mut T) -> Mref<'a, Self> {
        Mref::Own(Z80IoImpler(unsafe { Ref::new_mut(t) }))
    }
}

impl<U> Z80Io for Z80IoImpler<U>
where
    U: Z80Internal + Io16 + Memory16 + ?Sized,
{
    fn in_c(&mut self, x: Reg8, y: Reg8) {
        let z = self.0.mut_0();
        let address_lo = y.view(z);
        let address_hi = x.view(z);
        let address = utilities::to16(address_lo, address_hi);
        let val = z.input(address);
        x.change(z, val);
    }

    fn in_f(&mut self, x: Reg8) {
        in_help(self.0.mut_0(), x);
    }

    fn in_n(&mut self, x: Reg8, y: u8) {
        let z = self.0.mut_0();
        let address_lo = y.view(z);
        let address_hi = x.view(z);
        let address = utilities::to16(address_lo, address_hi);
        let val = z.input(address);
        x.change(z, val);
    }

    fn ind(&mut self) {
        let z = self.0.mut_0();
        let new_b = inid_help(z, 0xFFFF);
        z.set_zero(new_b);
        z.set_flag(NF);
    }

    fn indr(&mut self) {
        self.ind();
        let z = self.0.mut_0();
        if z.reg16(BC) != 0 {
            let pc = z.reg16(PC);
            z.set_reg16(PC, pc.wrapping_sub(2));
        }
    }

    fn ini(&mut self) {
        let z = self.0.mut_0();
        let new_b = inid_help(z, 1);

        z.set_zero(new_b);
        z.set_flag(NF);
    }

    fn inir(&mut self) {
        self.ini();
        let z = self.0.mut_0();
        if z.reg16(BC) != 0 {
            let pc = z.reg16(PC);
            z.set_reg16(PC, pc.wrapping_sub(2));
        }
    }

    fn otdr(&mut self) {
        self.outd();
        let z = self.0.mut_0();
        if z.reg8(B) != 0 {
            let pc = z.reg16(PC);
            z.set_reg16(PC, pc.wrapping_sub(2));
        }
    }

    fn otir(&mut self) {
        self.outi();
        let z = self.0.mut_0();
        if z.reg8(B) != 0 {
            let pc = z.reg16(PC);
            z.set_reg16(PC, pc.wrapping_sub(2));
        }
    }

    fn out_c<T>(&mut self, x: Reg8, y: T)
    where
        T: Viewable<u8>,
    {
        let z = self.0.mut_0();
        let address_lo = x.view(z);
        let address_hi = B.view(z);
        let address = utilities::to16(address_lo, address_hi);
        let val = y.view(z);
        z.output(address, val);

        // our output may have triggered an interrupt
        z.set_interrupt_status(InterruptStatus::Check);
    }

    fn out_n(&mut self, x: u8, y: Reg8) {
        let z = self.0.mut_0();
        let address_lo = x.view(z);
        let address_hi = A.view(z);
        let address = utilities::to16(address_lo, address_hi);
        let val = y.view(z);
        z.output(address, val);

        // our output may have triggered an interrupt
        z.set_interrupt_status(InterruptStatus::Check);
    }

    fn outd(&mut self) {
        let z = self.0.mut_0();
        outid_help(z, 0xFFFF);
        let new_b = B.view(z);

        z.set_zero(new_b);
        z.set_flag(NF);

        // our output may have triggered an interrupt
        z.set_interrupt_status(InterruptStatus::Check);
    }

    fn outi(&mut self) {
        let z = self.0.mut_0();
        outid_help(z, 1);
        let new_b = B.view(z);

        z.set_zero(new_b);
        z.set_flag(NF);

        // our output may have triggered an interrupt
        z.set_interrupt_status(InterruptStatus::Check);
    }
}
