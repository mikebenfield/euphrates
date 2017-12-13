// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use std::convert::AsRef;

use memo::{Inbox, Outbox};
use super::*;
use utilities;

fn receive<Z>(z: &mut Z, memo: Memo)
where
    Z: Inbox<Memo> + AsRef<Component> + ?Sized,
{
    let id = z.as_ref().id();
    z.receive(id, memo);
}

/// Most of the functions in the rotate and shift group have similar addressing modes,
/// implementations, and flag behavior, so we write a macro to generate the
/// required functions in each case.
macro_rules! rotate_shift_functions_noa_impl {
    ($fn_impl: ident $fn_impl2: ident
    $fn_general: ident $fn_store: ident) => {
        fn $fn_impl2<Z, T1>(z: &mut Z, arg: T1) -> (u8, Flags)
        where
            Z: Machine + ?Sized,
            T1: Changeable<u8>,
        {
            let a = arg.view(z);
            let (result, mut f) = $fn_impl(z, a);
            arg.change(z, result);
            f.remove(Flags::HF | Flags::NF);
            (result, f)
        }

        pub fn $fn_general<Z, T1>(z: &mut Z, arg: T1)
        where
            Z: Machine + ?Sized,
            T1: Changeable<u8>,
        {
            let (result, mut f) = $fn_impl2(z, arg);
            f.set_parity(result);
            f.set_sign(result);
            f.set_zero(result);
            F.change(z, f.bits());
        }

        pub fn $fn_store<Z, T1>(z: &mut Z, arg: T1, store: Reg8)
        where
            Z: Machine + ?Sized,
            T1: Changeable<u8>,
        {
            let (result, mut f) = $fn_impl2(z, arg);
            f.set_parity(result);
            f.set_sign(result);
            f.set_zero(result);
            F.change(z, f.bits());
            store.change(z, result);
        }
    }
}

macro_rules! rotate_shift_functions_impl {
    ($fn_impl: ident $fn_impl2: ident $fn_general: ident
    $fn_store: ident $fn_a: ident) => {
        pub fn $fn_a<Z>(z: &mut Z)
        where
            Z: Machine + ?Sized
        {
            let (_, f) = $fn_impl2(z, A);
            F.change(z, f.bits());
        }
        rotate_shift_functions_noa_impl!{$fn_impl $fn_impl2 $fn_general $fn_store}
    }
}

pub fn rst<Z>(z: &mut Z, p: u16)
where
    Z: Machine + ?Sized,
{
    let sp = SP.view(z);
    let pch = PCH.view(z);
    let pcl = PCL.view(z);
    Address(sp.wrapping_sub(1)).change(z, pch);
    Address(sp.wrapping_sub(2)).change(z, pcl);
    SP.change(z, sp.wrapping_sub(2));
    PC.change(z, p);
}

pub fn nonmaskable_interrupt<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    // The Z80 manual implies that IFF2 is set to IFF1, but this
    // is false (see Young 5.3)
    receive(z, Memo::NonmaskableInterrupt);
    z.as_mut().inc_r();
    z.as_mut().iff1 = 0xFFFFFFFFFFFFFFFF;
    z.clear_nmi();
    z.as_mut().cycles += 11;
    if z.as_ref().halted {
        let pc = PC.view(z);
        PC.change(z, pc.wrapping_add(1));
    }
    rst(z, 0x66);
}

pub fn maskable_interrupt<Z>(z: &mut Z, x: u8) -> bool
where
    Z: Machine + ?Sized,
{
    if z.as_ref().iff1 < z.as_ref().cycles {
        receive(z, Memo::MaskableInterruptAllowed);

        z.as_mut().inc_r();

        z.as_mut().iff1 = 0xFFFFFFFFFFFFFFFF;
        z.as_mut().iff2 = false;

        if z.as_ref().halted {
            let pc = PC.view(z);
            PC.change(z, pc.wrapping_add(1));
        }

        let im = z.as_ref().interrupt_mode;
        match im {
            Im1 => {
                rst(z, 0x38);
                z.as_mut().cycles += 13;
            }
            Im2 => {
                let i = I.view(z);
                let new_pc = utilities::to16(x, i);
                rst(z, new_pc);
                z.as_mut().cycles += 19;
            }
            _ => unimplemented!(),
        }
        true
    } else {
        receive(z, Memo::MaskableInterruptDenied);
        false
    }
}

//// 8-Bit Load Group
/////////////////////

pub fn ld<Z, T1, T2>(z: &mut Z, arg1: T1, arg2: T2)
where
    Z: Machine + ?Sized,
    T1: Changeable<u8>,
    T2: Viewable<u8>,
{
    let val = arg2.view(z);
    arg1.change(z, val);
}

