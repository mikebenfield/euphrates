use utilities;

use hardware::io16::Io16;
use hardware::memory16::Memory16;

use super::*;

use self::Reg16::*;
use self::Reg8::*;

pub fn outid_help<Z>(z: &mut Z, inc: u16)
where
    Z: Z80Internal + Memory16 + Io16 + ?Sized,
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

pub fn in_help<Z, T1>(z: &mut Z, arg: T1) -> u8
where
    Z: Z80Internal + Memory16 + Io16 + ?Sized,
    T1: Viewable<u8>,
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

pub fn inid_help<Z>(z: &mut Z, inc: u16) -> u8
where
    Z: Z80Internal + Memory16 + Io16 + ?Sized,
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

pub fn add16_help<Z>(z: &mut Z, x: u16, y: u16, cf: u16) -> u16
where
    Z: Z80Internal + ?Sized,
{
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

pub fn adc16_help<Z>(z: &mut Z, x: u16, y: u16, cf: u16) -> u16
where
    Z: Z80Internal + ?Sized,
{
    let result = add16_help(z, x, y, cf as u16);

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

pub fn sub_help<Z>(z: &mut Z, a: u8, x: u8, cf: u8) -> u8
where
    Z: Z80Internal + ?Sized,
{
    let result = add_help(z, a, !x, 1 ^ cf);
    let cf_set = z.is_set_flag(CF);
    let hf_set = z.is_set_flag(HF);
    z.set_flag_by(CF, !cf_set);
    z.set_flag_by(HF, !hf_set);
    z.set_flag(NF);
    result
}

pub fn add_help<Z>(z: &mut Z, a: u8, x: u8, cf: u8) -> u8
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

pub fn andor_help<Z>(z: &mut Z, result: u8)
where
    Z: Z80Internal + ?Sized,
{
    z.set_reg8(A, result);

    // note that for AND and OR, the manual says PF is set according to whether
    // there is overflow. That is surely a mistake.
    z.set_parity(result);
    z.set_sign(result);
    z.set_zero(result);
    z.clear_flag(HF | NF | CF);
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

pub fn ldid<Z>(z: &mut Z, inc: u16)
where
    Z: Z80Internal + Memory16 + ?Sized,
{
    let hl = HL.view(z);
    let de = DE.view(z);
    let bc = BC.view(z);

    let phl = z.read(hl);
    Address(de).change(z, phl);

    HL.change(z, hl.wrapping_add(inc));
    DE.change(z, de.wrapping_add(inc));
    BC.change(z, bc.wrapping_sub(1));

    z.clear_flag(HF | NF);
    z.set_flag_by(PF, bc != 1);
}

/// Most of the functions in the rotate and shift group have similar addressing modes,
/// implementations, and flag behavior, so we write a macro to generate the
/// required functions in each case.
macro_rules! rotate_shift_functions_noa_help {
    ($fn_help:ident $fn_help2:ident $fn_general:ident $fn_store:ident) => {
        fn $fn_help2<Z, T1>(z: &mut Z, arg: T1) -> u8
        where
            Z: Z80Internal + Memory16 + ?Sized,
            T1: Changeable<u8>,
        {
            let a = arg.view(z);
            let result = $fn_help(z, a);
            arg.change(z, result);
            z.clear_flag(HF | NF);
            result
        }

        pub fn $fn_general<Z, T1>(z: &mut Z, arg: T1)
        where
            Z: Z80Internal + Memory16 + ?Sized,
            T1: Changeable<u8>,
        {
            let result = $fn_help2(z, arg);
            z.set_parity(result);
            z.set_sign(result);
            z.set_zero(result);
        }

        pub fn $fn_store<Z, T1>(z: &mut Z, arg: T1, store: Reg8)
        where
            Z: Z80Internal + Memory16 + ?Sized,
            T1: Changeable<u8>,
        {
            let result = $fn_help2(z, arg);
            z.set_parity(result);
            z.set_sign(result);
            z.set_zero(result);
            store.change(z, result);
        }
    };
}

macro_rules! rotate_shift_functions_help {
    ($fn_help:ident $fn_help2:ident $fn_general:ident $fn_store:ident $fn_a:ident) => {
        pub fn $fn_a<Z>(z: &mut Z)
        where
            Z: Z80Internal + Memory16 + ?Sized,
        {
            $fn_help2(z, A);
        }
        rotate_shift_functions_noa_help!{$fn_help $fn_help2 $fn_general $fn_store}
    };
}

fn rlc_help<Z>(z: &mut Z, x: u8) -> u8
where
    Z: Z80Internal + ?Sized,
{
    z.set_flag_by(CF, x & 0x80 != 0);
    x.rotate_left(1)
}

rotate_shift_functions_help!{
    rlc_help rlc_help2 rlc rlc_store rlca
}

fn rl_help<Z>(z: &mut Z, x: u8) -> u8
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

rotate_shift_functions_help!{
    rl_help rl_help2 rl rl_store rla
}

fn rrc_help<Z>(z: &mut Z, x: u8) -> u8
where
    Z: Z80Internal + ?Sized,
{
    z.set_flag_by(CF, x & 1 != 0);
    x.rotate_right(1)
}

rotate_shift_functions_help!{
    rrc_help rrc_help2 rrc rrc_store rrca
}

fn rr_help<Z>(z: &mut Z, x: u8) -> u8
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

rotate_shift_functions_help!{
    rr_help rr_help2 rr rr_store rra
}

fn sla_help<Z>(z: &mut Z, x: u8) -> u8
where
    Z: Z80Internal + ?Sized,
{
    z.set_flag_by(CF, x & 0x80 != 0);
    x << 1
}

rotate_shift_functions_noa_help!{
    sla_help sla_help2 sla sla_store
}

// SLL is undocumented; see Young
fn sll_help<Z>(z: &mut Z, x: u8) -> u8
where
    Z: Z80Internal + ?Sized,
{
    z.set_flag_by(CF, x & 0x80 != 0);
    let mut result = x << 1;
    result |= 1;
    result
}

rotate_shift_functions_noa_help!{
    sll_help sll_help2 sll sll_store
}

fn sra_help<Z>(z: &mut Z, x: u8) -> u8
where
    Z: Z80Internal + ?Sized,
{
    z.set_flag_by(CF, x & 1 != 0);
    let result = ((x as i8) >> 1) as u8;
    result
}

rotate_shift_functions_noa_help!{
    sra_help sra_help2 sra sra_store
}

fn srl_help<Z>(z: &mut Z, x: u8) -> u8
where
    Z: Z80Internal + ?Sized,
{
    z.set_flag_by(CF, x & 1 != 0);
    x >> 1
}

rotate_shift_functions_noa_help!{
    srl_help srl_help2 srl srl_store
}
