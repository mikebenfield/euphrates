// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

#![allow(dead_code)]

use ::log;
use ::bits::*;
use ::hardware::z80::types::*;
use ::hardware::io::Io;

fn set_sign<I: Io>(z80: &mut Z80<I>, x: u8) {
    let mut f = F.get(z80);
    assign_bit(&mut f, SF, x, SF);
    F.set(z80, f);
}

fn set_zero<I: Io>(z80: &mut Z80<I>, x: u8) {
    let mut f = F.get(z80);
    assign_bit(&mut f, ZF, (x == 0) as u8, 0);
    F.set(z80, f);
}

fn set_parity<I: Io>(z80: &mut Z80<I>, x: u8) {
    let parity = x.count_ones() %2 == 0;
    let mut f = F.get(z80);
    assign_bit(&mut f, PF, parity as u8, 0);
    F.set(z80, f);
}

//// Interrupts
///////////////

pub fn rst<I: Io>(z: &mut Z80<I>, p: u8) {
    let sp = SP.get(z);
    let pch = PCH.get(z);
    let pcl = PCL.get(z);
    Address(sp.wrapping_sub(1)).set(z, pch);
    Address(sp.wrapping_sub(2)).set(z, pcl);
    SP.set(z, sp.wrapping_sub(2));
    PCH.set(z, 0);
    PCL.set(z, p);
}

pub fn nonmaskable_interrupt<I: Io>(z: &mut Z80<I>) {
    let iff1 = z.iff1;
    z.iff2 = iff1 != 0;
    rst(z, 0x66);
}

pub fn maskable_interrupt<I: Io>(z: &mut Z80<I>) -> bool {
    if z.iff1 < z.cycles {
        log_major!("Z80: Maskable interrupt allowed");

        z.iff1 = 0xFFFFFFFFFFFFFFFF;
        z.iff2 = false;

        let im = z.interrupt_mode;
        match im {
            Im1 => {
                rst(z, 0x38);
                z.cycles += 13;
            },
            _ => unimplemented!(),
        }
        true
    } else {
        log_major!("Z80: Maskable interrupt denied");
        false
    }
}

//// 8-Bit Load Group
/////////////////////

pub fn ld<I, T1, T2>(z: &mut Z80<I>, arg1: T1, arg2: T2)
where
    I: Io,
    T1: Settable<u8>,
    T2: Gettable<u8>,
{
    let val = arg2.get(z);
    arg1.set(z, val);
}

// XXX text about interrupts in manual
pub fn ld8_ir<I: Io>(z: &mut Z80<I>, arg: Reg8) {
    let val = arg.get(z);
    set_sign(z, val);
    set_zero(z, val);
    let mut f = F.get(z);
    clear_bit(&mut f, HF);
    assign_bit(&mut f, PF, z.iff2 as u8, 0);
    clear_bit(&mut f, NF);
    F.set(z, f);
}

//// 16-Bit Load Group
//////////////////////

pub fn ld16<I, T1, T2>(z: &mut Z80<I>, arg1: T1, arg2: T2)
where
    I: Io,
    T1: Settable<u16>,
    T2: Gettable<u16>,
{
    let val = arg2.get(z);
    arg1.set(z, val);
}

pub fn push<I: Io>(z: &mut Z80<I>, reg: Reg16) {
    let (lo, hi) = to8(reg.get(z));
    let sp = SP.get(z);
    Address(sp.wrapping_sub(1)).set(z, hi);
    Address(sp.wrapping_sub(2)).set(z, lo);
    SP.set(z, sp.wrapping_sub(2));
}

pub fn pop<I: Io>(z: &mut Z80<I>, reg: Reg16) {
    let sp = SP.get(z);
    let lo = Address(sp).get(z);
    let hi = Address(sp.wrapping_add(1)).get(z);
    reg.set(z, to16(lo, hi));
    SP.set(z, sp.wrapping_add(2));
}

//// Exchange, Block Transfer, and Search Group
///////////////////////////////////////////////

pub fn ex<I, T1>(z: &mut Z80<I>, reg1: T1, reg2: Reg16)
where
    I: Io,
    T1: Settable<u16>,
{
    let val1 = reg1.get(z);
    let val2 = reg2.get(z);
    reg1.set(z, val2);
    reg2.set(z, val1);
}

pub fn exx<I: Io>(z: &mut Z80<I>) {
    for &(reg1, reg2) in [(BC, BC0), (DE, DE0), (HL, HL0)].iter() {
        let val1 = reg1.get(z);
        let val2 = reg2.get(z);
        reg1.set(z, val2);
        reg2.set(z, val1);
    }
}