// XXX text about interrupts in manual
pub fn ld_ir<Z>(z: &mut Z, arg1: Reg8, arg2: Reg8)
where
    Z: Machine + ?Sized,
{
    let val = arg2.view(z);
    arg1.change(z, val);
    let iff2 = z.as_ref().iff2;
    let mut f = z.as_ref().flags();
    f.set_sign(val);
    f.set_zero(val);
    f.remove(Flags::NF | Flags::HF);
    f.set(Flags::PF, iff2);
    z.as_mut().set_flags(f);
}

//// 16-Bit Load Group
//////////////////////

pub fn ld16<Z, T1, T2>(z: &mut Z, arg1: T1, arg2: T2)
where
    Z: Machine + ?Sized,
    T1: Changeable<u16>,
    T2: Viewable<u16>,
{
    let val = arg2.view(z);
    arg1.change(z, val);
}

pub fn push<Z>(z: &mut Z, reg: Reg16)
where
    Z: Machine + ?Sized,
{
    let (lo, hi) = utilities::to8(reg.view(z));
    let sp = SP.view(z);
    Address(sp.wrapping_sub(1)).change(z, hi);
    Address(sp.wrapping_sub(2)).change(z, lo);
    SP.change(z, sp.wrapping_sub(2));
}

pub fn pop<Z>(z: &mut Z, reg: Reg16)
where
    Z: Machine + ?Sized,
{
    let sp = SP.view(z);
    let lo = Address(sp).view(z);
    let hi = Address(sp.wrapping_add(1)).view(z);
    reg.change(z, utilities::to16(lo, hi));
    SP.change(z, sp.wrapping_add(2));
}

//// Exchange, Block Transfer, and Search Group
///////////////////////////////////////////////

pub fn ex<Z, T1>(z: &mut Z, reg1: T1, reg2: Reg16)
where
    Z: Machine + ?Sized,
    T1: Changeable<u16>,
{
    let val1 = reg1.view(z);
    let val2 = reg2.view(z);
    reg1.change(z, val2);
    reg2.change(z, val1);
}

pub fn exx<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    for &(reg1, reg2) in [(BC, BC0), (DE, DE0), (HL, HL0)].iter() {
        let val1 = reg1.view(z);
        let val2 = reg2.view(z);
        reg1.change(z, val2);
        reg2.change(z, val1);
    }
}

pub fn ldid<Z>(z: &mut Z, inc: u16)
where
    Z: Machine + ?Sized,
{
    let hl = HL.view(z);
    let de = DE.view(z);
    let bc = BC.view(z);

    let phl = Viewable::<u8>::view(Address(hl), z);
    Address(de).change(z, phl);

    HL.change(z, hl.wrapping_add(inc));
    DE.change(z, de.wrapping_add(inc));
    BC.change(z, bc.wrapping_sub(1));

    let mut f = z.as_ref().flags();
    f.remove(Flags::HF | Flags::NF);
    f.set(Flags::PF, bc != 1);
    F.change(z, f.bits());
}

pub fn ldi<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    ldid(z, 1);
}

pub fn ldd<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    ldid(z, 0xFFFF);
}

pub fn ldir<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    loop {
        ldi(z);
        if BC.view(z) == 0 {
            z.as_mut().cycles += 17;
            return;
        }
        z.as_mut().cycles += 21;

        // check the possibility that we have overwritten our own opcode
        let pc = PC.view(z);
        let apc1 = Viewable::<u8>::view(Address(pc.wrapping_sub(2)), z);
        let apc2 = Viewable::<u8>::view(Address(pc.wrapping_sub(1)), z);
        if apc1 != 0xED || apc2 != 0xB0 {
            PC.change(z, pc.wrapping_sub(2));
            return;
        }
        z.as_mut().inc_r();
        z.as_mut().inc_r();
    }
}

pub fn lddr<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    loop {
        ldd(z);
        if BC.view(z) == 0 {
            z.as_mut().cycles += 17;
            return;
        }
        z.as_mut().cycles += 21;

        // check the possibility that we have overwritten our own opcode
        let pc = PC.view(z);
        let apc1 = Viewable::<u8>::view(Address(pc.wrapping_sub(2)), z);
        let apc2 = Viewable::<u8>::view(Address(pc.wrapping_sub(1)), z);
        if apc1 != 0xED || apc2 != 0xB8 {
            PC.change(z, pc.wrapping_sub(1));
            return;
        }
        z.as_mut().inc_r();
        z.as_mut().inc_r();
    }
}

pub fn cpid<Z>(z: &mut Z, inc: u16)
where
    Z: Machine + ?Sized,
{
    let bc = BC.view(z);
    let a = A.view(z);
    let hl = HL.view(z);

    let phl: u8 = Address(HL).view(z);
    let result = a.wrapping_sub(phl);

    HL.change(z, hl.wrapping_add(inc));
    BC.change(z, bc.wrapping_sub(1));

    let mut f = z.as_ref().flags();
    f.set_sign(result);
    f.set_zero(result);
    f.set(Flags::HF, phl & 0xF > a & 0xF);
    f.set(Flags::PF, bc != 1);
    f.insert(Flags::NF);
    F.change(z, f.bits());
}

