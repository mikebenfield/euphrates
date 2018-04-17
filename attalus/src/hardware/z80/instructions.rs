use memo::{Inbox, Payload};
use hardware::io_16_8::Io16_8;
use hardware::memory_16_8::Memory16;

use super::*;
use super::InterruptMode::*;
use super::Reg16::*;
use super::Reg8::*;
use super::ConditionCode;
use super::memo::manifests;
use utilities;

/// Most of the functions in the rotate and shift group have similar addressing modes,
/// implementations, and flag behavior, so we write a macro to generate the
/// required functions in each case.
macro_rules! rotate_shift_functions_noa_impl {
    ($fn_impl: ident $fn_impl2: ident
    $fn_general: ident $fn_store: ident) => {
        fn $fn_impl2<Z, T1>(z: &mut Z, arg: T1) -> u8
        where
            Z: Z80Internal + ?Sized,
            T1: Changeable<u8, Z>,
        {
            let a = arg.view(z);
            let result = $fn_impl(z, a);
            arg.change(z, result);
            z.clear_flag(HF | NF);
            result
        }

        pub fn $fn_general<Z, T1>(z: &mut Z, arg: T1)
        where
            Z: Z80Internal + ?Sized,
            T1: Changeable<u8, Z>,
        {
            let result = $fn_impl2(z, arg);
            z.set_parity(result);
            z.set_sign(result);
            z.set_zero(result);
        }

        pub fn $fn_store<Z, T1>(z: &mut Z, arg: T1, store: Reg8)
        where
            Z: Z80Internal + ?Sized,
            T1: Changeable<u8, Z>,
        {
            let result = $fn_impl2(z, arg);
            z.set_parity(result);
            z.set_sign(result);
            z.set_zero(result);
            store.change(z, result);
        }
    }
}

macro_rules! rotate_shift_functions_impl {
    ($fn_impl: ident $fn_impl2: ident $fn_general: ident
    $fn_store: ident $fn_a: ident) => {
        pub fn $fn_a<Z>(z: &mut Z)
        where
            Z: Z80Internal + ?Sized
        {
            $fn_impl2(z, A);
        }
        rotate_shift_functions_noa_impl!{$fn_impl $fn_impl2 $fn_general $fn_store}
    }
}

pub fn rst<Z>(z: &mut Z, p: u16)
where
    Z: Z80Internal + Memory16 + ?Sized,
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
    Z: Z80Internal + Memory16 + Z80Irq + Inbox + ?Sized,
{
    // The Z80 manual implies that IFF2 is set to IFF1, but this
    // is false (see Young 5.3)
    manifests::NONMASKABLE_INTERRUPT.send(z, Payload::U64([0]));
    z.inc_r(1);
    z.set_iff1(false);
    z.clear_nmi();
    z.inc_cycles(11);
    if z.halted() {
        let pc = PC.view(z);
        PC.change(z, pc.wrapping_add(1));
    }
    rst(z, 0x66);
}

pub fn maskable_interrupt<Z>(z: &mut Z, x: u8) -> bool
where
    Z: Z80Internal + Memory16 + Z80Irq + Inbox + ?Sized,
{
    if z.iff1() {
        manifests::MASKABLE_INTERRUPT_ALLOWED.send(z, Payload::U64([0]));

        z.inc_r(1);

        z.set_iff1(false);
        z.set_iff2(false);

        if z.halted() {
            let pc = PC.view(z);
            PC.change(z, pc.wrapping_add(1));
        }

        let im = z.interrupt_mode();
        match im {
            Im1 => {
                rst(z, 0x38);
                z.inc_cycles(13);
            }
            Im2 => {
                let i = I.view(z);
                let new_pc = utilities::to16(x, i);
                rst(z, new_pc);
                z.inc_cycles(19);
            }
            _ => unimplemented!(),
        }
        true
    } else {
        manifests::MASKABLE_INTERRUPT_DENIED.send(z, Payload::U64([0]));
        false
    }
}

//// 8-Bit Load Group
/////////////////////

pub fn ld<Z, T1, T2>(z: &mut Z, arg1: T1, arg2: T2)
where
    Z: Z80Internal + ?Sized,
    T1: Changeable<u8, Z>,
    T2: Viewable<u8, Z>,
{
    let val = arg2.view(z);
    arg1.change(z, val);
}