fn ld_id_impl<I: Io>(z: &mut Z80<I>, inc: i8) {
    let val_hl: u8 = Address(HL).get(z);
    Address(DE).set(z, val_hl);
    let hl = HL.get(z);
    HL.set(z, hl.wrapping_add(inc as i16 as u16));
    let de = DE.get(z);
    DE.set(z, de.wrapping_add(inc as i16 as u16));
    let bc = BC.get(z);
    BC.set(z, bc.wrapping_sub(1));
}

fn ld_id_flag_impl<I: Io>(z: &mut Z80<I>) {
    let mut f = F.get(z);
    clear_bit(&mut f, HF);
    clear_bit(&mut f, NF);
    let b = (BC.get(z) != 0) as u8;
    assign_bit(&mut f, PF, b, 0);
    F.set(z, f);
}

pub fn ldid<I: Io>(z: &mut Z80<I>, inc: u16) {
    let hl = HL.get(z);
    let de = DE.get(z);
    let bc = BC.get(z);

    let phl = Gettable::<u8>::get(Address(hl), z);
    Address(de).set(z, phl);

    HL.set(z, hl.wrapping_add(inc));
    DE.set(z, de.wrapping_add(inc));
    BC.set(z, bc.wrapping_sub(1));

    let mut f = F.get(z);
    clear_bit(&mut f, HF);
    clear_bit(&mut f, NF);
    let b = bc != 1;
    assign_bit(&mut f, PF, b as u8, 0);
    F.set(z, f);
}

pub fn ldi<I: Io>(z: &mut Z80<I>) {
    ldid(z, 1);
}

pub fn ldd<I: Io>(z: &mut Z80<I>) {
    ldid(z, 0xFFFF);
}

pub fn ldir<I: Io>(z: &mut Z80<I>) {
    loop {
        ldi(z);
        if BC.get(z) == 0 {
            z.cycles += 17;
            return;
        }
        z.cycles += 21;

        // check the possibility that we have overwritten our own opcode
        let pc = PC.get(z);
        let apc1 = Gettable::<u8>::get(Address(pc.wrapping_sub(2)), z);
        let apc2 = Gettable::<u8>::get(Address(pc.wrapping_sub(1)), z);
        if apc1 != 0xED || apc2 != 0xB0 {
            PC.set(z, pc.wrapping_sub(2));
            return;
        }
        inc_r(z);
        inc_r(z);
    }
}

pub fn lddr<I: Io>(z: &mut Z80<I>) {
    loop {
        ldd(z);
        if BC.get(z) == 0 {
            z.cycles += 17;
            return;
        }
        z.cycles += 21;

        // check the possibility that we have overwritten our own opcode
        let pc = PC.get(z);
        let apc1 = Gettable::<u8>::get(Address(pc.wrapping_sub(2)), z);
        let apc2 = Gettable::<u8>::get(Address(pc.wrapping_sub(1)), z);
        if apc1 != 0xED || apc2 != 0xB8 {
            PC.set(z, pc.wrapping_sub(1));
            return;
        }
        inc_r(z);
        inc_r(z);
    }
}

fn cpid<I: Io>(z: &mut Z80<I>, inc: u16) {
    let bc = BC.get(z);
    let a = A.get(z);
    let hl = HL.get(z);

    let phl: u8 = Address(HL).get(z);
    let result = a.wrapping_sub(phl);

    set_sign(z, result);
    set_zero(z, result);
    let mut f = F.get(z);
    assign_bit(&mut f, HF, (phl & 0xF > a & 0xF) as u8, 0);
    assign_bit(&mut f, PF, (bc != 1) as u8, 0);
    set_bit(&mut f, NF);
    F.set(z, f);
    HL.set(z, hl.wrapping_add(inc));
    BC.set(z, bc.wrapping_sub(1));
}

pub fn cpi<I: Io>(z: &mut Z80<I>) {
    cpid(z, 1);
}

pub fn cpir<I: Io>(z: &mut Z80<I>) {
    while {
        cpi(z);
        z.cycles += 21;
        BC.get(z) != 0 && F.get(z) & (1 << ZF) == 0
    } {
        // r was already incremented twice by `run`
        inc_r(z);
        inc_r(z);
    }

    z.cycles += 17;
}

pub fn cpd<I: Io>(z: &mut Z80<I>) {
    cpid(z, 0xFFFF);
}

pub fn cpdr<I: Io>(z: &mut Z80<I>) {
    while {
        cpd(z);
        z.cycles += 21;
        BC.get(z) != 0 && F.get(z) & (1 << ZF) == 0
    } {
        // r was already incremented twice by `run`
        inc_r(z);
        inc_r(z);
    }

    z.cycles += 17;
}

//// 8-Bit Arithmetic Group
///////////////////////////