pub fn cpi<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    cpid(z, 1);
}

pub fn cpir<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    while {
        cpi(z);
        z.as_mut().cycles += 21;
        BC.view(z) != 0 && !z.as_ref().flags().contains(Flags::ZF)
    }
    {
        // r was already incremented twice by `run`
        z.as_mut().inc_r();
        z.as_mut().inc_r();
    }

    z.as_mut().cycles += 17;
}

pub fn cpd<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    cpid(z, 0xFFFF);
}

pub fn cpdr<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    while {
        cpd(z);
        z.as_mut().cycles += 21;
        BC.view(z) != 0 && !z.as_ref().flags().contains(Flags::ZF)
    }
    {
        // r was already incremented twice by `run`
        z.as_mut().inc_r();
        z.as_mut().inc_r();
    }

    z.as_mut().cycles += 17;
}

//// 8-Bit Arithmetic Group
///////////////////////////

fn add_impl<Z>(z: &mut Z, a: u8, x: u8, cf: u8) -> (u8, Flags)
where
    Z: Machine + ?Sized,
{
    // XXX optimize?
    let result16 = (x as u16).wrapping_add(a as u16).wrapping_add(cf as u16);
    let result8 = result16 as u8;

    let mut f = z.as_ref().flags();
    f.set_zero(result8);
    f.set_sign(result8);

    f.set(Flags::CF, result16 & (1 << 8) != 0);

    // carry from bit 3 happened if:
    // x and a have same bit 4 AND result is set OR
    // x and a have different bit 4 AND result is clear
    let hf = (x ^ a ^ result8) & (1 << 4) != 0;
    f.set(Flags::HF, hf);

    // overflow happened if:
    // x and a both have bit 7 AND result does not OR
    // x and a have clear bit 7 AND result is set
    // in other words, x and y have the same bit 7 and
    // result is different
    let overflow = !(x ^ a) & (x ^ result8) & (1 << 7) != 0;
    f.set(Flags::PF, overflow);

    f.remove(Flags::NF);

    (result8, f)
}

pub fn add<Z, T1, T2>(z: &mut Z, arg1: T1, arg2: T2)
where
    Z: Machine + ?Sized,
    T1: Changeable<u8>,
    T2: Viewable<u8>,
{
    let a = arg1.view(z);
    let b = arg2.view(z);
    let (result, f) = add_impl(z, a, b, 0);
    arg1.change(z, result);
    F.change(z, f.bits());
}

pub fn adc<Z, T1, T2>(z: &mut Z, arg1: T1, arg2: T2)
where
    Z: Machine + ?Sized,
    T1: Changeable<u8>,
    T2: Viewable<u8>,
{
    let cf = if z.as_ref().flags().contains(Flags::CF) {
        1u8
    } else {
        0u8
    };
    let a = arg1.view(z);
    let x = arg2.view(z);
    let (result, f) = add_impl(z, a, x, cf);
    arg1.change(z, result);
    F.change(z, f.bits());
}

fn sub_impl<Z>(z: &mut Z, a: u8, x: u8, cf: u8) -> (u8, Flags)
where
    Z: Machine + ?Sized,
{
    let (result, mut f) = add_impl(z, a, !x, 1 ^ cf);
    f.toggle(Flags::CF | Flags::HF);
    f.insert(Flags::NF);
    (result, f)
}

pub fn sub<Z, T1, T2>(z: &mut Z, arg1: T1, arg2: T2)
where
    Z: Machine + ?Sized,
    T1: Changeable<u8>,
    T2: Viewable<u8>,
{
    let a = arg1.view(z);
    let x = arg2.view(z);
    let (result, f) = sub_impl(z, a, x, 0);
    arg1.change(z, result);
    F.change(z, f.bits());
}

pub fn sbc<Z, T1, T2>(z: &mut Z, arg1: T1, arg2: T2)
where
    Z: Machine + ?Sized,
    T1: Changeable<u8>,
    T2: Viewable<u8>,
{
    let cf = if z.as_ref().flags().contains(Flags::CF) {
        1u8
    } else {
        0u8
    };
    let a = arg1.view(z);
    let x = arg2.view(z);
    let (result, f) = sub_impl(z, a, x, cf);
    arg1.change(z, result);
    F.change(z, f.bits());
}

fn andor_impl<Z>(z: &mut Z, result: u8) -> Flags
where
    Z: Machine + ?Sized,
{
    A.change(z, result);

    // note that for AND and OR, the manual says Flags::PF is set according to whether
    // there is overflow. I'm betting that is a mistake.
    let mut f = z.as_ref().flags();
    f.set_parity(result);
    f.set_sign(result);
    f.set_zero(result);
    f.remove(Flags::HF | Flags::NF | Flags::CF);
    f
}

