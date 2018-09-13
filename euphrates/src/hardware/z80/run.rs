use super::*;
use hardware::io16::Io16;
use hardware::memory16::Memory16;
use memo::Inbox;

use self::Reg16::*;

pub trait Z80Run {
    /// Run instructions until our total cycles are at least `target_cycles`.
    ///
    /// Doesn't handle interrupts. Will return early if an `ei` is executed and
    /// interrupts were not already enabled so that interrupts can be checked.
    fn run(&mut self, target_cycles: u64);
}

pub struct Z80RunImpler<
    'a,
    Z: 'a + ?Sized,
    M: 'a + ?Sized,
    Irq: 'a + ?Sized,
    I: 'a + ?Sized,
    Inb: 'a + ?Sized,
> {
    pub z80: &'a mut Z,
    pub memory: &'a mut M,
    pub io: &'a mut I,
    pub irq: &'a mut Irq,
    pub inbox: &'a mut Inb,
}

impl<'a, Z, M, Irq, I, Inb> instruction::Z80Emulator for Z80RunImpler<'a, Z, M, Irq, I, Inb>
where
    Z: 'a + ?Sized + Z80Internal,
    M: 'a + ?Sized + Memory16,
    Irq: 'a + ?Sized + Z80Irq,
    I: 'a + ?Sized + Io16,
    Inb: 'a + ?Sized + Inbox<Memo = Z80Memo>,
{
    type No = Z80NoImpler<Z>;

    type Mem = Z80MemImpler<Z, M>;

    type Io = Z80IoImpler<Z, M, I>;

    #[inline]
    fn no<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self::No),
    {
        f(unsafe { &mut Z80NoImpler::new(self.z80) });
    }

    #[inline]
    fn mem<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self::Mem),
    {
        f(unsafe { &mut Z80MemImpler::new(self.z80, self.memory) });
    }

    #[inline]
    fn io<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self::Io),
    {
        f(unsafe { &mut Z80IoImpler::new(self.z80, self.memory, self.io) });
    }

    #[inline]
    fn read_pc(&mut self) -> u8 {
        self.memory.read(self.z80.reg16(PC))
    }

    #[inline]
    fn inc_pc(&mut self) {
        let pc = self.z80.reg16(PC);
        self.z80.set_reg16(PC, pc.wrapping_add(1))
    }

    #[inline]
    fn inc_cycles(&mut self, c: u64) {
        let cycles = self.z80.cycles();
        self.z80.set_cycles(cycles + c);
    }
}

macro_rules! interrupt {
    ($x:expr; $var:ident; $($rest:tt)*) => {{
        let mut $var = Z80InterruptImpler {
            z80: $x.z80,
            memory: $x.memory,
            irq: $x.irq,
            inbox: $x.inbox,
        };
        {
            $($rest)*
        }
    }};
}

fn run<'a, Z: 'a, M: 'a, Irq: 'a, I: 'a, Inb: 'a>(
    z: &mut Z80RunImpler<'a, Z, M, Irq, I, Inb>,
    cycles: u64,
) where
    Z: Z80Internal + ?Sized,
    M: Memory16 + ?Sized,
    Irq: Z80Irq + ?Sized,
    I: Io16 + ?Sized,
    Inb: Inbox<Memo = Z80Memo> + ?Sized,
{
    use self::InterruptStatus::*;
    use self::Prefix::*;

    if z.z80.interrupt_status() == InterruptStatus::NoCheck {
        z.z80.set_interrupt_status(InterruptStatus::Check);
    }

    while z.z80.cycles() < cycles {
        if z.inbox.holding() {
            return;
        }
        let prefix = z.z80.prefix();
        let interrupt_status = z.z80.interrupt_status();
        let z80_cycles = z.z80.cycles();
        match (prefix, interrupt_status) {
            (Halt, NoCheck) => {
                use std::cmp::max;
                let current_cycles = z.z80.cycles();
                z.z80.set_cycles(max(current_cycles, cycles));
            }
            (Halt, _) => {
                interrupt!{z; i; i.check_interrupts()};
            }
            (NoPrefix, Check) => {
                interrupt!{z; i; i.check_interrupts()};
            }
            (NoPrefix, Ei(ei_cycles)) if z80_cycles > ei_cycles => {
                interrupt!{z; i; i.check_interrupts()};
            }
            (NoPrefix, _) => instruction::noprefix(z),
            (Cb, _) => {
                z.z80.set_prefix(NoPrefix);
                instruction::cb(z);
            }
            (Ed, _) => {
                z.z80.set_prefix(NoPrefix);
                instruction::ed(z);
            }
            (Dd, _) => {
                z.z80.inc_r(1);
                z.z80.set_prefix(NoPrefix);
                instruction::dd(z);
            }
            (Fd, _) => {
                z.z80.inc_r(1);
                z.z80.set_prefix(NoPrefix);
                instruction::fd(z);
            }
            (DdCb, _) => {
                z.z80.set_prefix(NoPrefix);
                instruction::ddcb(z);
            }
            (FdCb, _) => {
                z.z80.set_prefix(NoPrefix);
                instruction::fdcb(z);
            }
        }
    }
}

impl<'a, Z: 'a, M: 'a, Irq: 'a, I: 'a, Inb: 'a> Z80Run for Z80RunImpler<'a, Z, M, Irq, I, Inb>
where
    Z: Z80Internal + ?Sized,
    M: Memory16 + ?Sized,
    Irq: Z80Irq + ?Sized,
    I: Io16 + ?Sized,
    Inb: Inbox<Memo = Z80Memo> + ?Sized,
{
    #[inline]
    fn run(&mut self, target_cycles: u64) {
        run(self, target_cycles)
    }
}