fn add_impl<I: Io>(z: &mut Z80<I>, a: u8, x: u8, cf: u8) -> u8 {
    // XXX optimize?
    let result16 = (x as u16).wrapping_add(a as u16).wrapping_add(cf as u16);
    let result8 = result16 as u8;

    set_zero(z, result8);
    set_sign(z, result8);

    let mut f = F.get(z);

    assign_bit(&mut f, CF, (result16 >> 8) as u8, 0);

    // carry from bit 3 happened if:
    // x and a have same bit 4 AND result is set OR
    // x and a have different bit 4 AND result is clear
    assign_bit(&mut f, HF, (x ^ a ^ result8), 4);

    // overflow happened if:
    // x and a both have bit 7 AND result does not OR
    // x and a have clear bit 7 AND result is set
    assign_bit(&mut f, PF, !(x ^ a) & (x ^ result8), 7);


    clear_bit(&mut f, NF);
    F.set(z, f);

    return result8;
}

pub fn add<I, T1, T2>(z: &mut Z80<I>, arg1: T1, arg2: T2)
where
    I: Io,
    T1: Settable<u8>,
    T2: Gettable<u8>,
{
    let a = arg1.get(z);
    let b = arg2.get(z);
    let result = add_impl(z, a, b, 0);
    arg1.set(z, result);
}

pub fn adc<I, T1, T2>(z: &mut Z80<I>, arg1: T1, arg2: T2)
where
    I: Io,
    T1: Settable<u8>,
    T2: Gettable<u8>,
{
    let mut cf = 0u8;
    let f = F.get(z);
    assign_bit(&mut cf, 0, f, CF);
    let a = arg1.get(z);
    let x = arg2.get(z);
    let result = add_impl(z, a, x, cf);
    arg1.set(z, result);
}

fn sub_impl<I: Io>(z: &mut Z80<I>, a: u8, x: u8, cf: u8) -> u8 {
    // XXX check that flags are set correctly
    let result = add_impl(z, a, !x, 1 ^ cf);
    let mut f = F.get(z);
    f ^= 1 << CF;
    f ^= 1 << HF;
    set_bit(&mut f, NF);
    F.set(z, f);
    result
}

pub fn sub<I, T1, T2>(z: &mut Z80<I>, arg1: T1, arg2: T2)
where
    I: Io,
    T1: Settable<u8>,
    T2: Gettable<u8>,
{
    let a = arg1.get(z);
    let x = arg2.get(z);
    let result = sub_impl(z, a, x, 0);
    arg1.set(z, result);
}

pub fn sbc<I, T1, T2>(z: &mut Z80<I>, arg1: T1, arg2: T2)
where
    I: Io,
    T1: Settable<u8>,
    T2: Gettable<u8>,
{
    let mut cf = 0u8;
    let f = F.get(z);
    assign_bit(&mut cf, 0, f, CF);
    let a = arg1.get(z);
    let x = arg2.get(z);
    let result = sub_impl(z, a, x, cf);
    arg1.set(z, result);
}

fn andor_impl<I: Io>(z: &mut Z80<I>, result: u8) {
    A.set(z, result);

    // note that for AND and OR, the manual says PF is set according to whether
    // there is overflow. I'm betting that is a mistake.
    set_parity(z, result);
    set_sign(z, result);
    set_zero(z, result);
    let mut f = F.get(z);

    clear_bit(&mut f, HF);
    clear_bit(&mut f, NF);
    clear_bit(&mut f, CF);

    F.set(z, f);
}

pub fn and<I, T1>(z: &mut Z80<I>, arg: T1)
where
    I: Io,
    T1: Gettable<u8>,
{
    let result = arg.get(z) & A.get(z);
    andor_impl(z, result);
    let mut f = F.get(z);
    set_bit(&mut f, HF);
    F.set(z, f);
}

pub fn or<I, T1>(z: &mut Z80<I>, arg: T1)
where
    I: Io,
    T1: Gettable<u8>,
{
    let result = arg.get(z) | A.get(z);
    andor_impl(z, result);
}

pub fn xor<I, T1>(z: &mut Z80<I>, arg: T1)
where
    I: Io,
    T1: Gettable<u8>,
{
    let result = arg.get(z) ^ A.get(z);
    andor_impl(z, result);
}

fn cp_impl<I, T1>(z: &mut Z80<I>, arg: T1)
where
    I: Io,
    T1: Gettable<u8>,
{
    let x = arg.get(z);
    let a = A.get(z);
    sub_impl(z, a, x, 0);
    A.set(z, a);
}

pub fn cp<I, T1>(z: &mut Z80<I>, arg: T1)
where
    I: Io,
    T1: Gettable<u8>,
{
    cp_impl(z, arg);
}

pub fn inc<I, T1>(z: &mut Z80<I>, arg: T1)
where
    I: Io,
    T1: Settable<u8>,
{
    let x = arg.get(z);
    let result = x.wrapping_add(1);
    arg.set(z, result);
    set_zero(z, result);
    set_sign(z, result);
    let mut f = F.get(z);
    clear_bit(&mut f, NF);
    assign_bit(&mut f, HF, ((x & 0xF) == 0xF) as u8, 0);
    assign_bit(&mut f, PF, (x == 0x7F) as u8, 0);
    F.set(z, f);
}