pub fn and<Z, T1>(z: &mut Z, arg: T1)
where
    Z: Machine + ?Sized,
    T1: Viewable<u8>,
{
    let result = arg.view(z) & A.view(z);
    let mut f = andor_impl(z, result);
    f.insert(Flags::HF);
    F.change(z, f.bits());
}

pub fn or<Z, T1>(z: &mut Z, arg: T1)
where
    Z: Machine + ?Sized,
    T1: Viewable<u8>,
{
    let result = arg.view(z) | A.view(z);
    let f = andor_impl(z, result);
    F.change(z, f.bits());
}

pub fn xor<Z, T1>(z: &mut Z, arg: T1)
where
    Z: Machine + ?Sized,
    T1: Viewable<u8>,
{
    let result = arg.view(z) ^ A.view(z);
    let f = andor_impl(z, result);
    F.change(z, f.bits());
}

pub fn cp<Z, T1>(z: &mut Z, arg: T1)
where
    Z: Machine + ?Sized,
    T1: Viewable<u8>,
{
    let x = arg.view(z);
    let a = A.view(z);
    let (_, f) = sub_impl(z, a, x, 0);
    A.change(z, a);
    F.change(z, f.bits());
}

pub fn inc<Z, T1>(z: &mut Z, arg: T1)
where
    Z: Machine + ?Sized,
    T1: Changeable<u8>,
{
    let x = arg.view(z);
    let result = x.wrapping_add(1);
    arg.change(z, result);
    let mut f = z.as_ref().flags();
    f.set_zero(result);
    f.set_sign(result);
    f.set(Flags::HF, x & 0xF == 0xF);
    f.set(Flags::PF, x == 0x7F);
    f.remove(Flags::NF);
    F.change(z, f.bits());
}

pub fn dec<Z, T1>(z: &mut Z, arg: T1)
where
    Z: Machine + ?Sized,
    T1: Changeable<u8>,
{
    let x = arg.view(z);
    let result = x.wrapping_sub(1);
    arg.change(z, result);
    let mut f = z.as_ref().flags();
    f.set_zero(result);
    f.set_sign(result);
    f.set(Flags::HF, x & 0xF == 0);
    f.set(Flags::PF, x == 0x80);
    f.insert(Flags::NF);
    F.change(z, f.bits());
}

//// General-Purpose Arithmetic and CPU Control Groups
//////////////////////////////////////////////////////

pub fn daa<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    // see the table in Young
    let a = A.view(z);
    let cf = z.as_ref().flags().contains(Flags::CF);
    let hf = z.as_ref().flags().contains(Flags::HF);
    let nf = z.as_ref().flags().contains(Flags::NF);
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
    A.change(z, new_a);

    let mut f = z.as_ref().flags();
    f.set_parity(new_a);
    f.set_zero(new_a);
    f.set_sign(new_a);
    f.set(Flags::CF, new_cf != 0);
    f.set(Flags::HF, new_hf != 0);
    F.change(z, f.bits());
}

pub fn cpl<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    let a = A.view(z);
    A.change(z, !a);
    let mut f = z.as_ref().flags();
    f.insert(Flags::HF | Flags::NF);
    F.change(z, f.bits());
}

pub fn neg<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    // subtracts A from 0
    let a = A.view(z);
    let (result, mut f) = sub_impl(z, 0, a, 0);
    A.change(z, result);
    f.set(Flags::PF, a == 0x80);
    f.set(Flags::CF, a != 0);
    F.change(z, f.bits());
}

pub fn ccf<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    let mut f = z.as_ref().flags();
    let cf = f.contains(Flags::CF);
    f.set(Flags::HF, cf);
    f.toggle(Flags::CF);
    f.remove(Flags::NF);
    F.change(z, f.bits());
}

pub fn scf<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    let mut f = z.as_ref().flags();
    f.remove(Flags::HF | Flags::NF);
    f.insert(Flags::CF);
    F.change(z, f.bits());
}

pub fn nop<Z>(_z: &mut Z)
where
    Z: Machine + ?Sized,
{
}

pub fn halt<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    z.as_mut().halted = true;
}

pub fn di<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    z.as_mut().iff1 = 0xFFFFFFFFFFFFFFFF;
    z.as_mut().iff2 = false;
}

pub fn ei<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    z.as_mut().iff1 = z.as_mut().cycles + 4;
    z.as_mut().iff2 = true;
}

pub fn im<Z>(z: &mut Z, m: u8)
where
    Z: Machine + ?Sized,
{
    match m {
        0 => z.as_mut().interrupt_mode = Im0,
        1 => z.as_mut().interrupt_mode = Im1,
        2 => z.as_mut().interrupt_mode = Im2,
        _ => panic!("Z80: Invalid interrupt mode"),
    }
}

pub fn im1<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    z.as_mut().interrupt_mode = Im1;
}

pub fn im2<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    z.as_mut().interrupt_mode = Im2;
}

//// 16-Bit Arithmetic Group
////////////////////////////