// XXX text about interrupts in manual
pub fn ld_ir<Z>(z: &mut Z, arg1: Reg8, arg2: Reg8)
where
    Z: Z80Internal + ?Sized,
{
    let val = arg2.view(z);
    arg1.change(z, val);
    let iff2 = z.iff2();
    z.set_sign(val);
    z.set_zero(val);
    z.clear_flag(NF | HF);
    z.set_flag_by(PF, iff2);
}

//// 16-Bit Load Group
//////////////////////

pub fn ld16<Z, T1, T2>(z: &mut Z, arg1: T1, arg2: T2)
where
    Z: Z80Internal + ?Sized,
    T1: Changeable<u16, Z>,
    T2: Viewable<u16, Z>,
{
    let val = arg2.view(z);
    arg1.change(z, val);
}

pub fn push<Z>(z: &mut Z, reg: Reg16)
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    let (lo, hi) = utilities::to8(reg.view(z));
    let sp = SP.view(z);
    Address(sp.wrapping_sub(1)).change(z, hi);
    Address(sp.wrapping_sub(2)).change(z, lo);
    SP.change(z, sp.wrapping_sub(2));
}

pub fn pop<Z>(z: &mut Z, reg: Reg16)
where
    Z: Z80Internal + Memory16 + ?Sized,
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
    Z: Z80Internal + ?Sized,
    T1: Changeable<u16, Z>,
{
    let val1 = reg1.view(z);
    let val2 = reg2.view(z);
    reg1.change(z, val2);
    reg2.change(z, val1);
}

pub fn exx<Z>(z: &mut Z)
where
    Z: Z80Internal + ?Sized,
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
    Z: Z80Internal + Memory16 + ?Sized,
{
    let hl = HL.view(z);
    let de = DE.view(z);
    let bc = BC.view(z);

    let phl = Viewable::<u8, Z>::view(Address(hl), z);
    Address(de).change(z, phl);

    HL.change(z, hl.wrapping_add(inc));
    DE.change(z, de.wrapping_add(inc));
    BC.change(z, bc.wrapping_sub(1));

    z.clear_flag(HF | NF);
    z.set_flag_by(PF, bc != 1);
}

pub fn ldi<Z>(z: &mut Z)
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    ldid(z, 1);
}

pub fn ldd<Z>(z: &mut Z)
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    ldid(z, 0xFFFF);
}

pub fn ldir<Z>(z: &mut Z)
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    loop {
        ldi(z);
        if BC.view(z) == 0 {
            z.inc_cycles(17);
            return;
        }
        z.inc_cycles(21);

        // check the possibility that we have overwritten our own opcode
        let pc = PC.view(z);
        let apc1 = Viewable::<u8, Z>::view(Address(pc.wrapping_sub(2)), z);
        let apc2 = Viewable::<u8, Z>::view(Address(pc.wrapping_sub(1)), z);
        if apc1 != 0xED || apc2 != 0xB0 {
            PC.change(z, pc.wrapping_sub(2));
            return;
        }
        z.inc_r(2);
    }
}

pub fn lddr<Z>(z: &mut Z)
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    loop {
        ldd(z);
        if BC.view(z) == 0 {
            z.inc_cycles(17);
            return;
        }
        z.inc_cycles(21);

        // check the possibility that we have overwritten our own opcode
        let pc = PC.view(z);
        let apc1 = Viewable::<u8, Z>::view(Address(pc.wrapping_sub(2)), z);
        let apc2 = Viewable::<u8, Z>::view(Address(pc.wrapping_sub(1)), z);
        if apc1 != 0xED || apc2 != 0xB8 {
            PC.change(z, pc.wrapping_sub(1));
            return;
        }
        z.inc_r(2);
    }
}

pub fn cpid<Z>(z: &mut Z, inc: u16)
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    let bc = BC.view(z);
    let a = A.view(z);
    let hl = HL.view(z);

    let phl: u8 = Address(HL).view(z);
    let result = a.wrapping_sub(phl);

    HL.change(z, hl.wrapping_add(inc));
    BC.change(z, bc.wrapping_sub(1));

    z.set_sign(result);
    z.set_zero(result);
    z.set_flag_by(HF, phl & 0xF > a & 0xF);
    z.set_flag_by(PF, bc != 1);
    z.set_flag(NF);
}

pub fn cpi<Z>(z: &mut Z)
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    cpid(z, 1);
}

pub fn cpir<Z>(z: &mut Z)
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    while {
        cpi(z);
        z.inc_cycles(2);
        BC.view(z) != 0 && !z.is_set_flag(ZF)
    } {
        // r was already incremented twice by `run`
        z.inc_r(2);
    }

    z.inc_cycles(17);
}

