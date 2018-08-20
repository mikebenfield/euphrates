use super::instructions::Z80InstructionsImpler;
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

macro_rules! instructions {
    ($x:expr; $var:ident; $($rest:tt)*) => {{
        let mut $var = Z80InstructionsImpler {
            z80: $x.z80,
            memory: $x.memory,
            io: $x.io,
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

    fn read_pc<'a, Z: 'a, M: 'a, Irq: 'a, I: 'a, Inb: 'a>(
        z: &mut Z80RunImpler<'a, Z, M, Irq, I, Inb>,
    ) -> u8
    where
        Z: Z80Internal + ?Sized,
        M: Memory16 + ?Sized,
        Irq: Z80Irq + ?Sized,
        I: Io16 + ?Sized,
        Inb: Inbox<Memo = Z80Memo> + ?Sized,
    {
        let pc = z.z80.reg16(PC);
        let opcode: u8 = z.memory.read(pc);
        z.z80.set_reg16(PC, pc.wrapping_add(1));
        opcode
    }

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
            (NoPrefix, _) => {
                z.z80.inc_r(1);
                let opcode = read_pc(z);
                instructions!{z; i; instructions::execute_noprefix(&mut i, opcode)};
            }
            (Cb, _) => {
                z.z80.set_prefix(NoPrefix);
                let opcode = read_pc(z);
                instructions!{z; i; instructions::execute_cb(&mut i, opcode)};
            }
            (Ed, _) => {
                z.z80.set_prefix(NoPrefix);
                let opcode = read_pc(z);
                instructions!{z; i; instructions::execute_ed(&mut i, opcode)};
            }
            (Dd, _) => {
                z.z80.inc_r(1);
                z.z80.set_prefix(NoPrefix);
                let opcode = read_pc(z);
                instructions!{z; i; instructions::execute_dd(&mut i, opcode)};
            }
            (Fd, _) => {
                z.z80.inc_r(1);
                z.z80.set_prefix(NoPrefix);
                let opcode = read_pc(z);
                instructions!{z; i; instructions::execute_fd(&mut i, opcode)};
            }
            (DdCb, _) => {
                z.z80.set_prefix(NoPrefix);
                let _ = read_pc(z);
                let opcode = read_pc(z);
                instructions!{z; i; instructions::execute_ddcb(&mut i, opcode)};
            }
            (FdCb, _) => {
                z.z80.set_prefix(NoPrefix);
                let _ = read_pc(z);
                let opcode = read_pc(z);
                instructions!{z; i; instructions::execute_fdcb(&mut i, opcode)};
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