fn add16_impl<Z>(z: &mut Z, x: u16, y: u16, cf: u16) -> (u16, Flags)
where
    Z: Machine + ?Sized,
{
    // XXX optimize?
    let result32 = (x as u32).wrapping_add(y as u32).wrapping_add(cf as u32);
    let result16 = result32 as u16;

    let mut f = z.as_ref().flags();
    f.set(Flags::CF, result32 & (1 << 16) != 0);

    // carry from bit 11 happened if:
    // x and y have same bit 12 AND result is set OR
    // x and y have different bit 12 AND result is clear
    let hf = (x ^ y ^ result16) & (1 << 12) != 0;
    f.set(Flags::HF, hf);

    f.remove(Flags::NF);

    (result16, f)
}

pub fn add16<Z>(z: &mut Z, arg1: Reg16, arg2: Reg16)
where
    Z: Machine + ?Sized,
{
    let x = arg1.view(z);
    let y = arg2.view(z);
    let (result, f) = add16_impl(z, x, y, 0);
    arg1.change(z, result);
    F.change(z, f.bits());
}

fn adc16_impl<Z>(z: &mut Z, x: u16, y: u16, cf: u16) -> (u16, Flags)
where
    Z: Machine + ?Sized,
{
    let (result, mut f) = add16_impl(z, x, y, cf as u16);

    f.set_sign((result >> 8) as u8);
    f.set_zero((result as u8) | (result >> 8) as u8);

    // overflow happened if:
    // x and y both have bit 15 AND result does not OR
    // x and y have clear bit 15 AND result is set
    // in other words, x and y have the same bit 15, which is different from bit
    // 15 of result
    let overflow = !(x ^ y) & (x ^ result) & (1 << 15) != 0;
    f.set(Flags::PF, overflow);

    (result, f)
}

pub fn adc16<Z>(z: &mut Z, arg1: Reg16, arg2: Reg16)
where
    Z: Machine + ?Sized,
{
    let x = arg1.view(z);
    let y = arg2.view(z);
    let cf = if z.as_ref().flags().contains(Flags::CF) {
        1u8
    } else {
        0u8
    };
    let (result, f) = adc16_impl(z, x, y, cf as u16);
    arg1.change(z, result);
    F.change(z, f.bits());
}

pub fn sbc16<Z>(z: &mut Z, arg1: Reg16, arg2: Reg16)
where
    Z: Machine + ?Sized,
{
    let x = arg1.view(z);
    let y = arg2.view(z);
    let cf = if z.as_ref().flags().contains(Flags::CF) {
        1u8
    } else {
        0u8
    };
    let (result, mut f) = adc16_impl(z, x, !y, (1 ^ cf) as u16);
    arg1.change(z, result);
    f.toggle(Flags::CF | Flags::HF);
    f.insert(Flags::NF);
    F.change(z, f.bits());
}

pub fn inc16<Z>(z: &mut Z, arg: Reg16)
where
    Z: Machine + ?Sized,
{
    let val = arg.view(z);
    arg.change(z, val.wrapping_add(1));
}

pub fn dec16<Z>(z: &mut Z, arg: Reg16)
where
    Z: Machine + ?Sized,
{
    let val = arg.view(z);
    arg.change(z, val.wrapping_sub(1));
}

//// Rotate and Shift Group
///////////////////////////

fn rlc_impl<Z>(z: &mut Z, x: u8) -> (u8, Flags)
where
    Z: Machine + ?Sized,
{
    let mut f = z.as_ref().flags();
    f.set(Flags::CF, x & 0x80 != 0);
    (x.rotate_left(1), f)
}

rotate_shift_functions_impl!{
    rlc_impl rlc_impl2 rlc rlc_store rlca
}

fn rl_impl<Z>(z: &mut Z, x: u8) -> (u8, Flags)
where
    Z: Machine + ?Sized,
{
    let mut f = z.as_ref().flags();
    let mut result = x << 1;
    if f.contains(Flags::CF) {
        result |= 1;
    } else {
        result &= !1;
    }
    f.set(Flags::CF, x & 0x80 != 0);
    (result, f)
}

rotate_shift_functions_impl!{
    rl_impl rl_impl2 rl rl_store rla
}

fn rrc_impl<Z>(z: &mut Z, x: u8) -> (u8, Flags)
where
    Z: Machine + ?Sized,
{
    let mut f = z.as_ref().flags();
    f.set(Flags::CF, x & 1 != 0);
    (x.rotate_right(1), f)
}

rotate_shift_functions_impl!{
    rrc_impl rrc_impl2 rrc rrc_store rrca
}

fn rr_impl<Z>(z: &mut Z, x: u8) -> (u8, Flags)
where
    Z: Machine + ?Sized,
{
    let mut f = z.as_ref().flags();
    let mut result = x >> 1;
    if f.contains(Flags::CF) {
        result |= 0x80;
    } else {
        result &= !0x80;
    }
    f.set(Flags::CF, x & 1 != 0);
    (result, f)
}