pub fn cpd<Z>(z: &mut Z)
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    cpid(z, 0xFFFF);
}

pub fn cpdr<Z>(z: &mut Z)
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    while {
        cpd(z);
        z.inc_cycles(21);
        BC.view(z) != 0 && !z.is_set_flag(ZF)
    } {
        // r was already incremented twice by `run`
        z.inc_r(2);
    }

    z.inc_cycles(17);
}

//// 8-Bit Arithmetic Group
///////////////////////////

fn add_impl<Z>(z: &mut Z, a: u8, x: u8, cf: u8) -> u8
where
    Z: Z80Internal + ?Sized,
{
    // XXX optimize?
    let result16 = (x as u16).wrapping_add(a as u16).wrapping_add(cf as u16);
    let result8 = result16 as u8;

    z.set_zero(result8);
    z.set_sign(result8);

    z.set_flag_by(CF, result16 & (1 << 8) != 0);

    // carry from bit 3 happened if:
    // x and a have same bit 4 AND result is set OR
    // x and a have different bit 4 AND result is clear
    let hf = (x ^ a ^ result8) & (1 << 4) != 0;
    z.set_flag_by(HF, hf);

    // overflow happened if:
    // x and a both have bit 7 AND result does not OR
    // x and a have clear bit 7 AND result is set
    // in other words, x and y have the same bit 7 and
    // result is different
    let overflow = !(x ^ a) & (x ^ result8) & (1 << 7) != 0;
    z.set_flag_by(PF, overflow);

    z.clear_flag(NF);

    result8
}

pub fn add<Z, T1, T2>(z: &mut Z, arg1: T1, arg2: T2)
where
    Z: Z80Internal + Memory16 + ?Sized,
    T1: Changeable<u8, Z>,
    T2: Viewable<u8, Z>,
{
    let a = arg1.view(z);
    let b = arg2.view(z);
    let result = add_impl(z, a, b, 0);
    arg1.change(z, result);
}

pub fn adc<Z, T1, T2>(z: &mut Z, arg1: T1, arg2: T2)
where
    Z: Z80Internal + Memory16 + ?Sized,
    T1: Changeable<u8, Z>,
    T2: Viewable<u8, Z>,
{
    let cf = if z.is_set_flag(CF) { 1u8 } else { 0u8 };
    let a = arg1.view(z);
    let x = arg2.view(z);
    let result = add_impl(z, a, x, cf);
    arg1.change(z, result);
}

fn sub_impl<Z>(z: &mut Z, a: u8, x: u8, cf: u8) -> u8
where
    Z: Z80Internal + ?Sized,
{
    let result = add_impl(z, a, !x, 1 ^ cf);
    let cf_set = z.is_set_flag(CF);
    let hf_set = z.is_set_flag(HF);
    z.set_flag_by(CF, !cf_set);
    z.set_flag_by(HF, !hf_set);
    z.set_flag(NF);
    result
}

pub fn sub<Z, T1, T2>(z: &mut Z, arg1: T1, arg2: T2)
where
    Z: Z80Internal + Memory16 + ?Sized,
    T1: Changeable<u8, Z>,
    T2: Viewable<u8, Z>,
{
    let a = arg1.view(z);
    let x = arg2.view(z);
    let result = sub_impl(z, a, x, 0);
    arg1.change(z, result);
}

pub fn sbc<Z, T1, T2>(z: &mut Z, arg1: T1, arg2: T2)
where
    Z: Z80Internal + Memory16 + ?Sized,
    T1: Changeable<u8, Z>,
    T2: Viewable<u8, Z>,
{
    let cf = if z.is_set_flag(CF) { 1u8 } else { 0u8 };
    let a = arg1.view(z);
    let x = arg2.view(z);
    let result = sub_impl(z, a, x, cf);
    arg1.change(z, result);
}

fn andor_impl<Z>(z: &mut Z, result: u8)
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    A.change(z, result);

    // note that for AND and OR, the manual says PF is set according to whether
    // there is overflow. I'm betting that is a mistake.
    z.set_parity(result);
    z.set_sign(result);
    z.set_zero(result);
    z.clear_flag(HF | NF | CF);
}

pub fn and<Z, T1>(z: &mut Z, arg: T1)
where
    Z: Z80Internal + Memory16 + ?Sized,
    T1: Viewable<u8, Z>,
{
    let result = arg.view(z) & A.view(z);
    andor_impl(z, result);
    z.set_flag(HF);
}