pub fn dec<I, T1>(z: &mut Z80<I>, arg: T1)
where
    I: Io,
    T1: Settable<u8>,
{
    let x = arg.get(z);
    let result = x.wrapping_sub(1);
    arg.set(z, result);
    set_zero(z, result);
    let mut f = F.get(z);
    assign_bit(&mut f, SF, result, SF);
    assign_bit(&mut f, HF, ((x & 0xF) == 0) as u8, 0);
    assign_bit(&mut f, PF, (x == 0x80) as u8, 0);
    set_bit(&mut f, NF);
    F.set(z, f);
}

//// General-Purpose Arithmetic and CPU Control Groups
//////////////////////////////////////////////////////

pub fn daa<I: Io>(z: &mut Z80<I>) {
    // see the table in Young
    let a = A.get(z);
    let cf = F.get(z) & (1 << CF) != 0;
    let hf = F.get(z) & (1 << HF) != 0;
    let nf = F.get(z) & (1 << NF) != 0;
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
    A.set(z, new_a);
    set_parity(z, new_a);
    set_zero(z, new_a);
    set_sign(z, new_a);
    let mut f = F.get(z);
    assign_bit(&mut f, CF, new_cf, 0);
    assign_bit(&mut f, HF, new_hf, 0);
    F.set(z, f);
}

pub fn cpl<I: Io>(z: &mut Z80<I>) {
    let a = A.get(z);
    A.set(z, !a);
    let mut f = F.get(z);
    set_bit(&mut f, HF);
    set_bit(&mut f, NF);
    F.set(z, f);
}

pub fn neg<I: Io>(z: &mut Z80<I>) {
    // subtracts A from 0
    let a = A.get(z);
    let result = sub_impl(z, 0, a, 0);
    let mut f = F.get(z);
    assign_bit(&mut f, PF, (a == 0x80) as u8, 0);
    assign_bit(&mut f, CF, (a != 0) as u8, 0);
    F.set(z, f);
    A.set(z, result);
}

pub fn ccf<I: Io>(z: &mut Z80<I>) {
    let mut f = F.get(z);
    assign_bit(&mut f, HF, F.get(z), CF);
    f ^= 1 << CF;
    clear_bit(&mut f, NF);
    F.set(z, f);
}

pub fn scf<I: Io>(z: &mut Z80<I>) {
    let mut f = F.get(z);
    clear_bit(&mut f, HF);
    clear_bit(&mut f, NF);
    set_bit(&mut f, CF);
    F.set(z, f);
}

pub fn nop<I: Io>(_: &mut Z80<I>) {
}

pub fn halt<I: Io>(z: &mut Z80<I>) {
    z.halted = true;
}

pub fn di<I: Io>(z: &mut Z80<I>) {
    z.iff1 = 0xFFFFFFFFFFFFFFFF;
    z.iff2 = false;
}

pub fn ei<I: Io>(z: &mut Z80<I>) {
    z.iff1 = z.cycles + 4;
    z.iff2 = true;
}

pub fn im<I: Io>(z: &mut Z80<I>, m: u8) {
    match m {
        0 => z.interrupt_mode = Im0,
        1 => z.interrupt_mode = Im1,
        2 => z.interrupt_mode = Im2,
        _ => panic!("Z80: Invalid interrupt mode"),
    }
}

pub fn im1<I: Io>(z: &mut Z80<I>) {
    z.interrupt_mode = Im1;
}

pub fn im2<I: Io>(z: &mut Z80<I>) {
    z.interrupt_mode = Im2;
}

//// 16-Bit Arithmetic Group
////////////////////////////

fn add16_impl<I: Io>(z: &mut Z80<I>, x: u16, y: u16, cf: u16) -> u16 {
    // XXX optimize?
    let result32 = (x as u32).wrapping_add(y as u32).wrapping_add(cf as u32);
    let result16 = result32 as u16;
    let mut f = F.get(z);

    assign_bit(&mut f, CF, (result32 >> 16) as u8, 0);

    // carry from bit 11 happened if:
    // x and y have same bit 12 AND result is set OR
    // x and y have different bit 12 AND result is clear
    assign_bit(&mut f, HF, ((x ^ y ^ result16) >> 12) as u8, 0);

    clear_bit(&mut f, NF);
    F.set(z, f);

    return result16;
}

pub fn add16<I: Io>(z: &mut Z80<I>, arg1: Reg16, arg2: Reg16) {
    let x = arg1.get(z);
    let y = arg2.get(z);
    let result = add16_impl(z, x, y, 0);
    arg1.set(z, result);
}

