use impler::{ConstOrMut, Impler, ImplerImpl};
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

pub trait Z80IoImpl {
    type Impler: Z80Io + ?Sized;

    fn close<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Self::Impler) -> T;

    fn close_mut<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self::Impler) -> T;
}

impl<U> Z80Io for U
where
    U: Z80IoImpl + ?Sized,
{
    #[inline]
    fn in_c(&mut self, x: Reg8, y: Reg8) {
        self.close_mut(|z| z.in_c(x, y))
    }

    #[inline]
    fn in_f(&mut self, x: Reg8) {
        self.close_mut(|z| z.in_f(x))
    }

    #[inline]
    fn in_n(&mut self, x: Reg8, y: u8) {
        self.close_mut(|z| z.in_n(x, y))
    }

    #[inline]
    fn ind(&mut self) {
        self.close_mut(|z| z.ind())
    }

    #[inline]
    fn indr(&mut self) {
        self.close_mut(|z| z.indr())
    }

    #[inline]
    fn ini(&mut self) {
        self.close_mut(|z| z.ini())
    }

    #[inline]
    fn inir(&mut self) {
        self.close_mut(|z| z.inir())
    }

    #[inline]
    fn otdr(&mut self) {
        self.close_mut(|z| z.otdr())
    }

    #[inline]
    fn otir(&mut self) {
        self.close_mut(|z| z.otir())
    }

    #[inline]
    fn out_c<T>(&mut self, x: Reg8, y: T)
    where
        T: Viewable<u8>,
    {
        self.close_mut(|z| z.out_c(x, y))
    }

    #[inline]
    fn out_n(&mut self, x: u8, y: Reg8) {
        self.close_mut(|z| z.out_n(x, y))
    }

    #[inline]
    fn outd(&mut self) {
        self.close_mut(|z| z.outd())
    }

    #[inline]
    fn outi(&mut self) {
        self.close_mut(|z| z.outi())
    }
}

pub struct Z80IoImpler<T: ?Sized>(ConstOrMut<T>);

unsafe impl<T: ?Sized> ImplerImpl for Z80IoImpler<T> {
    type T = T;

    #[inline]
    unsafe fn new(c: ConstOrMut<Self::T>) -> Self {
        Z80IoImpler(c)
    }

    #[inline]
    fn get(&self) -> &ConstOrMut<Self::T> {
        &self.0
    }

    #[inline]
    fn get_mut(&mut self) -> &mut ConstOrMut<Self::T> {
        &mut self.0
    }
}

impl<U> Z80Io for Z80IoImpler<U>
where
    U: Z80Internal + Io16 + Memory16 + ?Sized,
{
    fn in_c(&mut self, x: Reg8, y: Reg8) {
        let z = &mut self.mut_0();
        let address_lo = y.view(*z);
        let address_hi = x.view(*z);
        let address = utilities::to16(address_lo, address_hi);
        let val = z.input(address);
        x.change(*z, val);
    }

    fn in_f(&mut self, x: Reg8) {
        in_help(self.mut_0(), x);
    }

    fn in_n(&mut self, x: Reg8, y: u8) {
        let z = &mut self.mut_0();
        let address_lo = y.view(*z);
        let address_hi = x.view(*z);
        let address = utilities::to16(address_lo, address_hi);
        let val = z.input(address);
        x.change(*z, val);
    }

    fn ind(&mut self) {
        let z = &mut self.mut_0();
        let new_b = inid_help(*z, 0xFFFF);
        z.set_zero(new_b);
        z.set_flag(NF);
    }

    fn indr(&mut self) {
        self.ind();
        let z = &mut self.mut_0();
        if z.reg16(BC) != 0 {
            let pc = z.reg16(PC);
            z.set_reg16(PC, pc.wrapping_sub(2));
        }
    }

    fn ini(&mut self) {
        let z = &mut self.mut_0();
        let new_b = inid_help(*z, 1);

        z.set_zero(new_b);
        z.set_flag(NF);
    }

    fn inir(&mut self) {
        self.ini();
        let z = &mut self.mut_0();
        if z.reg16(BC) != 0 {
            let pc = z.reg16(PC);
            z.set_reg16(PC, pc.wrapping_sub(2));
        }
    }

    fn otdr(&mut self) {
        self.outd();
        let z = &mut self.mut_0();
        if z.reg8(B) != 0 {
            let pc = z.reg16(PC);
            z.set_reg16(PC, pc.wrapping_sub(2));
        }
    }

    fn otir(&mut self) {
        self.outi();
        let z = &mut self.mut_0();
        if z.reg8(B) != 0 {
            let pc = z.reg16(PC);
            z.set_reg16(PC, pc.wrapping_sub(2));
        }
    }

    fn out_c<T>(&mut self, x: Reg8, y: T)
    where
        T: Viewable<u8>,
    {
        let z = &mut self.mut_0();
        let address_lo = x.view(*z);
        let address_hi = B.view(*z);
        let address = utilities::to16(address_lo, address_hi);
        let val = y.view(*z);
        z.output(address, val);

        // our output may have triggered an interrupt
        z.set_interrupt_status(InterruptStatus::Check);
    }

    fn out_n(&mut self, x: u8, y: Reg8) {
        let z = &mut self.mut_0();
        let address_lo = x.view(*z);
        let address_hi = A.view(*z);
        let address = utilities::to16(address_lo, address_hi);
        let val = y.view(*z);
        z.output(address, val);

        // our output may have triggered an interrupt
        z.set_interrupt_status(InterruptStatus::Check);
    }

    fn outd(&mut self) {
        let z = &mut self.mut_0();
        outid_help(*z, 0xFFFF);
        let new_b = B.view(*z);

        z.set_zero(new_b);
        z.set_flag(NF);

        // our output may have triggered an interrupt
        z.set_interrupt_status(InterruptStatus::Check);
    }

    fn outi(&mut self) {
        let z = &mut self.mut_0();
        outid_help(*z, 1);
        let new_b = B.view(*z);

        z.set_zero(new_b);
        z.set_flag(NF);

        // our output may have triggered an interrupt
        z.set_interrupt_status(InterruptStatus::Check);
    }
}