pub fn or<Z, T1>(z: &mut Z, arg: T1)
where
    Z: Z80Internal + Memory16 + ?Sized,
    T1: Viewable<u8, Z>,
{
    let result = arg.view(z) | A.view(z);
    andor_impl(z, result);
}

pub fn xor<Z, T1>(z: &mut Z, arg: T1)
where
    Z: Z80Internal + Memory16 + ?Sized,
    T1: Viewable<u8, Z>,
{
    let result = arg.view(z) ^ A.view(z);
    andor_impl(z, result);
}

pub fn cp<Z, T1>(z: &mut Z, arg: T1)
where
    Z: Z80Internal + Memory16 + ?Sized,
    T1: Viewable<u8, Z>,
{
    let x = arg.view(z);
    let a = A.view(z);
    // cp is like a subtraction whose result we ignore
    sub_impl(z, a, x, 0);
}

pub fn inc<Z, T1>(z: &mut Z, arg: T1)
where
    Z: Z80Internal + Memory16 + ?Sized,
    T1: Changeable<u8, Z>,
{
    let x = arg.view(z);
    let result = x.wrapping_add(1);
    arg.change(z, result);
    z.set_zero(result);
    z.set_sign(result);
    z.set_flag_by(HF, x & 0xF == 0xF);
    z.set_flag_by(PF, x == 0x7F);
    z.clear_flag(NF);
}

pub fn dec<Z, T1>(z: &mut Z, arg: T1)
where
    Z: Z80Internal + Memory16 + ?Sized,
    T1: Changeable<u8, Z>,
{
    let x = arg.view(z);
    let result = x.wrapping_sub(1);
    arg.change(z, result);
    z.set_zero(result);
    z.set_sign(result);
    z.set_flag_by(HF, x & 0xF == 0);
    z.set_flag_by(PF, x == 0x80);
    z.set_flag(NF);
}

//// General-Purpose Arithmetic and CPU Control Groups
//////////////////////////////////////////////////////

pub fn daa<Z>(z: &mut Z)
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    // see the table in Young
    let a = A.view(z);
    let cf = z.is_set_flag(CF);
    let hf = z.is_set_flag(HF);
    let nf = z.is_set_flag(NF);
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

    z.set_parity(new_a);
    z.set_zero(new_a);
    z.set_sign(new_a);
    z.set_flag_by(CF, new_cf != 0);
    z.set_flag_by(HF, new_hf != 0);
}

pub fn cpl<Z>(z: &mut Z)
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    let a = A.view(z);
    A.change(z, !a);
    z.set_flag(HF | NF);
}

pub fn neg<Z>(z: &mut Z)
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    // subtracts A from 0
    let a = A.view(z);
    let result = sub_impl(z, 0, a, 0);
    A.change(z, result);
    z.set_flag_by(PF, a == 0x80);
    z.set_flag_by(CF, a != 0);
}

pub fn ccf<Z>(z: &mut Z)
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    let cf = z.is_set_flag(CF);
    z.set_flag_by(HF, cf);
    z.set_flag_by(CF, !cf);
    z.clear_flag(NF);
}

pub fn scf<Z>(z: &mut Z)
where
    Z: Z80Internal + ?Sized,
{
    z.clear_flag(HF | NF);
    z.set_flag(CF);
}

pub fn nop<Z>(_z: &mut Z)
where
    Z: Z80Internal + ?Sized,
{
}

pub fn halt<Z>(z: &mut Z)
where
    Z: Z80Internal + ?Sized,
{
    z.set_halted(true);
}

pub fn di<Z>(z: &mut Z)
where
    Z: Z80Internal + ?Sized,
{
    z.set_iff1(false);
    z.set_iff2(false);
}

/// `ei` instruction.
///
/// The `ei` instruction leaves `iff1` in an intermediate state. A maskable
/// interrupt may not happen until after the *following* instruction. This
/// requires support from the emulator: calling this function is not sufficient
/// to emulate `ei`.
pub fn ei<Z>(z: &mut Z)
where
    Z: Z80Internal + ?Sized,
{
    z.set_iff2(true);
}

pub fn im<Z>(z: &mut Z, m: u8)
where
    Z: Z80Internal + ?Sized,
{
    match m {
        0 => z.set_interrupt_mode(Im0),
        1 => z.set_interrupt_mode(Im1),
        2 => z.set_interrupt_mode(Im2),
        _ => panic!("Z80: Invalid interrupt mode"),
    }
}

