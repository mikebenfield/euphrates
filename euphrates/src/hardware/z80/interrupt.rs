//! Handling interrupts for the Z80

use hardware::memory16::Memory16;
use memo::Inbox;
use utilities;

use super::*;

pub trait Z80Interrupt {
    fn check_interrupts(&mut self);

    fn maskable_interrupt(&mut self, x: u8);

    fn nonmaskable_interrupt(&mut self);
}

pub struct Z80InterruptImpler<'a, Z: 'a + ?Sized, M: 'a + ?Sized, Irq: 'a + ?Sized, I: 'a + ?Sized>
{
    pub z80: &'a mut Z,
    pub memory: &'a mut M,
    pub irq: &'a mut Irq,
    pub inbox: &'a mut I,
}

impl<'a, Z: 'a, M: 'a, Irq: 'a, I: 'a> Z80Interrupt for Z80InterruptImpler<'a, Z, M, Irq, I>
where
    Z: Z80Internal + ?Sized,
    M: Memory16 + ?Sized,
    Irq: Z80Irq + ?Sized,
    I: Inbox<Memo = Z80Memo> + ?Sized,
{
    fn check_interrupts(&mut self) {
        self.z80.set_interrupt_status(InterruptStatus::NoCheck);

        if self.irq.requesting_nmi() {
            self.inbox.receive(Z80Memo::NonmaskableInterrupt);
            self.irq.take_nmi();
            self.nonmaskable_interrupt();
            return;
        }

        if let Some(x) = self.irq.requesting_mi() {
            let memo = Z80Memo::MaskableInterrupt {
                mode: self.z80.interrupt_mode() as u8,
                byte: x,
            };
            self.inbox.receive(memo);
            self.maskable_interrupt(x);
        }
    }

    fn maskable_interrupt(&mut self, x: u8) {
        if self.z80.iff1() {
            self.z80.inc_r(1);

            self.z80.set_iff1(false);
            self.z80.set_iff2(false);
            self.z80.set_prefix(Prefix::NoPrefix);

            match self.z80.interrupt_mode() {
                InterruptMode::Im1 => {
                    Z80MemImpler {
                        z80: self.z80,
                        memory: self.memory,
                    }.rst(0x38);
                    self.z80.inc_cycles(13);
                }
                InterruptMode::Im2 => {
                    let i = self.z80.reg8(Reg8::I);
                    let new_pc = utilities::to16(x, i);
                    Z80MemImpler {
                        z80: self.z80,
                        memory: self.memory,
                    }.rst(new_pc);
                    self.z80.inc_cycles(19);
                }
                _ => unimplemented!(),
            }
        }
    }

    fn nonmaskable_interrupt(&mut self) {
        self.z80.inc_r(1);
        self.z80.set_iff1(false);
        self.z80.set_prefix(Prefix::NoPrefix);
        self.z80.inc_cycles(11);
        Z80MemImpler {
            z80: self.z80,
            memory: self.memory,
        }.rst(0x66);
    }
}
