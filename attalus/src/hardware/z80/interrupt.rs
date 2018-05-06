//! Handling interrupts for the Z80

use impler::{ConstOrMut, Impler, ImplerImpl};
use memo::{Inbox, InboxImpl, NothingInbox};
use utilities;

use super::*;

pub trait Z80Interrupt {
    fn check_interrupts(&mut self);

    fn maskable_interrupt(&mut self, x: u8);

    fn nonmaskable_interrupt(&mut self);
}

pub trait Z80InterruptImpl: Z80Interrupt {
    type Impler: Z80Interrupt;

    fn close<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Self::Impler) -> T;

    fn close_mut<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self::Impler) -> T;
}

impl<T> Z80Interrupt for T
where
    T: Z80InterruptImpl + ?Sized,
{
    #[inline]
    fn check_interrupts(&mut self) {
        self.close_mut(|z| z.check_interrupts())
    }

    #[inline]
    fn maskable_interrupt(&mut self, x: u8) {
        self.close_mut(|z| z.maskable_interrupt(x))
    }

    #[inline]
    fn nonmaskable_interrupt(&mut self) {
        self.close_mut(|z| z.nonmaskable_interrupt())
    }
}

pub struct Z80InterruptInboxImpler<T: ?Sized>(ConstOrMut<T>);

unsafe impl<T: ?Sized> ImplerImpl for Z80InterruptInboxImpler<T> {
    type T = T;

    #[inline]
    unsafe fn new(c: ConstOrMut<T>) -> Self {
        Z80InterruptInboxImpler(c)
    }

    #[inline]
    fn get(&self) -> &ConstOrMut<T> {
        &self.0
    }

    #[inline]
    fn get_mut(&mut self) -> &mut ConstOrMut<T> {
        &mut self.0
    }
}

impl<T> Z80Interrupt for Z80InterruptInboxImpler<T>
where
    T: Inbox + Z80Mem + Z80Irq + Z80Internal + ?Sized,
    <T as Inbox>::Memo: From<Z80Memo>,
{
    fn check_interrupts(&mut self) {
        if self.mut_0().requesting_nmi() {
            self.mut_0()
                .receive(From::from(Z80Memo::NonmaskableInterrupt));
            self.nonmaskable_interrupt();
            return;
        }

        if let InterruptStatus::Ei(_) = self.mut_0().interrupt_status() {
            return;
        }

        self.mut_0().set_interrupt_status(InterruptStatus::NoCheck);

        if let Some(x) = self.mut_0().requesting_mi() {
            let memo = From::from(Z80Memo::MaskableInterrupt {
                mode: unsafe { self.0.mut_0() }.interrupt_mode() as u8,
                byte: x,
            });
            self.mut_0().receive(memo);
            self.maskable_interrupt(x);
        }
    }

    fn maskable_interrupt(&mut self, x: u8) {
        let z = &mut unsafe { self.0.mut_0() };
        if z.iff1() {
            z.inc_r(1);

            z.set_iff1(false);
            z.set_iff2(false);
            z.set_prefix(Prefix::NoPrefix);

            match z.interrupt_mode() {
                InterruptMode::Im1 => {
                    z.rst(0x38);
                    z.inc_cycles(13);
                }
                InterruptMode::Im2 => {
                    let i = z.reg8(Reg8::I);
                    let new_pc = utilities::to16(x, i);
                    z.rst(new_pc);
                    z.inc_cycles(19);
                }
                _ => unimplemented!(),
            }
        }
    }

    fn nonmaskable_interrupt(&mut self) {
        let z = &mut unsafe { self.0.mut_0() };
        z.inc_r(1);
        z.set_iff1(false);
        z.set_prefix(Prefix::NoPrefix);
        z.inc_cycles(11);
        z.rst(0x66);
    }
}

pub struct Z80InterruptImpler<T: ?Sized>(ConstOrMut<T>);

unsafe impl<T: ?Sized> ImplerImpl for Z80InterruptImpler<T> {
    type T = T;

    #[inline]
    unsafe fn new(c: ConstOrMut<Self::T>) -> Self {
        Z80InterruptImpler(c)
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

impl<U> Z80IrqImpl for Z80InterruptImpler<U>
where
    U: Z80Irq + ?Sized,
{
    type Impler = U;

    fn close<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Self::Impler) -> T,
    {
        f(self._0())
    }

    fn close_mut<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self::Impler) -> T,
    {
        f(self.mut_0())
    }
}

impl<U> Z80InternalImpl for Z80InterruptImpler<U>
where
    U: Z80Internal + ?Sized,
{
    type Impler = U;

    fn close<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Self::Impler) -> T,
    {
        f(self._0())
    }

    fn close_mut<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self::Impler) -> T,
    {
        f(self.mut_0())
    }
}

impl<U> Z80MemImpl for Z80InterruptImpler<U>
where
    U: Z80Mem + ?Sized,
{
    type Impler = U;

    fn close<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Self::Impler) -> T,
    {
        f(self._0())
    }

    fn close_mut<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self::Impler) -> T,
    {
        f(self.mut_0())
    }
}

impl<U: ?Sized> InboxImpl for Z80InterruptImpler<U> {
    type Impler = NothingInbox<Z80Memo>;

    fn close<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Self::Impler) -> T,
    {
        f(&Default::default())
    }

    fn close_mut<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self::Impler) -> T,
    {
        f(&mut Default::default())
    }
}

impl<U> Z80InterruptImpl for Z80InterruptImpler<U>
where
    U: Z80Mem + Z80Irq + Z80Internal + ?Sized,
{
    type Impler = Z80InterruptInboxImpler<Self>;

    fn close<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Self::Impler) -> T,
    {
        Z80InterruptInboxImpler::iclose(self, |z| f(z))
    }

    fn close_mut<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self::Impler) -> T,
    {
        Z80InterruptInboxImpler::iclose_mut(self, |z| f(z))
    }
}