pub fn im1<Z>(z: &mut Z)
where
    Z: Z80Internal + ?Sized,
{
    z.set_interrupt_mode(Im1);
}

pub fn im2<Z>(z: &mut Z)
where
    Z: Z80Internal + ?Sized,
{
    z.set_interrupt_mode(Im2);
}

//// 16-Bit Arithmetic Group
////////////////////////////

fn add16_impl<Z>(z: &mut Z, x: u16, y: u16, cf: u16) -> u16
where
    Z: Z80Internal + ?Sized,
{
    // XXX optimize?
    let result32 = (x as u32).wrapping_add(y as u32).wrapping_add(cf as u32);
    let result16 = result32 as u16;

    z.set_flag_by(CF, result32 & (1 << 16) != 0);

    // carry from bit 11 happened if:
    // x and y have same bit 12 AND result is set OR
    // x and y have different bit 12 AND result is clear
    let hf = (x ^ y ^ result16) & (1 << 12) != 0;
    z.set_flag_by(HF, hf);

    z.clear_flag(NF);

    result16
}

pub fn add16<Z>(z: &mut Z, arg1: Reg16, arg2: Reg16)
where
    Z: Z80Internal + ?Sized,
{
    let x = arg1.view(z);
    let y = arg2.view(z);
    let result = add16_impl(z, x, y, 0);
    arg1.change(z, result);
}

fn adc16_impl<Z>(z: &mut Z, x: u16, y: u16, cf: u16) -> u16
where
    Z: Z80Internal + ?Sized,
{
    let result = add16_impl(z, x, y, cf as u16);

    z.set_sign((result >> 8) as u8);
    z.set_zero((result as u8) | (result >> 8) as u8);

    // overflow happened if:
    // x and y both have bit 15 AND result does not OR
    // x and y have clear bit 15 AND result is set
    // in other words, x and y have the same bit 15, which is different from bit
    // 15 of result
    let overflow = !(x ^ y) & (x ^ result) & (1 << 15) != 0;
    z.set_flag_by(PF, overflow);

    result
}

pub fn adc16<Z>(z: &mut Z, arg1: Reg16, arg2: Reg16)
where
    Z: Z80Internal + ?Sized,
{
    let x = arg1.view(z);
    let y = arg2.view(z);
    let cf = if z.is_set_flag(CF) { 1u8 } else { 0u8 };
    let result = adc16_impl(z, x, y, cf as u16);
    arg1.change(z, result);
}

pub fn sbc16<Z>(z: &mut Z, arg1: Reg16, arg2: Reg16)
where
    Z: Z80Internal + ?Sized,
{
    let x = arg1.view(z);
    let y = arg2.view(z);
    let cf = if z.is_set_flag(CF) { 1u8 } else { 0u8 };
    let result = adc16_impl(z, x, !y, (1 ^ cf) as u16);
    arg1.change(z, result);
    let cf = z.is_set_flag(CF);
    let hf = z.is_set_flag(HF);
    z.set_flag_by(CF, !cf);
    z.set_flag_by(HF, !hf);
    z.set_flag(NF);
}

pub fn inc16<Z>(z: &mut Z, arg: Reg16)
where
    Z: Z80Internal + ?Sized,
{
    let val = arg.view(z);
    arg.change(z, val.wrapping_add(1));
}

pub fn dec16<Z>(z: &mut Z, arg: Reg16)
where
    Z: Z80Internal + ?Sized,
{
    let val = arg.view(z);
    arg.change(z, val.wrapping_sub(1));
}

//// Rotate and Shift Group
///////////////////////////

fn rlc_impl<Z>(z: &mut Z, x: u8) -> u8
where
    Z: Z80Internal + ?Sized,
{
    z.set_flag_by(CF, x & 0x80 != 0);
    x.rotate_left(1)
}

rotate_shift_functions_impl!{
    rlc_impl rlc_impl2 rlc rlc_store rlca
}

fn rl_impl<Z>(z: &mut Z, x: u8) -> u8
where
    Z: Z80Internal + ?Sized,
{
    let mut result = x << 1;
    if z.is_set_flag(CF) {
        result |= 1;
    } else {
        result &= !1;
    }
    z.set_flag_by(CF, x & 0x80 != 0);
    result
}

rotate_shift_functions_impl!{
    rl_impl rl_impl2 rl rl_store rla
}

fn rrc_impl<Z>(z: &mut Z, x: u8) -> u8
where
    Z: Z80Internal + ?Sized,
{
    z.set_flag_by(CF, x & 1 != 0);
    x.rotate_right(1)
}