rotate_shift_functions_impl!{
    rr_impl rr_impl2 rr rr_store rra
}

fn sla_impl<Z>(z: &mut Z, x: u8) -> (u8, Flags)
where
    Z: Machine + ?Sized,
{
    let mut f = z.as_ref().flags();
    f.set(Flags::CF, x & 0x80 != 0);
    (x << 1, f)
}

rotate_shift_functions_noa_impl!{
    sla_impl sla_impl2 sla sla_store
}

// SLL is undocumented; see Young
fn sll_impl<Z>(z: &mut Z, x: u8) -> (u8, Flags)
where
    Z: Machine + ?Sized,
{
    let mut f = z.as_ref().flags();
    f.set(Flags::CF, x & 0x80 != 0);
    let mut result = x << 1;
    result |= 1;
    (result, f)
}

rotate_shift_functions_noa_impl!{
    sll_impl sll_impl2 sll sll_store
}

fn sra_impl<Z>(z: &mut Z, x: u8) -> (u8, Flags)
where
    Z: Machine + ?Sized,
{
    let mut f = z.as_ref().flags();
    f.set(Flags::CF, x & 1 != 0);
    let result = ((x as i8) >> 1) as u8;
    (result, f)
}

rotate_shift_functions_noa_impl!{
    sra_impl sra_impl2 sra sra_store
}

fn srl_impl<Z>(z: &mut Z, x: u8) -> (u8, Flags)
where
    Z: Machine + ?Sized,
{
    let mut f = z.as_ref().flags();
    f.set(Flags::CF, x & 1 != 0);
    (x >> 1, f)
}

rotate_shift_functions_noa_impl!{
    srl_impl srl_impl2 srl srl_store
}

pub fn rld<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    let hl: u8 = Address(HL).view(z);
    let hl_lo: u8 = 0xF & hl;
    let hl_hi: u8 = 0xF0 & hl;
    let a_lo = 0xF & A.view(z);
    let a_hi = 0xF0 & A.view(z);
    Address(HL).change(z, hl_lo << 4 | a_lo);
    A.change(z, hl_hi >> 4 | a_hi);
    let a = A.view(z);

    let mut f = z.as_ref().flags();
    f.set_parity(a);
    f.set_sign(a);
    f.set_zero(a);
    f.remove(Flags::HF | Flags::NF);
    F.change(z, f.bits());
}

pub fn rrd<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    let hl: u8 = Address(HL).view(z);
    let hl_lo: u8 = 0xF & hl;
    let hl_hi: u8 = 0xF0 & hl;
    let a_lo = 0xF & A.view(z);
    let a_hi = 0xF0 & A.view(z);
    Address(HL).change(z, a_lo << 4 | hl_hi >> 4);
    A.change(z, hl_lo | a_hi);
    let a = A.view(z);

    let mut f = z.as_ref().flags();
    f.set_parity(a);
    f.set_sign(a);
    f.set_zero(a);
    f.remove(Flags::HF | Flags::NF);
    F.change(z, f.bits());
}

//// Bit set, reset, and Test Group
///////////////////////////////////

pub fn bit<Z, T>(z: &mut Z, b: u8, arg: T)
where
    Z: Machine + ?Sized,
    T: Viewable<u8>,
{
    let x = arg.view(z);
    let bitflag = 1 << b;
    let x_contains = x & bitflag != 0;

    let mut f = z.as_ref().flags();
    f.set(Flags::ZF | Flags::PF, !x_contains);
    f.insert(Flags::HF);
    f.remove(Flags::NF);
    f.set(Flags::SF, b == 7 && x_contains);
    F.change(z, f.bits());
}

pub fn set<Z, T>(z: &mut Z, b: u8, arg: T)
where
    Z: Machine + ?Sized,
    T: Changeable<u8>,
{
    let mut x = arg.view(z);
    utilities::set_bit(&mut x, b);
    arg.change(z, x);
}

pub fn set_store<Z, T>(z: &mut Z, b: u8, arg: T, reg: Reg8)
where
    Z: Machine + ?Sized,
    T: Changeable<u8>,
{
    arg.change(z, b);
    let x = arg.view(z);
    reg.change(z, x);
}

pub fn res<Z, T>(z: &mut Z, b: u8, arg: T)
where
    Z: Machine + ?Sized,
    T: Changeable<u8>,
{
    let mut x = arg.view(z);
    utilities::clear_bit(&mut x, b);
    arg.change(z, x);
}

pub fn res_store<Z, T>(z: &mut Z, b: u8, arg: T, reg: Reg8)
where
    Z: Machine + ?Sized,
    T: Changeable<u8>,
{
    res(z, b, arg);
    let x = arg.view(z);
    reg.change(z, x);
}

//// Jump Group
///////////////

pub fn jp<Z, T>(z: &mut Z, arg: T)
where
    Z: Machine + ?Sized,
    T: Viewable<u16>,
{
    let addr = arg.view(z);
    PC.change(z, addr);
}