fn adc16_impl<I: Io>(z: &mut Z80<I>, x: u16, y: u16, cf: u16) -> u16 {
    let result = add16_impl(z, x, y, cf as u16);
    set_sign(z, (result >> 8) as u8);
    set_zero(z, (result as u8) | (result >> 8) as u8);

    let mut f = F.get(z);
    // overflow happened if:
    // x and y both have bit 15 AND result does not OR
    // x and y have clear bit 15 AND result is set
    assign_bit(&mut f, PF, ((!(x ^ y) & (x ^ result)) >> 15) as u8, 0);
    F.set(z, f);

    return result;
}

pub fn adc16<I: Io>(z: &mut Z80<I>, arg1: Reg16, arg2: Reg16) {
    let x = arg1.get(z);
    let y = arg2.get(z);
    let mut cf = 0u8;
    assign_bit(&mut cf, 0, F.get(z), CF);
    let result = adc16_impl(z, x, y, cf as u16);
    arg1.set(z, result);
}

fn sub16_impl<I: Io>(z: &mut Z80<I>, x: u16, y: u16, cf: u16) -> u16 {
    let result = add16_impl(z, x, !y, (1 ^ cf) as u16);
    let mut f = F.get(z);
    f ^= 1 << CF;
    f ^= 1 << HF;
    set_bit(&mut f, NF);
    F.set(z, f);
    return result;
}

pub fn sbc16<I: Io>(z: &mut Z80<I>, arg1: Reg16, arg2: Reg16) {
    let x = arg1.get(z);
    let y = arg2.get(z);
    let mut cf = 0u8;
    assign_bit(&mut cf, 0, F.get(z), CF);
    let result = adc16_impl(z, x, !y, (1 ^ cf) as u16);
    let mut f = F.get(z);
    f ^= 1 << CF;
    f ^= 1 << HF;
    set_bit(&mut f, NF);
    F.set(z, f);
    arg1.set(z, result);
}

pub fn inc16<I: Io>(z: &mut Z80<I>, arg: Reg16) {
    let val = arg.get(z);
    arg.set(z, val.wrapping_add(1));
}

pub fn dec16<I: Io>(z: &mut Z80<I>, arg: Reg16) {
    let val = arg.get(z);
    arg.set(z, val.wrapping_sub(1));
}

//// Rotate and Shift Group
///////////////////////////

/// Most of the functions in this group have similar addressing modes,
/// implementations, and flag behavior, so we write a macro to generate the
/// required functions in each case.
macro_rules! rotate_shift_functions_noa {
    ($fn_impl: ident $fn_impl2: ident
    $fn_general: ident $fn_store: ident) => {
        fn $fn_impl2<I: Io, T1: Settable<u8>>(z: &mut Z80<I>, arg: T1) {
            let a = arg.get(z);
            let result = $fn_impl(z, a);
            arg.set(z, result);
            let mut f = F.get(z);
            clear_bit(&mut f, HF);
            clear_bit(&mut f, NF);
            F.set(z, f);
        }

        pub fn $fn_general<I: Io, T1: Settable<u8>>(z: &mut Z80<I>, arg: T1) {
            $fn_impl2(z, arg);
            let result = arg.get(z);
            set_parity(z, result);
            set_sign(z, result);
            set_zero(z, result);
        }

        pub fn $fn_store<I: Io, T1: Settable<u8>>(z: &mut Z80<I>, arg: T1, store: Reg8) {
            $fn_general(z, arg);
            let result = arg.get(z);
            store.set(z, result);
        }
    }
}

macro_rules! rotate_shift_functions {
($fn_impl: ident $fn_impl2: ident $fn_general: ident
$fn_store: ident $fn_a: ident) => {
    pub fn $fn_a<I: Io>(z: &mut Z80<I>) {
        $fn_impl2(z, A);
    }
    rotate_shift_functions_noa!{$fn_impl $fn_impl2 $fn_general $fn_store}
    }
}

fn rlc_impl<I: Io>(z: &mut Z80<I>, x: u8) -> u8 {
    let mut f = F.get(z);
    assign_bit(&mut f, CF, x, 7);
    F.set(z, f);
    x.rotate_left(1)
}

rotate_shift_functions!{
    rlc_impl rlc_impl2
    rlc rlc_store rlca
}

fn rl_impl<I: Io>(z: &mut Z80<I>, x: u8) -> u8 {
    let mut f = F.get(z);
    let mut result = x << 1;
    assign_bit(&mut result, 0, f, CF);
    assign_bit(&mut f, CF, x, 7);
    F.set(z, f);
    result
}

rotate_shift_functions!{
    rl_impl rl_impl2
    rl rl_store rla
}

fn rrc_impl<I: Io>(z: &mut Z80<I>, x: u8) -> u8 {
    let mut f = F.get(z);
    assign_bit(&mut f, CF, x, 0);
    F.set(z, f);
    let result = x.rotate_right(1);
    result
}