rotate_shift_functions_impl!{
    rrc_impl rrc_impl2 rrc rrc_store rrca
}

fn rr_impl<Z>(z: &mut Z, x: u8) -> u8
where
    Z: Z80Internal + ?Sized,
{
    let mut result = x >> 1;
    if z.is_set_flag(CF) {
        result |= 0x80;
    } else {
        result &= !0x80;
    }
    z.set_flag_by(CF, x & 1 != 0);
    result
}

rotate_shift_functions_impl!{
    rr_impl rr_impl2 rr rr_store rra
}

fn sla_impl<Z>(z: &mut Z, x: u8) -> u8
where
    Z: Z80Internal + ?Sized,
{
    z.set_flag_by(CF, x & 0x80 != 0);
    x << 1
}

rotate_shift_functions_noa_impl!{
    sla_impl sla_impl2 sla sla_store
}

// SLL is undocumented; see Young
fn sll_impl<Z>(z: &mut Z, x: u8) -> u8
where
    Z: Z80Internal + ?Sized,
{
    z.set_flag_by(CF, x & 0x80 != 0);
    let mut result = x << 1;
    result |= 1;
    result
}

rotate_shift_functions_noa_impl!{
    sll_impl sll_impl2 sll sll_store
}

fn sra_impl<Z>(z: &mut Z, x: u8) -> u8
where
    Z: Z80Internal + ?Sized,
{
    z.set_flag_by(CF, x & 1 != 0);
    let result = ((x as i8) >> 1) as u8;
    result
}

rotate_shift_functions_noa_impl!{
    sra_impl sra_impl2 sra sra_store
}

fn srl_impl<Z>(z: &mut Z, x: u8) -> u8
where
    Z: Z80Internal + ?Sized,
{
    z.set_flag_by(CF, x & 1 != 0);
    x >> 1
}

rotate_shift_functions_noa_impl!{
    srl_impl srl_impl2 srl srl_store
}

pub fn rld<Z>(z: &mut Z)
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    let hl: u8 = Address(HL).view(z);
    let hl_lo: u8 = 0xF & hl;
    let hl_hi: u8 = 0xF0 & hl;
    let a_lo = 0xF & A.view(z);
    let a_hi = 0xF0 & A.view(z);
    Address(HL).change(z, hl_lo << 4 | a_lo);
    A.change(z, hl_hi >> 4 | a_hi);
    let a = A.view(z);

    z.set_parity(a);
    z.set_sign(a);
    z.set_zero(a);
    z.clear_flag(HF | NF);
}

pub fn rrd<Z>(z: &mut Z)
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    let hl: u8 = Address(HL).view(z);
    let hl_lo: u8 = 0xF & hl;
    let hl_hi: u8 = 0xF0 & hl;
    let a_lo = 0xF & A.view(z);
    let a_hi = 0xF0 & A.view(z);
    Address(HL).change(z, a_lo << 4 | hl_hi >> 4);
    A.change(z, hl_lo | a_hi);
    let a = A.view(z);

    z.set_parity(a);
    z.set_sign(a);
    z.set_zero(a);
    z.clear_flag(HF | NF);
}

//// Bit set, reset, and Test Group
///////////////////////////////////

pub fn bit<Z, T>(z: &mut Z, b: u8, arg: T)
where
    Z: Z80Internal + ?Sized,
    T: Viewable<u8, Z>,
{
    let x = arg.view(z);
    let bitflag = 1 << b;
    let x_contains = x & bitflag != 0;

    z.set_flag_by(ZF | PF, !x_contains);
    z.set_flag(HF);
    z.clear_flag(NF);
    z.set_flag_by(SF, b == 7 && x_contains);
}

pub fn set<Z, T>(z: &mut Z, b: u8, arg: T)
where
    Z: Z80Internal + ?Sized,
    T: Changeable<u8, Z>,
{
    let mut x = arg.view(z);
    utilities::set_bit(&mut x, b);
    arg.change(z, x);
}

pub fn set_store<Z, T>(z: &mut Z, b: u8, arg: T, reg: Reg8)
where
    Z: Z80Internal + ?Sized,
    T: Changeable<u8, Z>,
{
    arg.change(z, b);
    let x = arg.view(z);
    reg.change(z, x);
}

pub fn res<Z, T>(z: &mut Z, b: u8, arg: T)
where
    Z: Z80Internal + ?Sized,
    T: Changeable<u8, Z>,
{
    let mut x = arg.view(z);
    utilities::clear_bit(&mut x, b);
    arg.change(z, x);
}