pub fn jpcc<Z>(z: &mut Z, cc: ConditionCode, arg: u16)
where
    Z: Machine + ?Sized,
{
    if cc.view(z) {
        jp(z, arg);
    }
}

pub fn jr<Z>(z: &mut Z, e: i8)
where
    Z: Machine + ?Sized,
{
    let pc = PC.view(z);
    let new_pc = pc.wrapping_add(e as i16 as u16);
    PC.change(z, new_pc);
}

pub fn jrcc<Z>(z: &mut Z, cc: ConditionCode, e: i8)
where
    Z: Machine + ?Sized,
{
    if cc.view(z) {
        jr(z, e);
        z.as_mut().cycles += 12;
    } else {
        z.as_mut().cycles += 7;
    }
}

pub fn djnz<Z>(z: &mut Z, e: i8)
where
    Z: Machine + ?Sized,
{
    let b = B.view(z);
    let new_b = b.wrapping_sub(1);
    B.change(z, new_b);
    if new_b != 0 {
        jr(z, e);
        z.as_mut().cycles += 13;
    } else {
        z.as_mut().cycles += 8;
    }
}

//// Call and Return Group
//////////////////////////

pub fn call<Z>(z: &mut Z, nn: u16)
where
    Z: Machine + ?Sized,
{
    let pch = PCH.view(z);
    let pcl = PCL.view(z);
    let sp = SP.view(z);
    Address(sp.wrapping_sub(1)).change(z, pch);
    Address(sp.wrapping_sub(2)).change(z, pcl);
    SP.change(z, sp.wrapping_sub(2));
    PC.change(z, nn);
}

pub fn callcc<Z>(z: &mut Z, cc: ConditionCode, nn: u16)
where
    Z: Machine + ?Sized,
{
    if cc.view(z) {
        call(z, nn);
        z.as_mut().cycles += 17;
    } else {
        z.as_mut().cycles += 10;
    }
}

pub fn ret<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    let sp = SP.view(z);
    let n1 = Address(sp).view(z);
    PCL.change(z, n1);
    let n2 = Address(sp.wrapping_add(1)).view(z);
    PCH.change(z, n2);
    SP.change(z, sp.wrapping_add(2));
}

pub fn retcc<Z>(z: &mut Z, cc: ConditionCode)
where
    Z: Machine + ?Sized,
{
    if cc.view(z) {
        ret(z);
        z.as_mut().cycles += 11;
    } else {
        z.as_mut().cycles += 5;
    }
}

pub fn reti<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    retn(z);
}

pub fn retn<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    let iff2 = z.as_ref().iff2;
    z.as_mut().iff1 = if iff2 { 0 } else { 0xFFFFFFFFFFFFFFFF };

    let sp = SP.view(z);
    let pcl = Address(sp).view(z);
    let pch = Address(sp.wrapping_add(1)).view(z);
    PCL.change(z, pcl);
    PCH.change(z, pch);
    SP.change(z, sp.wrapping_add(2));
}

//// Input and Output Group
///////////////////////////

pub fn in_n<Z, T1, T2>(z: &mut Z, arg1: T1, arg2: T2)
where
    Z: Machine + ?Sized,
    T1: Changeable<u8>,
    T2: Viewable<u8>,
{
    let address_lo = arg2.view(z);
    let address_hi = arg1.view(z);
    let address = utilities::to16(address_lo, address_hi);
    z.as_mut().address = address;
    let x = z.input(address);
    z.as_mut().data = x;
    arg1.change(z, x);
}

fn in_impl<Z, T1>(z: &mut Z, arg: T1) -> (u8, Flags)
where
    Z: Machine + ?Sized,
    T1: Viewable<u8>,
{
    let address_lo = arg.view(z);
    let address_hi = B.view(z);
    let address = utilities::to16(address_lo, address_hi);
    z.as_mut().address = address;
    let x = z.input(address);
    z.as_mut().data = x;

    let mut f = z.as_ref().flags();
    f.set_parity(x);
    f.set_sign(x);
    f.set_zero(x);
    f.remove(Flags::HF | Flags::NF);

    (x, f)
}

pub fn in_f<Z, T1>(z: &mut Z, arg: T1)
where
    Z: Machine + ?Sized,
    T1: Viewable<u8>,
{
    let (_, f) = in_impl(z, arg);
    F.change(z, f.bits());
}

pub fn in_c<Z, T1, T2>(z: &mut Z, arg1: T1, arg2: T2)
where
    Z: Machine + ?Sized,
    T1: Changeable<u8>,
    T2: Viewable<u8>,
{
    let (x, f) = in_impl(z, arg2);
    z.as_mut().data = x;
    arg1.change(z, x);
    F.change(z, f.bits());
}

