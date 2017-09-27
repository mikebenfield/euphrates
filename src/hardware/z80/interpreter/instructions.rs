// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use ::message::{Receiver, Sender};
use ::bits::*;
use ::hardware::z80::types::*;
use ::hardware::io::Io;

/// Most of the functions in the rotate and shift group have similar addressing modes,
/// implementations, and flag behavior, so we write a macro to generate the
/// required functions in each case.
macro_rules! rotate_shift_functions_noa_impl {
    ($fn_impl: ident $fn_impl2: ident
    $fn_general: ident $fn_store: ident) => {
        fn $fn_impl2<T1, R>(&mut self, receiver: &mut R, arg: T1) -> (u8, Flags)
        where
            R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
            I: Io<R>,
            T1: Settable<u8>
        {
            let a = arg.get(receiver, self);
            let (result, mut f) = self.$fn_impl(receiver, a);
            arg.set(receiver, self, result);
            f.remove(HF | NF);
            (result, f)
        }

        pub fn $fn_general<T1, R>(&mut self, receiver: &mut R, arg: T1)
        where
            R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
            I: Io<R>,
            T1: Settable<u8>
        {
            let (result, mut f) = self.$fn_impl2(receiver, arg);
            f.set_parity(result);
            f.set_sign(result);
            f.set_zero(result);
            F.set(receiver, self, f.bits());
        }

        pub fn $fn_store<T1, R>(&mut self, receiver: &mut R, arg: T1, store: Reg8)
        where
            R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
            I: Io<R>,
            T1: Settable<u8>
        {
            let (result, mut f) = self.$fn_impl2(receiver, arg);
            f.set_parity(result);
            f.set_sign(result);
            f.set_zero(result);
            F.set(receiver, self, f.bits());
            store.set(receiver, self, result);
        }
    }
}

macro_rules! rotate_shift_functions_impl {
    ($fn_impl: ident $fn_impl2: ident $fn_general: ident
    $fn_store: ident $fn_a: ident) => {
        pub fn $fn_a<R>(&mut self, receiver: &mut R)
        where
            R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
            I: Io<R>,
        {
            let (_, f) = self.$fn_impl2(receiver, A);
            F.set(receiver, self, f.bits());
        }
        rotate_shift_functions_noa_impl!{$fn_impl $fn_impl2 $fn_general $fn_store}
    }
}