pub fn res_store<Z, T>(z: &mut Z, b: u8, arg: T, reg: Reg8)
where
    Z: Z80Internal + ?Sized,
    T: Changeable<u8, Z>,
{
    res(z, b, arg);
    let x = arg.view(z);
    reg.change(z, x);
}

//// Jump Group
///////////////

pub fn jp<Z, T>(z: &mut Z, arg: T)
where
    Z: Z80Internal + ?Sized,
    T: Viewable<u16, Z>,
{
    let addr = arg.view(z);
    PC.change(z, addr);
}

pub fn jpcc<Z>(z: &mut Z, cc: ConditionCode, arg: u16)
where
    Z: Z80Internal + ?Sized,
{
    if cc.view(z) {
        jp(z, arg);
    }
}

pub fn jr<Z>(z: &mut Z, e: i8)
where
    Z: Z80Internal + ?Sized,
{
    let pc = PC.view(z);
    let new_pc = pc.wrapping_add(e as i16 as u16);
    PC.change(z, new_pc);
}

pub fn jrcc<Z>(z: &mut Z, cc: ConditionCode, e: i8)
where
    Z: Z80Internal + ?Sized,
{
    if cc.view(z) {
        jr(z, e);
        z.inc_cycles(12);
    } else {
        z.inc_cycles(7);
    }
}

pub fn djnz<Z>(z: &mut Z, e: i8)
where
    Z: Z80Internal + ?Sized,
{
    let b = B.view(z);
    let new_b = b.wrapping_sub(1);
    B.change(z, new_b);
    if new_b != 0 {
        jr(z, e);
        z.inc_cycles(13);
    } else {
        z.inc_cycles(8);
    }
}

//// Call and Return Group
//////////////////////////

pub fn call<Z>(z: &mut Z, nn: u16)
where
    Z: Z80Internal + Memory16 + ?Sized,
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
    Z: Z80Internal + Memory16 + ?Sized,
{
    if cc.view(z) {
        call(z, nn);
        z.inc_cycles(17);
    } else {
        z.inc_cycles(10);
    }
}

pub fn ret<Z>(z: &mut Z)
where
    Z: Z80Internal + Memory16 + ?Sized,
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
    Z: Z80Internal + Memory16 + ?Sized,
{
    if cc.view(z) {
        ret(z);
        z.inc_cycles(11);
    } else {
        z.inc_cycles(5);
    }
}

pub fn reti<Z>(z: &mut Z)
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    retn(z);
}

pub fn retn<Z>(z: &mut Z)
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    let iff2 = z.iff2();
    z.set_iff1(iff2);

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
    Z: Z80Internal + Io16_8 + ?Sized,
    T1: Changeable<u8, Z>,
    T2: Viewable<u8, Z>,
{
    let address_lo = arg2.view(z);
    let address_hi = arg1.view(z);
    let address = utilities::to16(address_lo, address_hi);
    let x = z.input(address);
    arg1.change(z, x);
}

fn in_impl<Z, T1>(z: &mut Z, arg: T1) -> u8
where
    Z: Z80Internal + Io16_8 + ?Sized,
    T1: Viewable<u8, Z>,
{
    let address_lo = arg.view(z);
    let address_hi = B.view(z);
    let address = utilities::to16(address_lo, address_hi);
    let x = z.input(address);

    z.set_parity(x);
    z.set_sign(x);
    z.set_zero(x);
    z.clear_flag(HF | NF);

    x
}

pub fn in_f<Z, T1>(z: &mut Z, arg: T1)
where
    Z: Z80Internal + Io16_8 + ?Sized,
    T1: Viewable<u8, Z>,
{
    in_impl(z, arg);
}

pub fn in_c<Z, T1, T2>(z: &mut Z, arg1: T1, arg2: T2)
where
    Z: Z80Internal + Io16_8 + ?Sized,
    T1: Changeable<u8, Z>,
    T2: Viewable<u8, Z>,
{
    let x = in_impl(z, arg2);
    arg1.change(z, x);
}

/// The Z80 manual lists this instruction under IN r, (C) as "undefined" It
/// reads from the input ports and sets flags but doesn't change any register.
pub fn in0<Z>(z: &mut Z)
where
    Z: Z80Internal + Io16_8 + ?Sized,
{
    let addr = BC.view(z);
    let x = z.input(addr);

    z.set_parity(x);
    z.set_sign(x);
    z.set_zero(x);
    z.clear_flag(HF | NF);
}

