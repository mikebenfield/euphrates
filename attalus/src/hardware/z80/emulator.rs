use impler::{ConstOrMut, Impler, ImplerImpl};

use super::*;

pub trait Z80Emulator {
    fn emulate(&mut self, target_cycles: u64);
}

pub trait Z80EmulatorImpl {
    type Impler: Z80Emulator;

    fn close<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Self::Impler) -> T;

    fn close_mut<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self::Impler) -> T;
}

impl<T> Z80Emulator for T
where
    T: Z80EmulatorImpl + ?Sized,
{
    #[inline]
    fn emulate(&mut self, target_cycles: u64) {
        self.close_mut(|z| z.emulate(target_cycles))
    }
}

pub struct Z80EmulatorImpler<T: ?Sized>(ConstOrMut<T>);

unsafe impl<T: ?Sized> ImplerImpl for Z80EmulatorImpler<T> {
    type T = T;

    #[inline]
    unsafe fn new(c: ConstOrMut<Self::T>) -> Self {
        Z80EmulatorImpler(c)
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

impl<T> Z80Emulator for Z80EmulatorImpler<T>
where
    T: Z80Internal + Z80Run + Z80Interrupt,
{
    fn emulate(&mut self, target_cycles: u64) {
        let z = self.mut_0();
        while z.cycles() < target_cycles {
            z.check_interrupts();
            z.run(target_cycles);
        }
    }
}
