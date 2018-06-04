use hardware::memory16::Memory16;
use impler::{Cref, Impl, Mref, Ref};
use memo::Inbox;

use super::*;

use self::Reg16::*;

pub trait Z80Run {
    /// Run instructions until our total cycles are at least `target_cycles`.
    ///
    /// Doesn't handle interrupts. Will return early if an `ei` is executed and
    /// interrupts were not already enabled so that interrupts can be checked.
    fn run(&mut self, target_cycles: u64);
}

pub struct Z80RunImpl;

impl<T> Z80Run for T
where
    T: Impl<Z80RunImpl> + ?Sized,
    T::Impler: Z80Run,
{
    #[inline]
    fn run(&mut self, target_cycles: u64) {
        self.make_mut().run(target_cycles)
    }
}

pub struct Z80RunInterpreterImpler<T: ?Sized>(Ref<T>);

impl<T: ?Sized> Z80RunInterpreterImpler<T> {
    #[inline(always)]
    pub fn new<'a>(t: &'a T) -> Cref<'a, Self> {
        Cref::Own(Z80RunInterpreterImpler(unsafe { Ref::new(t) }))
    }

    #[inline(always)]
    pub fn new_mut<'a>(t: &'a mut T) -> Mref<'a, Self> {
        Mref::Own(Z80RunInterpreterImpler(unsafe { Ref::new_mut(t) }))
    }
}

impl<T> Z80Run for Z80RunInterpreterImpler<T>
where
    T: Z80Internal + Z80Irq + Z80Interrupt + Z80No + Z80Mem + Z80Io + Memory16 + Inbox + ?Sized,
    T::Memo: From<Z80Memo>,
{
    #[inline]
    fn run(&mut self, target_cycles: u64) {
        run(self.0.mut_0(), target_cycles)
    }
}

fn run<Z>(z: &mut Z, cycles: u64)
where
    Z: Z80Internal + Z80Irq + Z80Interrupt + Z80No + Z80Mem + Z80Io + Memory16 + Inbox + ?Sized,
    Z::Memo: From<Z80Memo>,
{
    use self::InterruptStatus::*;
    use self::Prefix::*;

    fn read_pc<Z>(z: &mut Z) -> u8
    where
        Z: Z80Internal + Memory16 + ?Sized,
    {
        let pc = PC.view(z);
        let opcode: u8 = z.read(pc);
        PC.change(z, pc.wrapping_add(1));
        opcode
    }

    z.set_interrupt_status(InterruptStatus::Check);

    while z.cycles() < cycles {
        match (z.prefix(), z.interrupt_status()) {
            (Halt, NoCheck) => {
                use std::cmp::max;
                let current_cycles = z.cycles();
                z.set_cycles(max(current_cycles, cycles));
            }
            (Halt, _) => {
                z.check_interrupts();
            }
            (NoPrefix, Check) => {
                z.check_interrupts();
            }
            (NoPrefix, Ei(ei_cycles)) if z.cycles() > ei_cycles => {
                z.check_interrupts();
            }
            (NoPrefix, _) => {
                z.inc_r(1);
                let opcode = read_pc(z);
                instructions::execute_noprefix(z, opcode);
            }
            (Cb, _) => {
                z.set_prefix(NoPrefix);
                let opcode = read_pc(z);
                instructions::execute_cb(z, opcode);
            }
            (Ed, _) => {
                z.set_prefix(NoPrefix);
                let opcode = read_pc(z);
                instructions::execute_ed(z, opcode);
            }
            (Dd, _) => {
                z.inc_r(1);
                z.set_prefix(NoPrefix);
                let opcode = read_pc(z);
                instructions::execute_dd(z, opcode);
            }
            (Fd, _) => {
                z.inc_r(1);
                z.set_prefix(NoPrefix);
                let opcode = read_pc(z);
                instructions::execute_fd(z, opcode);
            }
            (DdCb, _) => {
                z.set_prefix(NoPrefix);
                let _ = read_pc(z);
                let opcode = read_pc(z);
                instructions::execute_ddcb(z, opcode);
            }
            (FdCb, _) => {
                z.set_prefix(NoPrefix);
                let _ = read_pc(z);
                let opcode = read_pc(z);
                instructions::execute_fdcb(z, opcode);
            }
        }
    }
}