rotate_shift_functions!{
    rrc_impl rrc_impl2
    rrc rrc_store rrca
}

fn rr_impl<I: Io>(z: &mut Z80<I>, x: u8) -> u8 {
    let mut result = x >> 1;
    let f0 = F.get(z);
    assign_bit(&mut result, 7, f0, CF);
    let mut f = F.get(z);
    assign_bit(&mut f, CF, x, 0);
    F.set(z, f);
    result
}

rotate_shift_functions!{
    rr_impl rr_impl2
    rr rr_store rra
}

fn sla_impl<I: Io>(z: &mut Z80<I>, x: u8) -> u8 {
    let mut f = F.get(z);
    let result = x << 1;
    assign_bit(&mut f, CF, x, 7);
    F.set(z, f);
    result
}

rotate_shift_functions_noa!{
    sla_impl sla_impl2
    sla sla_store
}

// SLL is undocumented; see Young
fn sll_impl<I: Io>(z: &mut Z80<I>, x: u8) -> u8 {
    let mut f = F.get(z);
    let mut result = x << 1;
    set_bit(&mut result, 0);
    assign_bit(&mut f, CF, x, 7);
    F.set(z, f);
    result
}

rotate_shift_functions_noa!{
    sll_impl sll_impl2
    sll sll_store
}

fn sra_impl<I: Io>(z: &mut Z80<I>, x: u8) -> u8 {
    let mut f = F.get(z);
    let result = ((x as i8) >> 1) as u8;
    assign_bit(&mut f, CF, x, 0);
    F.set(z, f);
    result
}

rotate_shift_functions_noa!{
    sra_impl sra_impl2
    sra sra_store
}

fn srl_impl<I: Io>(z: &mut Z80<I>, x: u8) -> u8 {
    let mut f = F.get(z);
    let result = x >> 1;
    assign_bit(&mut f, CF, x, 0);
    F.set(z, f);
    result
}

rotate_shift_functions_noa!{
    srl_impl srl_impl2
    srl srl_store
}

pub fn rld<I: Io>(z: &mut Z80<I>) {
    let hl: u8 = Address(HL).get(z);
    let hl_lo: u8 = 0xF & hl;
    let hl_hi: u8 = 0xF0 & hl;
    let a_lo = 0xF & A.get(z);
    let a_hi = 0xF0 & A.get(z);
    Address(HL).set(z, hl_lo << 4 | a_lo);
    A.set(z, hl_hi >> 4 | a_hi);
    let a = A.get(z);
    set_parity(z, a);
    set_sign(z, a);
    set_zero(z, a);
    let mut f = F.get(z);
    clear_bit(&mut f, HF);
    clear_bit(&mut f, NF);
    F.set(z, f);
}

pub fn rrd<I: Io>(z: &mut Z80<I>) {
    let hl: u8 = Address(HL).get(z);
    let hl_lo: u8 = 0xF & hl;
    let hl_hi: u8 = 0xF0 & hl;
    let a_lo = 0xF & A.get(z);
    let a_hi = 0xF0 & A.get(z);
    Address(HL).set(z, a_lo << 4 | hl_hi >> 4);
    A.set(z, hl_lo | a_hi);
    let a = A.get(z);
    set_parity(z, a);
    set_zero(z, a);
    set_sign(z, a);
    let mut f = F.get(z);
    clear_bit(&mut f, HF);
    clear_bit(&mut f, NF);
    F.set(z, f);
}

//// Bit Set, Reset, and Test Group
///////////////////////////////////

pub fn bit<I, T>(z: &mut Z80<I>, b: u8, arg: T)
where
    I: Io,
    T: Gettable<u8>,
{
    let x = arg.get(z);
    let mut f = F.get(z);
    assign_bit(&mut f, ZF, !x, b);
    assign_bit(&mut f, PF, !x, b);
    set_bit(&mut f, HF);
    clear_bit(&mut f, NF);
    let need_sf = b == 7 && (f & ZF) == 0;
    assign_bit(&mut f, SF, need_sf as u8, 0);
    F.set(z, f);
}

pub fn set<I, T>(z: &mut Z80<I>, b: u8, arg: T)
where
    I: Io,
    T: Settable<u8>,
{
    let mut x = arg.get(z);
    set_bit(&mut x, b);
    arg.set(z, x);
}

pub fn set_store<I, T>(z: &mut Z80<I>, b: u8, arg: T, reg: Reg8)
where
    I: Io,
    T: Settable<u8>,
{
    set(z, b, arg);
    let x = arg.get(z);
    reg.set(z, x);
}

pub fn res<I, T>(z: &mut Z80<I>, b: u8, arg: T)
where
    I: Io,
    T: Settable<u8>,
{
    let mut x = arg.get(z);
    clear_bit(&mut x, b);
    arg.set(z, x);
}

