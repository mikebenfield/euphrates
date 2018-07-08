//! Handling interrupts for the Z80

use impler::{Cref, Impl, Mref, Ref};
use memo::Inbox;
use utilities;

use super::*;

pub trait Z80Interrupt {
    fn check_interrupts(&mut self);

    fn maskable_interrupt(&mut self, x: u8);

    fn nonmaskable_interrupt(&mut self);
}

pub struct Z80InterruptImpl;

impl<T> Z80Interrupt for T
where
    T: Impl<Z80InterruptImpl> + ?Sized,
    T::Impler: Z80Interrupt,
{
    #[inline]
    fn check_interrupts(&mut self) {
        self.make_mut().check_interrupts()
    }

    #[inline]
    fn maskable_interrupt(&mut self, x: u8) {
        self.make_mut().maskable_interrupt(x)
    }

    #[inline]
    fn nonmaskable_interrupt(&mut self) {
        self.make_mut().nonmaskable_interrupt()
    }
}

pub struct Z80InterruptImpler<T: ?Sized>(Ref<T>);

impl<T: ?Sized> Z80InterruptImpler<T> {
    #[inline(always)]
    pub fn new<'a>(t: &'a T) -> Cref<'a, Self> {
        Cref::Own(Z80InterruptImpler(unsafe { Ref::new(t) }))
    }

    #[inline(always)]
    pub fn new_mut<'a>(t: &'a mut T) -> Mref<'a, Self> {
        Mref::Own(Z80InterruptImpler(unsafe { Ref::new_mut(t) }))
    }
}

impl<T> Z80Interrupt for Z80InterruptImpler<T>
where
    T: Inbox + Z80Mem + Z80Irq + Z80Internal + ?Sized,
    <T as Inbox>::Memo: From<Z80Memo>,
{
    fn check_interrupts(&mut self) {
        self.0
            .mut_0()
            .set_interrupt_status(InterruptStatus::NoCheck);

        if self.0.mut_0().requesting_nmi() {
            self.0
                .mut_0()
                .receive(From::from(Z80Memo::NonmaskableInterrupt));
            self.0.mut_0().take_nmi();
            self.nonmaskable_interrupt();
            return;
        }

        if let Some(x) = self.0.mut_0().requesting_mi() {
            let memo = From::from(Z80Memo::MaskableInterrupt {
                mode: self.0.mut_0().interrupt_mode() as u8,
                byte: x,
            });
            self.0.mut_0().receive(memo);
            self.maskable_interrupt(x);
        }
    }

    fn maskable_interrupt(&mut self, x: u8) {
        let z = &mut self.0.mut_0();
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
        let z = &mut self.0.mut_0();
        z.inc_r(1);
        z.set_iff1(false);
        z.set_prefix(Prefix::NoPrefix);
        z.inc_cycles(11);
        z.rst(0x66);
    }
}
