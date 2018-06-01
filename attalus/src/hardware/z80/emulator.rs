use impler::{Cref, Impl, Mref, Ref};

use super::*;

pub trait Z80Emulator {
    fn emulate(&mut self, target_cycles: u64);
}

pub struct Z80EmulatorImpl;

impl<T> Z80Emulator for T
where
    T: Impl<Z80EmulatorImpl> + ?Sized,
    T::Impler: Z80Emulator,
{
    #[inline]
    fn emulate(&mut self, target_cycles: u64) {
        self.make_mut().emulate(target_cycles)
    }
}

pub struct Z80EmulatorImpler<T: ?Sized>(Ref<T>);

impl<T: ?Sized> Z80EmulatorImpler<T> {
    #[inline(always)]
    pub fn new<'a>(t: &'a T) -> Cref<'a, Self> {
        Cref::Own(Z80EmulatorImpler(unsafe { Ref::new(t) }))
    }

    #[inline(always)]
    pub fn new_mut<'a>(t: &'a mut T) -> Mref<'a, Self> {
        Mref::Own(Z80EmulatorImpler(unsafe { Ref::new_mut(t) }))
    }
}

impl<T> Z80Emulator for Z80EmulatorImpler<T>
where
    T: Z80Internal + Z80Run + Z80Interrupt,
{
    fn emulate(&mut self, target_cycles: u64) {
        let z = self.0.mut_0();
        while z.cycles() < target_cycles {
            if z.prefix() == Prefix::NoPrefix || z.prefix() == Prefix::Halt {
                z.set_interrupt_status(InterruptStatus::NoCheck);
                z.check_interrupts();
            } else {
                z.set_interrupt_status(InterruptStatus::Check);
            }
            z.run(target_cycles);
        }
    }
}