pub fn res_store<I, T>(z: &mut Z80<I>, b: u8, arg: T, reg: Reg8)
where
    I: Io,
    T: Settable<u8>,
{
    res(z, b, arg);
    let x = arg.get(z);
    reg.set(z, x);
}

//// Jump Group
///////////////

pub fn jp<I, T>(z: &mut Z80<I>, arg: T)
where
    I: Io,
    T: Gettable<u16>,
{
    let addr = arg.get(z);
    PC.set(z, addr);
}

pub fn jpcc<I: Io>(z: &mut Z80<I>, cc: ConditionCode, arg: u16) {
    if cc.get(z) {
        jp(z, arg);
    }
}

pub fn jr<I: Io>(z: &mut Z80<I>, e: i8) {
    let pc = PC.get(z);
    let new_pc = pc.wrapping_add(e as i16 as u16);
    PC.set(z, new_pc);
}

pub fn jrcc<I: Io>(z: &mut Z80<I>, cc: ConditionCode, e: i8) {
    if cc.get(z) {
        jr(z, e);
        z.cycles += 12;
    } else {
        z.cycles += 7;
    }
}

pub fn djnz<I: Io>(z: &mut Z80<I>, e: i8) {
    let b = B.get(z);
    let new_b = b.wrapping_sub(1);
    B.set(z, new_b);
    if new_b != 0 {
        jr(z, e);
        z.cycles += 13;
    } else {
        z.cycles += 8;
    }
}

//// Call and Return Group
//////////////////////////

pub fn call<I: Io>(z: &mut Z80<I>, nn: u16) {
    let pch = PCH.get(z);
    let pcl = PCL.get(z);
    let sp = SP.get(z);
    Address(sp.wrapping_sub(1)).set(z, pch);
    Address(sp.wrapping_sub(2)).set(z, pcl);
    SP.set(z, sp.wrapping_sub(2));
    PC.set(z, nn);
}

pub fn callcc<I: Io>(z: &mut Z80<I>, cc: ConditionCode, nn: u16) {
    if cc.get(z) {
        call(z, nn);
        z.cycles += 17;
    } else {
        z.cycles += 10;
    }
}

pub fn ret<I: Io>(z: &mut Z80<I>) {
    let sp = SP.get(z);
    let n1 = Address(sp).get(z);
    PCL.set(z, n1);
    let n2 = Address(sp.wrapping_add(1)).get(z);
    PCH.set(z, n2);
    SP.set(z, sp.wrapping_add(2));
}

pub fn retcc<I: Io>(z: &mut Z80<I>, cc: ConditionCode) {
    if cc.get(z) {
        ret(z);
        z.cycles += 11;
    } else {
        z.cycles += 5;
    }
}

pub fn reti<I: Io>(z: &mut Z80<I>) {
    retn(z);
}

pub fn retn<I: Io>(z: &mut Z80<I>) {
    let iff2 = z.iff2;
    z.iff1 = iff2 as u64;

    let sp = SP.get(z);
    let pcl = Address(sp).get(z);
    let pch = Address(sp.wrapping_add(1)).get(z);
    PCL.set(z, pcl);
    PCH.set(z, pch);
    SP.set(z, sp.wrapping_add(2));
}

//// Input and Output Group
///////////////////////////

pub fn in_n<I, T1, T2>(z: &mut Z80<I>, arg1: T1, arg2: T2)
where
    I: Io,
    T1: Settable<u8>,
    T2: Gettable<u8>,
{
    let address_lo = arg2.get(z);
    let address_hi = arg1.get(z);
    let address = to16(address_lo, address_hi);
    z.address = address;
    let x = z.io.input(address);
    z.data = x;
    arg1.set(z, x);
}

pub fn in_f<I, T1>(z: &mut Z80<I>, arg: T1) -> u8
where
    I: Io,
    T1: Gettable<u8>,
{
    let address_lo = arg.get(z);
    let address_hi = B.get(z);
    let address = to16(address_lo, address_hi);
    z.address = address;
    let x = z.io.input(address);
    z.data = x;
    set_parity(z, x);
    set_sign(z, x);
    set_zero(z, x);
    let mut f = F.get(z);
    clear_bit(&mut f, HF);
    clear_bit(&mut f, NF);
    F.set(z, f);
    x
}

pub fn in_c<I, T1, T2: Gettable<u8>>(z: &mut Z80<I>, arg1: T1, arg2: T2)
where
    I: Io,
    T1: Settable<u8>,
{
    let x = in_f(z, arg2);
    z.data = x;
    arg1.set(z, x);
}

/// The Z80 manual lists this instruction under IN r, (C) as "undefined"
/// It sets
pub fn in0<I: Io>(z: &mut Z80<I>) {
    let addr = BC.get(z);
    z.address = addr;
    let x = z.io.input(addr);
    z.data = x;
    set_parity(z, x);
    set_sign(z, x);
    set_zero(z, x);
    let mut f = F.get(z);
    clear_bit(&mut f, HF);
    clear_bit(&mut f, NF);
    F.set(z, f);
}