fn inid_impl<Z>(z: &mut Z, inc: u16) -> u8
where
    Z: Z80Internal + Memory16 + Io16_8 + ?Sized,
{
    let b = B.view(z);
    let hl = HL.view(z);
    let addr = BC.view(z);
    let x = z.input(addr);
    Address(hl).change(z, x);
    B.change(z, b.wrapping_sub(1));
    HL.change(z, hl.wrapping_add(inc));
    b.wrapping_sub(1)
}

pub fn ini<Z>(z: &mut Z)
where
    Z: Z80Internal + Memory16 + Io16_8 + ?Sized,
{
    let new_b = inid_impl(z, 1);

    z.set_zero(new_b);
    z.set_flag(NF);
}

pub fn inir<Z>(z: &mut Z)
where
    Z: Z80Internal + Memory16 + Io16_8 + ?Sized,
{
    while {
        z.inc_cycles(21);
        inid_impl(z, 1) != 0
    } {
        // r was already incremented twice by `run`
        z.inc_r(2);
    }

    z.set_flag(ZF | NF);

    z.inc_cycles(16);
}

pub fn ind<Z>(z: &mut Z)
where
    Z: Z80Internal + Memory16 + Io16_8 + ?Sized,
{
    let new_b = inid_impl(z, 0xFFFF);

    z.set_zero(new_b);
    z.set_flag(NF);
}

pub fn indr<Z>(z: &mut Z)
where
    Z: Z80Internal + Memory16 + Io16_8 + ?Sized,
{
    while {
        z.inc_cycles(21);
        inid_impl(z, 0xFFFF) != 0
    } {
        // r was already incremented twice by `run`
        z.inc_r(2);
    }

    z.set_flag(ZF | NF);

    z.inc_cycles(16);
}

pub fn out_n<Z, T1, T2>(z: &mut Z, arg1: T1, arg2: T2)
where
    Z: Z80Internal + Io16_8 + ?Sized,
    T1: Viewable<u8, Z>,
    T2: Viewable<u8, Z>,
{
    let address_lo = arg1.view(z);
    let address_hi = A.view(z);
    let address = utilities::to16(address_lo, address_hi);
    let x = arg2.view(z);
    z.output(address, x);
}

pub fn out_c<Z, T1, T2>(z: &mut Z, arg1: T1, arg2: T2)
where
    Z: Z80Internal + Io16_8 + ?Sized,
    T1: Viewable<u8, Z>,
    T2: Viewable<u8, Z>,
{
    let address_lo = arg1.view(z);
    let address_hi = B.view(z);
    let address = utilities::to16(address_lo, address_hi);
    let x = arg2.view(z);
    z.output(address, x);
}

fn outid_impl<Z>(z: &mut Z, inc: u16)
where
    Z: Z80Internal + Memory16 + Io16_8 + ?Sized,
{
    let b = B.view(z);
    let new_b = b.wrapping_sub(1);
    B.change(z, new_b);
    let addr = BC.view(z);
    let hl = HL.view(z);
    let x = Address(hl).view(z);
    z.output(addr, x);
    HL.change(z, hl.wrapping_add(inc));
}

pub fn outi<Z>(z: &mut Z)
where
    Z: Z80Internal + Memory16 + Io16_8 + ?Sized,
{
    outid_impl(z, 1);
    let new_b = B.view(z);

    z.set_zero(new_b);
    z.set_flag(NF);
}

pub fn otir<Z>(z: &mut Z)
where
    Z: Z80Internal + Memory16 + Io16_8 + ?Sized,
{
    while {
        z.inc_cycles(21);
        outid_impl(z, 1);
        B.view(z) != 0
    } {
        // r was already incremented twice by `run`
        z.inc_r(2);
    }

    z.set_flag(ZF | NF);

    z.inc_cycles(16);
}

pub fn outd<Z>(z: &mut Z)
where
    Z: Z80Internal + Memory16 + Io16_8 + ?Sized,
{
    outid_impl(z, 0xFFFF);
    let new_b = B.view(z);

    z.set_zero(new_b);
    z.set_flag(NF);
}

pub fn otdr<Z>(z: &mut Z)
where
    Z: Z80Internal + Memory16 + Io16_8 + ?Sized,
{
    while {
        z.inc_cycles(21);
        outid_impl(z, 0xFFFF);
        B.view(z) != 0
    } {
        // r was already incremented twice by `run`
        z.inc_r(2);
    }

    z.set_flag(ZF | NF);

    z.inc_cycles(16);
}