impl<I> Z80<I>
{
    pub fn rst<R>(&mut self, receiver: &mut R, p: u16)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let sp = SP.get(receiver, self);
        let pch = PCH.get(receiver, self);
        let pcl = PCL.get(receiver, self);
        Address(sp.wrapping_sub(1)).set(receiver, self, pch);
        Address(sp.wrapping_sub(2)).set(receiver, self, pcl);
        SP.set(receiver, self, sp.wrapping_sub(2));
        PC.set(receiver, self, p);
    }

    pub fn nonmaskable_interrupt<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        // The Z80 manual implies that IFF2 is set to IFF1, but this
        // is false (see Young 5.3)
        receiver.receive(self.id(), Z80Message::NonmaskableInterrupt);
        inc_r(self);
        self.iff1 = 0xFFFFFFFFFFFFFFFF;
        self.io.clear_nmi();
        self.cycles += 11;
        if self.halted {
            let pc = PC.get(receiver, self);
            PC.set(receiver, self, pc.wrapping_add(1));
        }
        self.rst(receiver, 0x66);
    }

    pub fn maskable_interrupt<R>(&mut self, receiver: &mut R, x: u8) -> bool
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        if self.iff1 < self.cycles {
            receiver.receive(self.id(), Z80Message::MaskableInterruptAllowed);

            inc_r(self);

            self.iff1 = 0xFFFFFFFFFFFFFFFF;
            self.iff2 = false;

            if self.halted {
                let pc = PC.get(receiver, self);
                PC.set(receiver, self, pc.wrapping_add(1));
            }

            let im = self.interrupt_mode;
            match im {
                Im1 => {
                    self.rst(receiver, 0x38);
                    self.cycles += 13;
                },
                Im2 => {
                    let i = I.get(receiver, self);
                    let new_pc = to16(x, i);
                    self.rst(receiver, new_pc);
                    self.cycles += 19;
                }
                _ => unimplemented!(),
            }
            true
        } else {
            receiver.receive(self.id(), Z80Message::MaskableInterruptDenied);
            false
        }
    }

    //// 8-Bit Load Group
    /////////////////////

    pub fn ld<T1, T2, R>(&mut self, receiver: &mut R, arg1: T1, arg2: T2)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
        T1: Settable<u8>,
        T2: Gettable<u8>,
    {
        let val = arg2.get(receiver, self);
        arg1.set(receiver, self, val);
    }

    // XXX text about interrupts in manual
    pub fn ld_ir<R>(&mut self, receiver: &mut R, arg1: Reg8, arg2: Reg8)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let val = arg2.get(receiver, self);
        arg1.set(receiver, self, val);
        let iff2 = self.iff2;
        let mut f = self.flags();
        f.set_sign(val);
        f.set_zero(val);
        f.remove(NF | HF);
        f.set(PF, iff2);
        self.set_flags(f);
    }

    //// 16-Bit Load Group
    //////////////////////

    pub fn ld16<T1, T2, R>(&mut self, receiver: &mut R, arg1: T1, arg2: T2)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
        T1: Settable<u16>,
        T2: Gettable<u16>,
    {
        let val = arg2.get(receiver, self);
        arg1.set(receiver, self, val);
    }

    pub fn push<R>(&mut self, receiver: &mut R, reg: Reg16)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let (lo, hi) = to8(reg.get(receiver, self));
        let sp = SP.get(receiver, self);
        Address(sp.wrapping_sub(1)).set(receiver, self, hi);
        Address(sp.wrapping_sub(2)).set(receiver, self, lo);
        SP.set(receiver, self, sp.wrapping_sub(2));
    }

    pub fn pop<R>(&mut self, receiver: &mut R, reg: Reg16)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let sp = SP.get(receiver, self);
        let lo = Address(sp).get(receiver, self);
        let hi = Address(sp.wrapping_add(1)).get(receiver, self);
        reg.set(receiver, self, to16(lo, hi));
        SP.set(receiver, self, sp.wrapping_add(2));
    }

    //// Exchange, Block Transfer, and Search Group
    ///////////////////////////////////////////////

    pub fn ex<T1, R>(&mut self, receiver: &mut R, reg1: T1, reg2: Reg16)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
        T1: Settable<u16>,
    {
        let val1 = reg1.get(receiver, self);
        let val2 = reg2.get(receiver, self);
        reg1.set(receiver, self, val2);
        reg2.set(receiver, self, val1);
    }

    pub fn exx<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        for &(reg1, reg2) in [(BC, BC0), (DE, DE0), (HL, HL0)].iter() {
            let val1 = reg1.get(receiver, self);
            let val2 = reg2.get(receiver, self);
            reg1.set(receiver, self, val2);
            reg2.set(receiver, self, val1);
        }
    }

    fn ldid<R>(&mut self, receiver: &mut R, inc: u16)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let hl = HL.get(receiver, self);
        let de = DE.get(receiver, self);
        let bc = BC.get(receiver, self);

        let phl = Gettable::<u8>::get(Address(hl), receiver, self);
        Address(de).set(receiver, self, phl);

        HL.set(receiver, self, hl.wrapping_add(inc));
        DE.set(receiver, self, de.wrapping_add(inc));
        BC.set(receiver, self, bc.wrapping_sub(1));

        let mut f = self.flags();
        f.remove(HF | NF);
        f.set(PF, bc != 1);
        F.set(receiver, self, f.bits());
    }

    pub fn ldi<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        self.ldid(receiver, 1);
    }

    pub fn ldd<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        self.ldid(receiver, 0xFFFF);
    }

    pub fn ldir<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        loop {
            self.ldi(receiver);
            if BC.get(receiver, self) == 0 {
                self.cycles += 17;
                return;
            }
            self.cycles += 21;

            // check the possibility that we have overwritten our own opcode
            let pc = PC.get(receiver, self);
            let apc1 = Gettable::<u8>::get(Address(pc.wrapping_sub(2)), receiver, self);
            let apc2 = Gettable::<u8>::get(Address(pc.wrapping_sub(1)), receiver, self);
            if apc1 != 0xED || apc2 != 0xB0 {
                PC.set(receiver, self, pc.wrapping_sub(2));
                return;
            }
            inc_r(self);
            inc_r(self);
        }
    }

    pub fn lddr<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        loop {
            self.ldd(receiver);
            if BC.get(receiver, self) == 0 {
                self.cycles += 17;
                return;
            }
            self.cycles += 21;

            // check the possibility that we have overwritten our own opcode
            let pc = PC.get(receiver, self);
            let apc1 = Gettable::<u8>::get(Address(pc.wrapping_sub(2)), receiver, self);
            let apc2 = Gettable::<u8>::get(Address(pc.wrapping_sub(1)), receiver, self);
            if apc1 != 0xED || apc2 != 0xB8 {
                PC.set(receiver, self, pc.wrapping_sub(1));
                return;
            }
            inc_r(self);
            inc_r(self);
        }
    }

    fn cpid<R>(&mut self, receiver: &mut R, inc: u16)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let bc = BC.get(receiver, self);
        let a = A.get(receiver, self);
        let hl = HL.get(receiver, self);

        let phl: u8 = Address(HL).get(receiver, self);
        let result = a.wrapping_sub(phl);

        HL.set(receiver, self, hl.wrapping_add(inc));
        BC.set(receiver, self, bc.wrapping_sub(1));

        let mut f = self.flags();
        f.set_sign(result);
        f.set_zero(result);
        f.set(HF, phl & 0xF > a & 0xF);
        f.set(PF, bc != 1);
        f.insert(NF);
        F.set(receiver, self, f.bits());
    }

    pub fn cpi<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        self.cpid(receiver, 1);
    }

    pub fn cpir<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        while {
            self.cpi(receiver);
            self.cycles += 21;
            BC.get(receiver, self) != 0 && !self.flags().contains(ZF)
        } {
            // r was already incremented twice by `run`
            inc_r(self);
            inc_r(self);
        }

        self.cycles += 17;
    }

    pub fn cpd<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        self.cpid(receiver, 0xFFFF);
    }

    pub fn cpdr<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        while {
            self.cpd(receiver);
            self.cycles += 21;
            BC.get(receiver, self) != 0 && !self.flags().contains(ZF)
        } {
            // r was already incremented twice by `run`
            inc_r(self);
            inc_r(self);
        }

        self.cycles += 17;
    }

    //// 8-Bit Arithmetic Group
    ///////////////////////////

    fn add_impl<R>(&mut self, _receiver: &mut R, a: u8, x: u8, cf: u8) -> (u8, Flags)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        // XXX optimize?
        let result16 = (x as u16).wrapping_add(a as u16).wrapping_add(cf as u16);
        let result8 = result16 as u8;

        let mut f = self.flags();
        f.set_zero(result8);
        f.set_sign(result8);

        f.set(CF, result16 & (1 << 8) != 0);

        // carry from bit 3 happened if:
        // x and a have same bit 4 AND result is set OR
        // x and a have different bit 4 AND result is clear
        let hf = (x ^ a ^ result8) & (1 << 4) != 0;
        f.set(HF, hf);

        // overflow happened if:
        // x and a both have bit 7 AND result does not OR
        // x and a have clear bit 7 AND result is set
        // in other words, x and y have the same bit 7 and
        // result is different
        let overflow = !(x ^ a) & (x ^ result8) & (1 << 7) != 0;
        f.set(PF, overflow);

        f.remove(NF);

        (result8, f)
    }

    pub fn add<T1, T2, R>(&mut self, receiver: &mut R, arg1: T1, arg2: T2)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
        T1: Settable<u8>,
        T2: Gettable<u8>,
    {
        let a = arg1.get(receiver, self);
        let b = arg2.get(receiver, self);
        let (result, f) = self.add_impl(receiver, a, b, 0);
        arg1.set(receiver, self, result);
        F.set(receiver, self, f.bits());
    }

    pub fn adc<T1, T2, R>(&mut self, receiver: &mut R, arg1: T1, arg2: T2)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
        T1: Settable<u8>,
        T2: Gettable<u8>,
    {
        let cf = if self.flags().contains(CF) { 1u8 } else { 0u8 };
        let a = arg1.get(receiver, self);
        let x = arg2.get(receiver, self);
        let (result, f) = self.add_impl(receiver, a, x, cf);
        arg1.set(receiver, self, result);
        F.set(receiver, self, f.bits());
    }

    fn sub_impl<R>(&mut self, receiver: &mut R, a: u8, x: u8, cf: u8) -> (u8, Flags)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let (result, mut f) = self.add_impl(receiver, a, !x, 1 ^ cf);
        f.toggle(CF | HF);
        f.insert(NF);
        (result, f)
    }

    pub fn sub<T1, T2, R>(&mut self, receiver: &mut R, arg1: T1, arg2: T2)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
        T1: Settable<u8>,
        T2: Gettable<u8>,
    {
        let a = arg1.get(receiver, self);
        let x = arg2.get(receiver, self);
        let (result, f) = self.sub_impl(receiver, a, x, 0);
        arg1.set(receiver, self, result);
        F.set(receiver, self, f.bits());
    }

    pub fn sbc<T1, T2, R>(&mut self, receiver: &mut R, arg1: T1, arg2: T2)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
        T1: Settable<u8>,
        T2: Gettable<u8>,
    {
        let cf = if self.flags().contains(CF) { 1u8 } else { 0u8 };
        let a = arg1.get(receiver, self);
        let x = arg2.get(receiver, self);
        let (result, f) = self.sub_impl(receiver, a, x, cf);
        arg1.set(receiver, self, result);
        F.set(receiver, self, f.bits());
    }

    fn andor_impl<R>(&mut self, receiver: &mut R, result: u8) -> Flags
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        A.set(receiver, self, result);

        // note that for AND and OR, the manual says PF is set according to whether
        // there is overflow. I'm betting that is a mistake.
        let mut f = self.flags();
        f.set_parity(result);
        f.set_sign(result);
        f.set_zero(result);
        f.remove(HF | NF | CF);
        f
    }

    pub fn and<T1, R>(&mut self, receiver: &mut R, arg: T1)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
        T1: Gettable<u8>,
    {
        let result = arg.get(receiver, self) & A.get(receiver, self);
        let mut f = self.andor_impl(receiver, result);
        f.insert(HF);
        F.set(receiver, self, f.bits());
    }

    pub fn or<T1, R>(&mut self, receiver: &mut R, arg: T1)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
        T1: Gettable<u8>,
    {
        let result = arg.get(receiver, self) | A.get(receiver, self);
        let f = self.andor_impl(receiver, result);
        F.set(receiver, self, f.bits());
    }

    pub fn xor<T1, R>(&mut self, receiver: &mut R, arg: T1)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
        T1: Gettable<u8>,
    {
        let result = arg.get(receiver, self) ^ A.get(receiver, self);
        let f = self.andor_impl(receiver, result);
        F.set(receiver, self, f.bits());
    }

    pub fn cp<T1, R>(&mut self, receiver: &mut R, arg: T1)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
        T1: Gettable<u8>,
    {
        let x = arg.get(receiver, self);
        let a = A.get(receiver, self);
        let (_, f) = self.sub_impl(receiver, a, x, 0);
        A.set(receiver, self, a);
        F.set(receiver, self, f.bits());
    }

    pub fn inc<T1, R>(&mut self, receiver: &mut R, arg: T1)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
        T1: Settable<u8>,
    {
        let x = arg.get(receiver, self);
        let result = x.wrapping_add(1);
        arg.set(receiver, self, result);
        let mut f = self.flags();
        f.set_zero(result);
        f.set_sign(result);
        f.set(HF, x & 0xF == 0xF);
        f.set(PF, x == 0x7F);
        f.remove(NF);
        F.set(receiver, self, f.bits());
    }

    pub fn dec<T1, R>(&mut self, receiver: &mut R, arg: T1)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
        T1: Settable<u8>,
    {
        let x = arg.get(receiver, self);
        let result = x.wrapping_sub(1);
        arg.set(receiver, self, result);
        let mut f = self.flags();
        f.set_zero(result);
        f.set_sign(result);
        f.set(HF, x & 0xF == 0);
        f.set(PF, x == 0x80);
        f.insert(NF);
        F.set(receiver, self, f.bits());
    }

    //// General-Purpose Arithmetic and CPU Control Groups
    //////////////////////////////////////////////////////

    pub fn daa<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        // see the table in Young
        let a = A.get(receiver, self);
        let cf = self.flags().contains(CF);
        let hf = self.flags().contains(HF);
        let nf = self.flags().contains(NF);
        let diff = match (cf, a >> 4, hf, a & 0xF) {
            (false, 0...9, false, 0...9) => 0,
            (false, 0...9, true, 0...9) => 0x6,
            (false, 0...8, _, 0xA...0xF) => 0x6,
            (false, 0xA...0xF, false, 0...9) => 0x60,
            (true, _, false, 0...9) => 0x60,
            _ => 0x66,
        };

        let new_cf = match (cf, a >> 4, a & 0xF) {
            (false, 0...9, 0...9) => 0,
            (false, 0...8, 0xA...0xF) => 0,
            _ => 1,
        };

        let new_hf = match (nf, hf, a & 0xF) {
            (false, _, 0xA...0xF) => 1,
            (true, true, 0...5) => 1,
            _ => 0,
        };

        let new_a = if nf {
            a.wrapping_sub(diff)
        } else {
            a.wrapping_add(diff)
        };
        A.set(receiver, self, new_a);
        
        let mut f = self.flags();
        f.set_parity(new_a);
        f.set_zero(new_a);
        f.set_sign(new_a);
        f.set(CF, new_cf != 0);
        f.set(HF, new_hf != 0);
        F.set(receiver, self, f.bits());
    }

    pub fn cpl<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let a = A.get(receiver, self);
        A.set(receiver, self, !a);
        let mut f = self.flags();
        f.insert(HF | NF);
        F.set(receiver, self, f.bits());
    }

    pub fn neg<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        // subtracts A from 0
        let a = A.get(receiver, self);
        let (result, mut f) = self.sub_impl(receiver, 0, a, 0);
        A.set(receiver, self, result);
        f.set(PF, a == 0x80);
        f.set(CF, a != 0);
        F.set(receiver, self, f.bits());
    }

    pub fn ccf<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let mut f = self.flags();
        let cf = f.contains(CF);
        f.set(HF, cf);
        f.toggle(CF);
        f.remove(NF);
        F.set(receiver, self, f.bits());
    }

    pub fn scf<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let mut f = self.flags();
        f.remove(HF | NF);
        f.insert(CF);
        F.set(receiver, self, f.bits());
    }

    pub fn nop<R>(&mut self, _receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
    }

    pub fn halt<R>(&mut self, _receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        self.halted = true;
    }

    pub fn di<R>(&mut self, _receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        self.iff1 = 0xFFFFFFFFFFFFFFFF;
        self.iff2 = false;
    }

    pub fn ei<R>(&mut self, _receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        self.iff1 = self.cycles + 4;
        self.iff2 = true;
    }

    pub fn im<R>(&mut self, _receiver: &mut R, m: u8)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        match m {
            0 => self.interrupt_mode = Im0,
            1 => self.interrupt_mode = Im1,
            2 => self.interrupt_mode = Im2,
            _ => panic!("Z80: Invalid interrupt mode"),
        }
    }

    pub fn im1<R>(&mut self, _receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        self.interrupt_mode = Im1;
    }

    pub fn im2<R>(&mut self, _receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        self.interrupt_mode = Im2;
    }

    //// 16-Bit Arithmetic Group
    ////////////////////////////

    fn add16_impl<R>(&mut self, _receiver: &mut R, x: u16, y: u16, cf: u16) -> (u16, Flags)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        // XXX optimiselfe?
        let result32 = (x as u32).wrapping_add(y as u32).wrapping_add(cf as u32);
        let result16 = result32 as u16;

        let mut f = self.flags();
        f.set(CF, result32 & (1 << 16) != 0);

        // carry from bit 11 happened if:
        // x and y have same bit 12 AND result is set OR
        // x and y have different bit 12 AND result is clear
        let hf = (x ^ y ^ result16) & (1 << 12) != 0;
        f.set(HF, hf);

        f.remove(NF);

        (result16, f)
    }

    pub fn add16<R>(&mut self, receiver: &mut R, arg1: Reg16, arg2: Reg16)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let x = arg1.get(receiver, self);
        let y = arg2.get(receiver, self);
        let (result, f) = self.add16_impl(receiver, x, y, 0);
        arg1.set(receiver, self, result);
        F.set(receiver, self, f.bits());
    }

    fn adc16_impl<R>(&mut self, receiver: &mut R, x: u16, y: u16, cf: u16) -> (u16, Flags)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let (result, mut f) = self.add16_impl(receiver, x, y, cf as u16);

        f.set_sign((result >> 8) as u8);
        f.set_zero((result as u8) | (result >> 8) as u8);

        // overflow happened if:
        // x and y both have bit 15 AND result does not OR
        // x and y have clear bit 15 AND result is set
        // in other words, x and y have the same bit 15, which is different from bit
        // 15 of result
        let overflow = !(x ^ y) & (x ^ result) & (1 << 15) != 0;
        f.set(PF, overflow);

        (result, f)
    }

    pub fn adc16<R>(&mut self, receiver: &mut R, arg1: Reg16, arg2: Reg16)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let x = arg1.get(receiver, self);
        let y = arg2.get(receiver, self);
        let cf = if self.flags().contains(CF) { 1u8 } else { 0u8 };
        let (result, f) = self.adc16_impl(receiver, x, y, cf as u16);
        arg1.set(receiver, self, result);
        F.set(receiver, self, f.bits());
    }

    pub fn sbc16<R>(&mut self, receiver: &mut R, arg1: Reg16, arg2: Reg16)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let x = arg1.get(receiver, self);
        let y = arg2.get(receiver, self);
        let cf = if self.flags().contains(CF) { 1u8 } else { 0u8 };
        let (result, mut f) = self.adc16_impl(receiver, x, !y, (1 ^ cf) as u16);
        arg1.set(receiver, self, result);
        f.toggle(CF | HF);
        f.insert(NF);
        F.set(receiver, self, f.bits());
    }

    pub fn inc16<R>(&mut self, receiver: &mut R, arg: Reg16)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let val = arg.get(receiver, self);
        arg.set(receiver, self, val.wrapping_add(1));
    }

    pub fn dec16<R>(&mut self, receiver: &mut R, arg: Reg16)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let val = arg.get(receiver, self);
        arg.set(receiver, self, val.wrapping_sub(1));
    }

    //// Rotate and Shift Group
    ///////////////////////////

    fn rlc_impl<R>(&mut self, _receiver: &mut R, x: u8) -> (u8, Flags)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let mut f = self.flags();
        f.set(CF, x & 0x80 != 0);
        (x.rotate_left(1), f)
    }

    rotate_shift_functions_impl!{
        rlc_impl rlc_impl2 rlc rlc_store rlca
    }

    fn rl_impl<R>(&mut self, _receiver: &mut R, x: u8) -> (u8, Flags)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let mut f = self.flags();
        let mut result = x << 1;
        if f.contains(CF) {
            result |= 1;
        } else {
            result &= !1;
        }
        f.set(CF, x & 0x80 != 0);
        (result, f)
    }

    rotate_shift_functions_impl!{
        rl_impl rl_impl2 rl rl_store rla
    }

    fn rrc_impl<R>(&mut self, _receiver: &mut R, x: u8) -> (u8, Flags)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let mut f = self.flags();
        f.set(CF, x & 1 != 0);
        (x.rotate_right(1), f)
    }

    rotate_shift_functions_impl!{
        rrc_impl rrc_impl2 rrc rrc_store rrca
    }

    fn rr_impl<R>(&mut self, _receiver: &mut R, x: u8) -> (u8, Flags)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let mut f = self.flags();
        let mut result = x >> 1;
        if f.contains(CF) {
            result |= 0x80;
        } else {
            result &= !0x80;
        }
        f.set(CF, x & 1 != 0);
        (result, f)
    }

    rotate_shift_functions_impl!{
        rr_impl rr_impl2 rr rr_store rra
    }

    fn sla_impl<R>(&mut self, _receiver: &mut R, x: u8) -> (u8, Flags)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let mut f = self.flags();
        f.set(CF, x & 0x80 != 0);
        (x << 1, f)
    }

    rotate_shift_functions_noa_impl!{
        sla_impl sla_impl2 sla sla_store
    }

    // SLL is undocumented; see Young
    fn sll_impl<R>(&mut self, _receiver: &mut R, x: u8) -> (u8, Flags)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let mut f = self.flags();
        f.set(CF, x & 0x80 != 0);
        let mut result = x << 1;
        result |= 1;
        (result, f)
    }

    rotate_shift_functions_noa_impl!{
        sll_impl sll_impl2 sll sll_store
    }

    fn sra_impl<R>(&mut self, _receiver: &mut R, x: u8) -> (u8, Flags)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let mut f = self.flags();
        f.set(CF, x & 1 != 0);
        let result = ((x as i8) >> 1) as u8;
        (result, f)
    }

    rotate_shift_functions_noa_impl!{
        sra_impl sra_impl2 sra sra_store
    }

    fn srl_impl<R>(&mut self, _receiver: &mut R, x: u8) -> (u8, Flags)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let mut f = self.flags();
        f.set(CF, x & 1 != 0);
        (x >> 1, f)
    }

    rotate_shift_functions_noa_impl!{
        srl_impl srl_impl2 srl srl_store
    }

    pub fn rld<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let hl: u8 = Address(HL).get(receiver, self);
        let hl_lo: u8 = 0xF & hl;
        let hl_hi: u8 = 0xF0 & hl;
        let a_lo = 0xF & A.get(receiver, self);
        let a_hi = 0xF0 & A.get(receiver, self);
        Address(HL).set(receiver, self, hl_lo << 4 | a_lo);
        A.set(receiver, self, hl_hi >> 4 | a_hi);
        let a = A.get(receiver, self);

        let mut f = self.flags();
        f.set_parity(a);
        f.set_sign(a);
        f.set_zero(a);
        f.remove(HF | NF);
        F.set(receiver, self, f.bits());
    }

    pub fn rrd<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let hl: u8 = Address(HL).get(receiver, self);
        let hl_lo: u8 = 0xF & hl;
        let hl_hi: u8 = 0xF0 & hl;
        let a_lo = 0xF & A.get(receiver, self);
        let a_hi = 0xF0 & A.get(receiver, self);
        Address(HL).set(receiver, self, a_lo << 4 | hl_hi >> 4);
        A.set(receiver, self, hl_lo | a_hi);
        let a = A.get(receiver, self);

        let mut f = self.flags();
        f.set_parity(a);
        f.set_sign(a);
        f.set_zero(a);
        f.remove(HF | NF);
        F.set(receiver, self, f.bits());
    }

    //// Bit Set, Reset, and Test Group
    ///////////////////////////////////

    pub fn bit<T, R>(&mut self, receiver: &mut R, b: u8, arg: T)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
        T: Gettable<u8>,
    {
        let x = arg.get(receiver, self);
        let bitflag = 1 << b;
        let x_contains = x & bitflag != 0;

        let mut f = self.flags();
        f.set(ZF | PF, !x_contains);
        f.insert(HF);
        f.remove(NF);
        f.set(SF, b == 7 && x_contains);
        F.set(receiver, self, f.bits());
    }

    pub fn set<T, R>(&mut self, receiver: &mut R, b: u8, arg: T)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
        T: Settable<u8>,
    {
        let mut x = arg.get(receiver, self);
        set_bit(&mut x, b);
        arg.set(receiver, self, x);
    }

    pub fn set_store<T, R>(&mut self, receiver: &mut R, b: u8, arg: T, reg: Reg8)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
        T: Settable<u8>,
    {
        self.set(receiver, b, arg);
        let x = arg.get(receiver, self);
        reg.set(receiver, self, x);
    }

    pub fn res<T, R>(&mut self, receiver: &mut R, b: u8, arg: T)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
        T: Settable<u8>,
    {
        let mut x = arg.get(receiver, self);
        clear_bit(&mut x, b);
        arg.set(receiver, self, x);
    }

    pub fn res_store<T, R>(&mut self, receiver: &mut R, b: u8, arg: T, reg: Reg8)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
        T: Settable<u8>,
    {
        self.res(receiver, b, arg);
        let x = arg.get(receiver, self);
        reg.set(receiver, self, x);
    }

    //// Jump Group
    ///////////////

    pub fn jp<T, R>(&mut self, receiver: &mut R, arg: T)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
        T: Gettable<u16>,
    {
        let addr = arg.get(receiver, self);
        PC.set(receiver, self, addr);
    }

    pub fn jpcc<R>(&mut self, receiver: &mut R, cc: ConditionCode, arg: u16)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        if cc.get(receiver, self) {
            self.jp(receiver, arg);
        }
    }

    pub fn jr<R>(&mut self, receiver: &mut R, e: i8)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let pc = PC.get(receiver, self);
        let new_pc = pc.wrapping_add(e as i16 as u16);
        PC.set(receiver, self, new_pc);
    }

    pub fn jrcc<R>(&mut self, receiver: &mut R, cc: ConditionCode, e: i8)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        if cc.get(receiver, self) {
            self.jr(receiver, e);
            self.cycles += 12;
        } else {
            self.cycles += 7;
        }
    }

    pub fn djnz<R>(&mut self, receiver: &mut R, e: i8)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let b = B.get(receiver, self);
        let new_b = b.wrapping_sub(1);
        B.set(receiver, self, new_b);
        if new_b != 0 {
            self.jr(receiver, e);
            self.cycles += 13;
        } else {
            self.cycles += 8;
        }
    }

    //// Call and Return Group
    //////////////////////////

    pub fn call<R>(&mut self, receiver: &mut R, nn: u16)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let pch = PCH.get(receiver, self);
        let pcl = PCL.get(receiver, self);
        let sp = SP.get(receiver, self);
        Address(sp.wrapping_sub(1)).set(receiver, self, pch);
        Address(sp.wrapping_sub(2)).set(receiver, self, pcl);
        SP.set(receiver, self, sp.wrapping_sub(2));
        PC.set(receiver, self, nn);
    }

    pub fn callcc<R>(&mut self, receiver: &mut R, cc: ConditionCode, nn: u16)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        if cc.get(receiver, self) {
            self.call(receiver, nn);
            self.cycles += 17;
        } else {
            self.cycles += 10;
        }
    }

    pub fn ret<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let sp = SP.get(receiver, self);
        let n1 = Address(sp).get(receiver, self);
        PCL.set(receiver, self, n1);
        let n2 = Address(sp.wrapping_add(1)).get(receiver, self);
        PCH.set(receiver, self, n2);
        SP.set(receiver, self, sp.wrapping_add(2));
    }

    pub fn retcc<R>(&mut self, receiver: &mut R, cc: ConditionCode)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        if cc.get(receiver, self) {
            self.ret(receiver);
            self.cycles += 11;
        } else {
            self.cycles += 5;
        }
    }

    pub fn reti<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        self.retn(receiver);
    }

    pub fn retn<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let iff2 = self.iff2;
        self.iff1 = if iff2 { 0 } else { 0xFFFFFFFFFFFFFFFF };

        let sp = SP.get(receiver, self);
        let pcl = Address(sp).get(receiver, self);
        let pch = Address(sp.wrapping_add(1)).get(receiver, self);
        PCL.set(receiver, self, pcl);
        PCH.set(receiver, self, pch);
        SP.set(receiver, self, sp.wrapping_add(2));
    }

    //// Input and Output Group
    ///////////////////////////

    pub fn in_n<T1, T2, R>(&mut self, receiver: &mut R, arg1: T1, arg2: T2)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
        T1: Settable<u8>,
        T2: Gettable<u8>,
    {
        let address_lo = arg2.get(receiver, self);
        let address_hi = arg1.get(receiver, self);
        let address = to16(address_lo, address_hi);
        self.address = address;
        let x = self.io.input(receiver, address);
        self.data = x;
        arg1.set(receiver, self, x);
    }

    fn in_impl<T1, R>(&mut self, receiver: &mut R, arg: T1) -> (u8, Flags)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
        T1: Gettable<u8>,
    {
        let address_lo = arg.get(receiver, self);
        let address_hi = B.get(receiver, self);
        let address = to16(address_lo, address_hi);
        self.address = address;
        let x = self.io.input(receiver, address);
        self.data = x;

        let mut f = self.flags();
        f.set_parity(x);
        f.set_sign(x);
        f.set_zero(x);
        f.remove(HF | NF);

        (x, f)
    }

    pub fn in_f<T1, R>(&mut self, receiver: &mut R, arg: T1)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
        T1: Gettable<u8>,
    {
        let (_, f) = self.in_impl(receiver, arg);
        F.set(receiver, self, f.bits());
    }

    pub fn in_c<T1, T2, R>(&mut self, receiver: &mut R, arg1: T1, arg2: T2)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
        T1: Settable<u8>,
        T2: Gettable<u8>,
    {
        let (x, f) = self.in_impl(receiver, arg2);
        self.data = x;
        arg1.set(receiver, self, x);
        F.set(receiver, self, f.bits());
    }

    /// The Z80 manual lists this instruction under IN r, (C) as "undefined"
    /// It sets
    pub fn in0<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let addr = BC.get(receiver, self);
        self.address = addr;
        let x = self.io.input(receiver, addr);
        self.data = x;

        let mut f = self.flags();
        f.set_parity(x);
        f.set_sign(x);
        f.set_zero(x);
        f.remove(HF | NF);
        F.set(receiver, self, f.bits());
    }

    fn inid_impl<R>(&mut self, receiver: &mut R, inc: u16) -> u8
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let b = B.get(receiver, self);
        let hl = HL.get(receiver, self);
        let addr = BC.get(receiver, self);
        self.address = addr;
        let x = self.io.input(receiver, addr);
        self.data = x;
        Address(hl).set(receiver, self, x);
        B.set(receiver, self, b.wrapping_sub(1));
        HL.set(receiver, self, hl.wrapping_add(inc));
        b.wrapping_sub(1)
    }

    pub fn ini<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let new_b = self.inid_impl(receiver, 1);

        let mut f = self.flags();
        f.set_zero(new_b);
        f.insert(NF);
        F.set(receiver, self, f.bits());
    }

    pub fn inir<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        while {
            self.cycles += 21;
            self.inid_impl(receiver, 1) != 0
        } {
            // r was already incremented twice by `run`
            inc_r(self);
            inc_r(self);
        }

        let mut f = self.flags();
        f.insert(ZF | NF);
        F.set(receiver, self, f.bits());

        self.cycles += 16;
    }

    pub fn ind<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let new_b = self.inid_impl(receiver, 0xFFFF);

        let mut f = self.flags();
        f.set_zero(new_b);
        f.insert(NF);
        F.set(receiver, self, f.bits());
    }

    pub fn indr<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        while {
            self.cycles += 21;
            self.inid_impl(receiver, 0xFFFF) != 0
        } {
            // r was already incremented twice by `run`
            inc_r(self);
            inc_r(self);
        }

        let mut f = self.flags();
        f.insert(ZF | NF);
        F.set(receiver, self, f.bits());

        self.cycles += 16;
    }

    pub fn out_n<T1, T2, R>(&mut self, receiver: &mut R, arg1: T1, arg2: T2)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
        T1: Gettable<u8>,
        T2: Gettable<u8>,
    {
        let address_lo = arg1.get(receiver, self);
        let address_hi = A.get(receiver, self);
        let address = to16(address_lo, address_hi);
        let x = arg2.get(receiver, self);
        self.address = address;
        self.data = x;
        self.io.output(receiver, address, x);
    }

    pub fn out_c<T1, T2, R>(&mut self, receiver: &mut R, arg1: T1, arg2: T2)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
        T1: Gettable<u8>,
        T2: Gettable<u8>,
    {
        let address_lo = arg1.get(receiver, self);
        let address_hi = B.get(receiver, self);
        let address = to16(address_lo, address_hi);
        let x = arg2.get(receiver, self);
        self.address = address;
        self.data = x;
        self.io.output(receiver, address, x);
    }

    fn outid_impl<R>(&mut self, receiver: &mut R, inc: u16)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        let b = B.get(receiver, self);
        let new_b = b.wrapping_sub(1);
        B.set(receiver, self, new_b);
        let addr = BC.get(receiver, self);
        self.address = addr;
        let hl = HL.get(receiver, self);
        let x = Address(hl).get(receiver, self);
        self.data = x;
        self.io.output(receiver, addr, x);
        HL.set(receiver, self, hl.wrapping_add(inc));
    }

    pub fn outi<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        self.outid_impl(receiver, 1);
        let new_b = B.get(receiver, self);

        let mut f = self.flags();
        f.set_zero(new_b);
        f.insert(NF);
        F.set(receiver, self, f.bits());
    }

    pub fn otir<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        while {
            self.cycles += 21;
            self.outid_impl(receiver, 1);
            B.get(receiver, self) != 0
        } {
            // r was already incremented twice by `run`
            inc_r(self);
            inc_r(self);
        }

        let mut f = self.flags();
        f.insert(ZF | NF);
        F.set(receiver, self, f.bits());

        self.cycles += 16;
    }

    pub fn outd<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        self.outid_impl(receiver, 0xFFFF);
        let new_b = B.get(receiver, self);

        let mut f = self.flags();
        f.set_zero(new_b);
        f.insert(NF);
        F.set(receiver, self, f.bits());
    }

    pub fn otdr<R>(&mut self, receiver: &mut R)
    where
        R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
        I: Io<R>,
    {
        while {
            self.cycles += 21;
            self.outid_impl(receiver, 0xFFFF);
            B.get(receiver, self) != 0
        } {
            // r was already incremented twice by `run`
            inc_r(self);
            inc_r(self);
        }

        let mut f = self.flags();
        f.insert(ZF | NF);
        F.set(receiver, self, f.bits());

        self.cycles += 16;
    }
}