fn inid_impl<I: Io>(z: &mut Z80<I>, inc: u16) -> u8 {
    let b = B.get(z);
    let hl = HL.get(z);
    let addr = BC.get(z);
    z.address = addr;
    let x = z.io.input(addr);
    z.data = x;
    // XXX - the Z80 manual says HL is put on the address bus, but I am
    // skeptical about that
    // z.set_address_bus(hl);
    Address(hl).set(z, x);
    B.set(z, b.wrapping_sub(1));
    HL.set(z, hl.wrapping_add(inc));
    b.wrapping_sub(1)
}

pub fn ini<I: Io>(z: &mut Z80<I>) {
    let new_b = inid_impl(z, 1);
    set_zero(z, new_b);
    let mut f = F.get(z);
    set_bit(&mut f, NF);
    F.set(z, f);
}

pub fn inir<I: Io>(z: &mut Z80<I>) {
    while {
        z.cycles += 21;
        inid_impl(z, 1) != 0
    } {
        // r was already incremented twice by `run`
        inc_r(z);
        inc_r(z);
    }

    let mut f = F.get(z);
    set_bit(&mut f, ZF);
    set_bit(&mut f, NF);
    F.set(z, f);

    z.cycles += 16;
}

pub fn ind<I: Io>(z: &mut Z80<I>) {
    let new_b = inid_impl(z, 0xFFFF);
    set_zero(z, new_b);
    let mut f = F.get(z);
    set_bit(&mut f, NF);
    F.set(z, f);
}

pub fn indr<I: Io>(z: &mut Z80<I>) {
    while {
        z.cycles += 21;
        inid_impl(z, 0xFFFF) != 0
    } {
        // r was already incremented twice by `run`
        inc_r(z);
        inc_r(z);
    }

    let mut f = F.get(z);
    set_bit(&mut f, ZF);
    set_bit(&mut f, NF);
    F.set(z, f);

    z.cycles += 16;
}

pub fn out_n<I, T1, T2>(z: &mut Z80<I>, arg1: T1, arg2: T2)
where
    I: Io,
    T1: Gettable<u8>,
    T2: Gettable<u8>,
{
    let address_lo = arg1.get(z);
    let address_hi = A.get(z);
    let address = to16(address_lo, address_hi);
    let x = arg2.get(z);
    z.address = address;
    z.data = x;
    z.io.output(address, x);
}

pub fn out_c<I, T1, T2>(z: &mut Z80<I>, arg1: T1, arg2: T2)
where
    I: Io,
    T1: Gettable<u8>,
    T2: Gettable<u8>,
{
    let address_lo = arg1.get(z);
    let address_hi = B.get(z);
    let address = to16(address_lo, address_hi);
    let x = arg2.get(z);
    z.address = address;
    z.data = x;
    z.io.output(address, x);
}

fn outid_impl<I: Io>(z: &mut Z80<I>, inc: u16) {
    let b = B.get(z);
    let new_b = b.wrapping_sub(1);
    B.set(z, new_b);
    let addr = BC.get(z);
    z.address = addr;
    let hl = HL.get(z);
    let x = Address(hl).get(z);
    z.data = x;
    z.io.output(addr, x);
    HL.set(z, hl.wrapping_add(inc));
}

pub fn outi<I: Io>(z: &mut Z80<I>) {
    outid_impl(z, 1);
    let new_b = B.get(z);
    set_zero(z, new_b);
    let mut f = F.get(z);
    set_bit(&mut f, NF);
    F.set(z, f);
}

pub fn otir<I: Io>(z: &mut Z80<I>) {
    while {
        z.cycles += 21;
        outid_impl(z, 1);
        B.get(z) != 0
    } {
        // r was already incremented twice by `run`
        inc_r(z);
        inc_r(z);
    }

    let mut f = F.get(z);
    set_bit(&mut f, ZF);
    set_bit(&mut f, NF);
    F.set(z, f);

    z.cycles += 16;
}

pub fn outd<I: Io>(z: &mut Z80<I>) {
    outid_impl(z, 0xFFFF);
    let new_b = B.get(z);
    set_zero(z, new_b);
    let mut f = F.get(z);
    set_bit(&mut f, NF);
    F.set(z, f);
}

pub fn otdr<I: Io>(z: &mut Z80<I>) {
    while {
        z.cycles += 21;
        outid_impl(z, 0xFFFF);
        B.get(z) != 0
    } {
        // r was already incremented twice by `run`
        inc_r(z);
        inc_r(z);
    }

    let mut f = F.get(z);
    set_bit(&mut f, ZF);
    set_bit(&mut f, NF);

    z.cycles += 16;
}