/// The Z80 manual lists this instruction under IN r, (C) as "undefined"
/// It sets
pub fn in0<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    let addr = BC.view(z);
    z.as_mut().address = addr;
    let x = z.input(addr);
    z.as_mut().data = x;

    let mut f = z.as_ref().flags();
    f.set_parity(x);
    f.set_sign(x);
    f.set_zero(x);
    f.remove(Flags::HF | Flags::NF);
    F.change(z, f.bits());
}

fn inid_impl<Z>(z: &mut Z, inc: u16) -> u8
where
    Z: Machine + ?Sized,
{
    let b = B.view(z);
    let hl = HL.view(z);
    let addr = BC.view(z);
    z.as_mut().address = addr;
    let x = z.input(addr);
    z.as_mut().data = x;
    Address(hl).change(z, x);
    B.change(z, b.wrapping_sub(1));
    HL.change(z, hl.wrapping_add(inc));
    b.wrapping_sub(1)
}

pub fn ini<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    let new_b = inid_impl(z, 1);

    let mut f = z.as_ref().flags();
    f.set_zero(new_b);
    f.insert(Flags::NF);
    F.change(z, f.bits());
}

pub fn inir<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    while {
        z.as_mut().cycles += 21;
        inid_impl(z, 1) != 0
    }
    {
        // r was already incremented twice by `run`
        z.as_mut().inc_r();
        z.as_mut().inc_r();
    }

    let mut f = z.as_ref().flags();
    f.insert(Flags::ZF | Flags::NF);
    F.change(z, f.bits());

    z.as_mut().cycles += 16;
}

pub fn ind<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    let new_b = inid_impl(z, 0xFFFF);

    let mut f = z.as_ref().flags();
    f.set_zero(new_b);
    f.insert(Flags::NF);
    F.change(z, f.bits());
}

pub fn indr<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    while {
        z.as_mut().cycles += 21;
        inid_impl(z, 0xFFFF) != 0
    }
    {
        // r was already incremented twice by `run`
        z.as_mut().inc_r();
        z.as_mut().inc_r();
    }

    let mut f = z.as_ref().flags();
    f.insert(Flags::ZF | Flags::NF);
    F.change(z, f.bits());

    z.as_mut().cycles += 16;
}

pub fn out_n<Z, T1, T2>(z: &mut Z, arg1: T1, arg2: T2)
where
    Z: Machine + ?Sized,
    T1: Viewable<u8>,
    T2: Viewable<u8>,
{
    let address_lo = arg1.view(z);
    let address_hi = A.view(z);
    let address = utilities::to16(address_lo, address_hi);
    let x = arg2.view(z);
    z.as_mut().address = address;
    z.as_mut().data = x;
    z.output(address, x);
}

pub fn out_c<Z, T1, T2>(z: &mut Z, arg1: T1, arg2: T2)
where
    Z: Machine + ?Sized,
    T1: Viewable<u8>,
    T2: Viewable<u8>,
{
    let address_lo = arg1.view(z);
    let address_hi = B.view(z);
    let address = utilities::to16(address_lo, address_hi);
    let x = arg2.view(z);
    z.as_mut().address = address;
    z.as_mut().data = x;
    z.output(address, x);
}

fn outid_impl<Z>(z: &mut Z, inc: u16)
where
    Z: Machine + ?Sized,
{
    let b = B.view(z);
    let new_b = b.wrapping_sub(1);
    B.change(z, new_b);
    let addr = BC.view(z);
    z.as_mut().address = addr;
    let hl = HL.view(z);
    let x = Address(hl).view(z);
    z.as_mut().data = x;
    z.output(addr, x);
    HL.change(z, hl.wrapping_add(inc));
}

pub fn outi<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    outid_impl(z, 1);
    let new_b = B.view(z);

    let mut f = z.as_ref().flags();
    f.set_zero(new_b);
    f.insert(Flags::NF);
    F.change(z, f.bits());
}

pub fn otir<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    while {
        z.as_mut().cycles += 21;
        outid_impl(z, 1);
        B.view(z) != 0
    }
    {
        // r was already incremented twice by `run`
        z.as_mut().inc_r();
        z.as_mut().inc_r();
    }

    let mut f = z.as_ref().flags();
    f.insert(Flags::ZF | Flags::NF);
    F.change(z, f.bits());

    z.as_mut().cycles += 16;
}

pub fn outd<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    outid_impl(z, 0xFFFF);
    let new_b = B.view(z);

    let mut f = z.as_ref().flags();
    f.set_zero(new_b);
    f.insert(Flags::NF);
    F.change(z, f.bits());
}

pub fn otdr<Z>(z: &mut Z)
where
    Z: Machine + ?Sized,
{
    while {
        z.as_mut().cycles += 21;
        outid_impl(z, 0xFFFF);
        B.view(z) != 0
    }
    {
        // r was already incremented twice by `run`
        z.as_mut().inc_r();
        z.as_mut().inc_r();
    }

    let mut f = z.as_ref().flags();
    f.insert(Flags::ZF | Flags::NF);
    F.change(z, f.bits());

    z.as_mut().cycles += 16;
}
