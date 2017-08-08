#![allow(dead_code)]

use bits::*;
use hardware::z80::types::*;

fn set_parity(flags: &mut u8, x: u8) {
    let mut tmp = x;
    tmp ^= tmp >> 4;
    tmp ^= tmp >> 2;
    tmp ^= tmp >> 1;
    let parity: u8 = (!tmp) & 1;
    assign_bit(flags, PF, parity, 0);
}

fn set_sign(flags: &mut u8, x: u8) {
    assign_bit(flags, SF, x, 7);
}

fn set_zero(flags: &mut u8, x: u8) {
    let z = (x == 0) as u8;
    assign_bit(flags, ZF, z, 0);
}

//// Interrupts
///////////////

pub fn rst<Z: Z80>(z: &mut Z, p: u8) {
    let sp = SP.get(z);
    let pch = PCH.get(z);
    let pcl = PCL.get(z);
    Address(sp.wrapping_sub(1)).set(z, pch);
    Address(sp.wrapping_sub(2)).set(z, pcl);
    SP.set(z, sp.wrapping_sub(2));
    PCH.set(z, 0);
    PCL.set(z, p);
}

pub fn nonmaskable_interrupt<Z: Z80>(z: &mut Z) {
    let iff1 = z.get_z80_hardware().iff1;
    z.get_mut_z80_hardware().iff2 = !iff1;
    rst(z, 0x66);
}

pub fn maskable_interrupt<Z: Z80>(z: &mut Z) -> bool {
    let iff1 = z.get_z80_hardware().iff1;
    z.get_mut_z80_hardware().iff1 = false;
    z.get_mut_z80_hardware().iff2 = false;

    if iff1 {
        log_minor!(z, "Z80: Maskable interrupt allowed");

        let im = z.get_z80_hardware().interrupt_mode;
        match im {
            Im1 => rst(z, 0x38),
            _ => unimplemented!(),
        }
    } else {
        log_minor!(z, "Z80: Maskable interrupt denied");
    }
    iff1
}

//// 8-Bit Load Group
/////////////////////

pub fn ld<Z: Z80, T1: Settable<u8>, T2: Gettable<u8>>(
    z: &mut Z, arg1: T1, arg2: T2
) {
    let val = arg2.get(z);
    arg1.set(z, val);
}

// XXX text about interrupts in manual
pub fn ld8_ir<Z: Z80>(z: &mut Z, arg: Reg8) {
    let val = arg.get(z);
    let mut f = F.get(z);
    set_zero(&mut f, val);
    set_sign(&mut f, val);
    clear_bit(&mut f, HF);
    assign_bit(&mut f, PF, z.get_z80_hardware().iff2 as u8, 0);
    clear_bit(&mut f, NF);
    F.set(z, f);
}

//// 16-Bit Load Group
//////////////////////

pub fn ld16<Z: Z80, T1: Settable<u16>, T2: Gettable<u16>>(
    z: &mut Z, arg1: T1, arg2: T2
) {
    let val = arg2.get(z);
    arg1.set(z, val);
}

pub fn push<Z: Z80>(z: &mut Z, reg: Reg16) {
    let (lo, hi) = to8(reg.get(z));
    let sp = SP.get(z);
    Address(sp.wrapping_sub(1)).set(z, hi);
    Address(sp.wrapping_sub(2)).set(z, lo);
    SP.set(z, sp.wrapping_sub(2));
}

pub fn pop<Z: Z80>(z: &mut Z, reg: Reg16) {
    let sp = SP.get(z);
    let lo = Address(sp).get(z);
    let hi = Address(sp.wrapping_add(1)).get(z);
    // println!("popping {:0>4X} {:0>2$X} {:0>2$X}", sp, lo, hi);
    reg.set(z, to16(lo, hi));
    SP.set(z, sp.wrapping_add(2));
}

//// Exchange, Block Transfer, and Search Group
///////////////////////////////////////////////

pub fn ex<Z: Z80, T1: Settable<u16>>(
   z: &mut Z, reg1: T1, reg2: Reg16
) {
    let val1 = reg1.get(z);
    let val2 = reg2.get(z);
    reg1.set(z, val2);
    reg2.set(z, val1);
}

pub fn exx<Z: Z80>(z: &mut Z) {
    for &(reg1, reg2) in [(BC, BC0), (DE, DE0), (HL, HL0)].iter() {
        let val1 = reg1.get(z);
        let val2 = reg2.get(z);
        reg1.set(z, val2);
        reg2.set(z, val1);
    }
}

fn ld_id_impl<Z: Z80>(z: &mut Z, inc: i8) {
    let val_hl: u8 = Address(HL).get(z);
    Address(DE).set(z, val_hl);
    let hl = HL.get(z);
    HL.set(z, hl.wrapping_add(inc as i16 as u16));
    let de = DE.get(z);
    DE.set(z, de.wrapping_add(inc as i16 as u16));
    let bc = BC.get(z);
    BC.set(z, bc.wrapping_sub(1));
}

fn ld_id_flag_impl<Z: Z80>(z: &mut Z) {
    let mut f = F.get(z);
    clear_bit(&mut f, HF);
    clear_bit(&mut f, NF);
    let b = (BC.get(z) != 0) as u8;
    assign_bit(&mut f, PF, b, 0);
    F.set(z, f);
}

// these ldi and ldd instructions affect the XF and YF registers
// in ways I have not attempted to emulate
pub fn ldi<Z: Z80>(z: &mut Z) {
    ld_id_impl(z, 1);
    ld_id_flag_impl(z);
}

pub fn ldd<Z: Z80>(z: &mut Z) {
    ld_id_impl(z, -1);
    ld_id_flag_impl(z);
}

pub fn ldir<Z: Z80>(z: &mut Z) {
    // XXX check interrupts
    while {
        // goofy hack to get a do-while loop
        ld_id_impl(z, 1);
        z.advance_t_states(21);
        BC.get(z) != 0
    }
    {}

    ld_id_flag_impl(z);

    z.advance_t_states(17);
}

pub fn lddr<Z: Z80>(z: &mut Z) {
    while {
        ld_id_impl(z, -1);
        z.advance_t_states(21);
        BC.get(z) != 0
    }
    {}

    ld_id_flag_impl(z);

    z.advance_t_states(17);
}

fn cpi_impl<Z: Z80>(z: &mut Z) {
    cp_impl(z, Address(HL));
    let bc = BC.get(z);
    BC.set(z, bc.wrapping_sub(1));
    let hl = HL.get(z);
    HL.set(z, hl.wrapping_add(1));
    let mut f = F.get(z);
    assign_bit(&mut f, PF, (bc != 1) as u8, 0);
    F.set(z, f);
}

pub fn cpi<Z: Z80>(z80: &mut Z) {
    cpi_impl(z80);
}

pub fn cpir<Z: Z80>(z: &mut Z) {
    while {
        cpi_impl(z);
        z.advance_t_states(21);
        BC.get(z) != 0 && F.get(z) & (1 << ZF) != 0
    }
    {}

    z.advance_t_states(17);
}

pub fn cpd<Z: Z80>(z: &mut Z) {
    cp_impl(z, Address(HL));
    let bc = BC.get(z);
    BC.set(z, bc.wrapping_sub(1));
    let hl = HL.get(z);
    HL.set(z, hl.wrapping_sub(2));
    let mut f = F.get(z);
    assign_bit(&mut f, PF, (bc != 1) as u8, 0);
    F.set(z, f);
}

pub fn cpdr<Z: Z80>(z: &mut Z) {
    while {
        inc_r(z);
        inc_r(z);
        cpd(z);
        z.advance_t_states(21);
        BC.get(z) !=0 && F.get(z) & (1 << ZF) != 0
    }
    {}

    z.advance_t_states(17);
}

//// 8-Bit Arithmetic Group
///////////////////////////

fn add_impl<Z: Z80>(z: &mut Z, a: u8, x: u8, cf: u8) -> u8 {
    // XXX optimize?
    let result16 = (x as u16).wrapping_add(a as u16).wrapping_add(cf as u16);
    let result8 = result16 as u8;
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

    assign_bit(&mut f, SF, result8, 7);
    assign_bit(&mut f, ZF, (result8 == 0) as u8, 0);

    clear_bit(&mut f, NF);
    F.set(z, f);

    return result8;
}

pub fn add<Z: Z80, T1: Settable<u8>, T2: Gettable<u8>>(
  z: &mut Z,
  arg1: T1,
  arg2: T2,
) {
    let a = arg1.get(z);
    let b = arg2.get(z);
    let result = add_impl(z, a, b, 0);
    arg1.set(z, result);
}

pub fn adc<Z: Z80, T1: Settable<u8>, T2: Gettable<u8>>(
  z: &mut Z,
  arg1: T1,
  arg2: T2,
) {
    let mut cf = 0u8;
    let f = F.get(z);
    assign_bit(&mut cf, 0, f, CF);
    let a = arg1.get(z);
    let x = arg2.get(z);
    let result = add_impl(z, a, x, cf);
    arg1.set(z, result);
}

fn sub_impl<Z: Z80>(z: &mut Z, a: u8, x: u8, cf: u8) -> u8 {
    // XXX check that flags are set correctly
    let result = add_impl(z, a, !x, 1 ^ cf);
    let mut f = F.get(z);
    f ^= 1 << CF;
    f ^= 1 << HF;
    set_bit(&mut f, NF);
    F.set(z, f);
    result
}

pub fn sub<Z: Z80, T1: Settable<u8>, T2: Gettable<u8>>(
  z: &mut Z,
  arg1: T1,
  arg2: T2,
) {
    let a = arg1.get(z);
    let x = arg2.get(z);
    let result = sub_impl(z, a, x, 0);
    arg1.set(z, result);
}

pub fn sbc<Z: Z80, T1: Settable<u8>, T2: Gettable<u8>>(
  z: &mut Z,
  arg1: T1,
  arg2: T2,
) {
    let mut cf = 0u8;
    let f = F.get(z);
    assign_bit(&mut cf, 0, f, CF);
    let a = arg1.get(z);
    let x = arg2.get(z);
    let result = sub_impl(z, a, x, cf);
    arg1.set(z, result);
}

fn andor_impl<Z: Z80>(z: &mut Z, result: u8) {
    A.set(z, result);

    let mut f = F.get(z);

    assign_bit(&mut f, XF, result, XF);
    assign_bit(&mut f, YF, result, YF);
    assign_bit(&mut f, SF, result, SF);

    let zero: u8 = (result == 0) as u8;
    assign_bit(&mut f, ZF, zero, 0);

    // note that for AND and OR, the manual says PF is set according to whether
    // there is overflow. I'm betting that is a mistake.
    set_parity(&mut f, result);
    set_bit(&mut f, HF);
    clear_bit(&mut f, NF);
    clear_bit(&mut f, CF);

    F.set(z, f);
}

pub fn and<Z: Z80, T1: Gettable<u8>>(z: &mut Z, arg: T1) {
    let result = arg.get(z) & A.get(z);
    andor_impl(z, result);
}

pub fn or<Z: Z80, T1: Gettable<u8>>(z: &mut Z, arg: T1) {
    let result = arg.get(z) | A.get(z);
    andor_impl(z, result);
}

pub fn xor<Z: Z80, T1: Gettable<u8>>(z: &mut Z, arg: T1) {
    let result = arg.get(z) ^ A.get(z);
    andor_impl(z, result);
}

fn cp_impl<Z: Z80, T1: Gettable<u8>>(z: &mut Z, arg: T1) {
    let x = arg.get(z);
    let a = A.get(z);
    sub_impl(z, a, x, 0);
    A.set(z, a);
}

pub fn cp<Z: Z80, T1: Gettable<u8>>(z: &mut Z, arg: T1) {
    cp_impl(z, arg);
}

pub fn inc<Z: Z80, T1: Settable<u8>>(z: &mut Z, arg: T1) {
    let x = arg.get(z);
    let result = x.wrapping_add(1);
    arg.set(z, result);
    let mut f = F.get(z);
    assign_bit(&mut f, SF, result, SF);
    assign_bit(&mut f, XF, result, XF);
    assign_bit(&mut f, YF, result, YF);
    set_zero(&mut f, result);
    assign_bit(&mut f, HF, ((x & 0b111) == 0b111) as u8, 0);
    assign_bit(&mut f, PF, (x == 0x7F) as u8, 0);
    clear_bit(&mut f, NF);
    F.set(z, f);
}

pub fn dec<Z: Z80, T1: Settable<u8>>(z: &mut Z, arg: T1) {
    let x = arg.get(z);
    let result = x.wrapping_sub(1);
    arg.set(z, result);
    let mut f = F.get(z);
    assign_bit(&mut f, SF, result, SF);
    assign_bit(&mut f, XF, result, XF);
    assign_bit(&mut f, YF, result, YF);
    set_zero(&mut f, result);
    assign_bit(&mut f, HF, ((x & 0b111) == 0) as u8, 0);
    assign_bit(&mut f, PF, (x == 0x80) as u8, 0);
    set_bit(&mut f, NF);
    F.set(z, f);
}

//// General-Purpose Arithmetic and CPU Control Groups
//////////////////////////////////////////////////////

pub fn daa<Z: Z80>(z: &mut Z) {
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

    A.set(z, a - diff);
    let mut f = F.get(z);
    assign_bit(&mut f, CF, new_cf, 0);
    assign_bit(&mut f, HF, new_hf, 0);
    assign_bit(&mut f, SF, A.get(z), 7);
    set_zero(&mut f, A.get(z));
    set_parity(&mut f, A.get(z));
    F.set(z, f);
}

pub fn cpl<Z: Z80>(z: &mut Z) {
    let a = A.get(z);
    A.set(z, !a);
    let mut f = F.get(z);
    set_bit(&mut f, HF);
    set_bit(&mut f, NF);
    F.set(z, f);
}

pub fn neg<Z: Z80>(z: &mut Z) {
    // subtracts A from 0
    let a = A.get(z);
    let result = sub_impl(z, 0, a, 0);
    A.set(z, result);
}

pub fn ccf<Z: Z80>(z: &mut Z) {
    let mut f = F.get(z);
    f ^= 1 << CF;
    clear_bit(&mut f, NF);
    F.set(z, f);
}

pub fn scf<Z: Z80>(z: &mut Z) {
    let mut f = F.get(z);
    clear_bit(&mut f, HF);
    clear_bit(&mut f, NF);
    set_bit(&mut f, CF);
    F.set(z, f);
}

pub fn nop<Z: Z80>(z: &mut Z) {
}

pub fn halt<Z: Z80>(z: &mut Z) {
}

pub fn di<Z: Z80>(z: &mut Z) {
    z.get_mut_z80_hardware().iff1 = false;
    z.get_mut_z80_hardware().iff2 = false;
}

pub fn ei<Z: Z80>(z: &mut Z) {
    use super::execute::execute_loop;

    // Interrupts are not actually enabled until after the following instruction
    let t_states = z.get_t_states();
    execute_loop(z, 1 + t_states);

    z.get_mut_z80_hardware().iff1 = true;
    z.get_mut_z80_hardware().iff2 = true;
}

pub fn im<Z: Z80>(z: &mut Z, m: u8) {
    match m {
        0 => z.get_mut_z80_hardware().interrupt_mode = Im0,
        1 => z.get_mut_z80_hardware().interrupt_mode = Im1,
        2 => z.get_mut_z80_hardware().interrupt_mode = Im2,
        _ => panic!("Z80: Invalid interrupt mode"),
    }
}

pub fn im1<Z: Z80>(z: &mut Z) {
    z.get_mut_z80_hardware().interrupt_mode = Im1;
}

pub fn im2<Z: Z80>(z: &mut Z) {
    z.get_mut_z80_hardware().interrupt_mode = Im2;
}

//// 16-Bit Arithmetic Group
////////////////////////////

fn add16_impl<Z: Z80>(z: &mut Z, x: u16, y: u16, cf: u16) -> u16 {
    // XXX optimize?
    let result32 = (x as u32).wrapping_add(y as u32).wrapping_add(cf as u32);
    let result16 = result32 as u16;
    let mut f = F.get(z);

    assign_bit(&mut f, CF, (result32 >> 16) as u8, 0);

    // carry from bit 11 happened if:
    // x and y have same bit 11 AND result is set OR
    // x and y have different bit 11 AND result is clear
    assign_bit(&mut f, HF, ((x ^ y ^ result16) >> 11) as u8, 0);

    // overflow happened if:
    // x and y both have bit 15 AND result does not OR
    // x and y have clear bit 15 AND result is set
    assign_bit(&mut f, PF, ((!(x ^ y) & (x ^ result16)) >> 15) as u8, 0);

    assign_bit(&mut f, SF, (result16 >> 15) as u8, 0);
    assign_bit(&mut f, ZF, (result16 == 0) as u8, 0);

    clear_bit(&mut f, NF);
    F.set(z, f);

    return result16;
}

pub fn add16<Z: Z80>(z: &mut Z, arg1: Reg16, arg2: Reg16) {
    let x = arg1.get(z);
    let y = arg2.get(z);
    let result = add16_impl(z, x, y, 0);
    arg1.set(z, result);
}

pub fn adc16<Z: Z80>(z: &mut Z, arg1: Reg16, arg2: Reg16) {
    let x = arg1.get(z);
    let y = arg2.get(z);
    let mut cf = 0u8;
    assign_bit(&mut cf, 0, F.get(z), CF);
    let result = add16_impl(z, x, y, cf as u16);
    arg1.set(z, result);
}

fn sub16_impl<Z: Z80>(z: &mut Z, x: u16, y: u16, cf: u16) -> u16 {
    // XXX check that flags are set correctly
    let result = add16_impl(z, x, !y, 1 ^ cf);
    let mut f = F.get(z);
    f ^= 1 << CF;
    f ^= 1 << HF;
    set_bit(&mut f, NF);
    F.set(z, f);
    result
}

pub fn sbc16<Z: Z80>(z: &mut Z, arg1: Reg16, arg2: Reg16) {
    let x = arg1.get(z);
    let y = arg2.get(z);
    let mut cf = 0u8;
    assign_bit(&mut cf, 0, F.get(z), CF);
    let result = sub16_impl(z, x, y, cf as u16);
    arg1.set(z, result);
}

pub fn inc16<Z: Z80>(z: &mut Z, arg: Reg16) {
    let val = arg.get(z);
    arg.set(z, val.wrapping_add(1));
}

pub fn dec16<Z: Z80>(z: &mut Z, arg: Reg16) {
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
    $fn_general: ident $string_general: expr,
    $fn_store: ident $string_store: expr) => {
        fn $fn_impl2<Z: Z80, T1: Settable<u8>>(z: &mut Z, arg: T1) {
            let a = arg.get(z);
            let result = $fn_impl(z, a);
            arg.set(z, result);
            let mut f = F.get(z);
            set_parity(&mut f, result);
            set_zero(&mut f, result);
            set_sign(&mut f, result);
            clear_bit(&mut f, HF);
            clear_bit(&mut f, NF);
            F.set(z, f);
        }

        pub fn $fn_general<Z: Z80, T1: Settable<u8>>(z: &mut Z, arg: T1) {
            $fn_impl2(z, arg);
        }

        pub fn $fn_store<Z: Z80, T1: Settable<u8>>(z: &mut Z, arg: T1, store: Reg8) {
            $fn_impl2(z, arg);
            let result = arg.get(z);
            store.set(z, result);
        }
    }
}

macro_rules! rotate_shift_functions {
($fn_impl: ident $fn_impl2: ident $fn_general: ident $string_general: expr,
$fn_store: ident $string_store: expr, $fn_a: ident $string_a: expr) => {
    pub fn $fn_a<Z: Z80>(z: &mut Z) {
        let a = A.get(z);
        let result = $fn_impl(z, a);
        A.set(z, result);
    }
    rotate_shift_functions_noa!{$fn_impl $fn_impl2 $fn_general
        $string_general, $fn_store $string_store}
    }
}

fn rlc_impl<Z: Z80>(z: &mut Z, x: u8) -> u8 {
    let mut f = F.get(z);
    assign_bit(&mut f, CF, x, 7);
    F.set(z, f);
    x.rotate_left(1)
}

rotate_shift_functions!{
    rlc_impl rlc_impl2
    rlc "Z80: op: RLC {:?}",
    rlc_store "Z80: op: RLC {:?}, {:?}",
    rlca "Z80: op: RLCA"
}

fn rl_impl<Z: Z80>(z: &mut Z, x: u8) -> u8 {
    let mut f = F.get(z);
    let mut result = x << 1;
    assign_bit(&mut result, 0, f, CF);
    assign_bit(&mut f, CF, x, 7);
    result
}

rotate_shift_functions!{rl_impl rl_impl2
    rl "Z80: op: RL {:?}",
    rl_store "Z80: op: RL {:?}, {:?}",
    rla "Z80: op: RLA"
}

fn rrc_impl<Z: Z80>(z: &mut Z, x: u8) -> u8 {
    let mut f = F.get(z);
    assign_bit(&mut f, CF, x, 0);
    F.set(z, f);
    let result = x.rotate_right(1);
    result
}

rotate_shift_functions!{
    rrc_impl rrc_impl2
    rrc "Z80: op: RRC {:?}",
    rrc_store "Z80: op: RRC {:?}, {:?}",
    rrca "Z80: op: RRCA"
}

fn rr_impl<Z: Z80>(z: &mut Z, x: u8) -> u8 {
    let mut f = F.get(z);
    let mut result = x >> 1;
    assign_bit(&mut result, 7, f, CF);
    assign_bit(&mut f, CF, x, 0);
    F.set(z, f);
    result
}

rotate_shift_functions!{
    rr_impl rr_impl2
    rr "Z80: op: RR {:?}",
    rr_store "Z80: op: RR {:?}, {:?}",
    rra "Z80: op: RRA"
}

fn sla_impl<Z: Z80>(z: &mut Z, x: u8) -> u8 {
    let mut f = F.get(z);
    let result = x << 1;
    assign_bit(&mut f, CF, x, 7);
    F.set(z, f);
    result
}

rotate_shift_functions_noa!{
    sla_impl sla_impl2
    sla "Z80: op: SLA {:?}",
    sla_store "Z80: op: SLA {:?}, {:?}"
}

// SLL is undocumented; see Young
fn sll_impl<Z: Z80>(z: &mut Z, x: u8) -> u8 {
    let mut f = F.get(z);
    let mut result = x << 1;
    set_bit(&mut result, 0);
    assign_bit(&mut f, CF, x, 7);
    F.set(z, f);
    result
}

rotate_shift_functions_noa!{
    sll_impl sll_impl2
    sll "Z80: op: SLL {:?}",
    sll_store "Z80: op: SLL {:?}, {:?}"
}

fn sra_impl<Z: Z80>(z: &mut Z, x: u8) -> u8 {
    let mut f = F.get(z);
    let result = ((x as i8) >> 1) as u8;
    assign_bit(&mut f, CF, x, 0);
    F.set(z, f);
    result
}

rotate_shift_functions_noa!{
    sra_impl sra_impl2
    sra "Z80: op: SRA {:?}",
    sra_store "Z80: op: SRA {:?}, {:?}"
}

fn srl_impl<Z: Z80>(z: &mut Z, x: u8) -> u8 {
    let mut f = F.get(z);
    let result = x >> 1;
    assign_bit(&mut f, CF, x, 0);
    F.set(z, f);
    result
}

rotate_shift_functions_noa!{
    srl_impl srl_impl2
    srl "Z80: op: SRL {:?}",
    srl_store "Z80: op: SRL {:?}, {:?}"
}

pub fn rld<Z: Z80>(z: &mut Z) {
    let hl: u8 = Address(HL).get(z);
    let hl_lo: u8 = 0xF & hl;
    let hl_hi: u8 = 0xF0 & hl;
    let a_lo = 0xF & A.get(z);
    let a_hi = 0xF0 & A.get(z);
    Address(HL).set(z, hl_lo << 4 | a_lo);
    A.set(z, hl_hi >> 4 | a_hi);
    let a = A.get(z);
    let mut f = F.get(z);
    assign_bit(&mut f, SF, a, 7);
    set_zero(&mut f, a);
    set_parity(&mut f, a);
    set_sign(&mut f, a);
    clear_bit(&mut f, HF);
    clear_bit(&mut f, NF);
    F.set(z, f);
}

pub fn rrd<Z: Z80>(z: &mut Z) {
    let hl: u8 = Address(HL).get(z);
    let hl_lo: u8 = 0xF & hl;
    let hl_hi: u8 = 0xF0 & hl;
    let a_lo = 0xF & A.get(z);
    let a_hi = 0xF0 & A.get(z);
    Address(HL).set(z, a_lo | hl_hi >> 4);
    A.set(z, hl_lo | a_hi);
    let a = A.get(z);
    let mut f = F.get(z);
    assign_bit(&mut f, SF, a, 7);
    set_zero(&mut f, a);
    set_parity(&mut f, a);
    set_sign(&mut f, a);
    clear_bit(&mut f, HF);
    clear_bit(&mut f, NF);
    F.set(z, f);
}

//// Bit Set, Reset, and Test Group
///////////////////////////////////

pub fn bit<Z: Z80, T: Gettable<u8>>(z: &mut Z, b: u8, arg: T) {
    let x = arg.get(z);
    let mut f = F.get(z);
    assign_bit(&mut f, ZF, !x, b);
    set_bit(&mut f, HF);
    clear_bit(&mut f, NF);
    F.set(z, f);
}

fn set_impl<Z: Z80, T: Settable<u8>>(z: &mut Z, b: u8, arg: T) {
    let mut x = arg.get(z);
    set_bit(&mut x, b);
    arg.set(z, x);
}

pub fn set<Z: Z80, T: Settable<u8>>(z: &mut Z, b: u8, arg: T) {
    set_impl(z, b, arg);
}

pub fn set_store<Z: Z80, T: Settable<u8>>(z: &mut Z, b: u8, arg: T, reg: Reg8) {
    set_impl(z, b, arg);
    let x = arg.get(z);
    reg.set(z, x);
}

fn res_impl<Z: Z80, T: Settable<u8>>(z: &mut Z, b: u8, arg: T) {
    let mut x = arg.get(z);
    clear_bit(&mut x, b);
    arg.set(z, x);
}

pub fn res<Z: Z80, T: Settable<u8>>(z: &mut Z, b: u8, arg: T) {
    res_impl(z, b, arg);
}

pub fn res_store<Z: Z80, T: Settable<u8>>(z: &mut Z, b: u8, arg: T, reg: Reg8) {
    res_impl(z, b, arg);
    let x = arg.get(z);
    reg.set(z, x);
}

//// Jump Group
///////////////

pub fn jp<Z: Z80, T: Gettable<u16>>(z: &mut Z, arg: T) {
    let addr = arg.get(z);
    PC.set(z, addr);
}

pub fn jpcc<Z: Z80>(z: &mut Z, cc: ConditionCode, arg: u16) {
    if cc.get(z) {
        jp(z, arg);
    }
}

pub fn jr<Z: Z80>(z: &mut Z, e: i8) {
    let pc = PC.get(z);
    let new_pc = pc.wrapping_add(e as i16 as u16);
    PC.set(z, new_pc);
}

pub fn jrcc<Z: Z80>(z: &mut Z, cc: ConditionCode, e: i8) {
    if cc.get(z) {
        jr(z, e);
        z.advance_t_states(12);
    } else {
        z.advance_t_states(7);
    }
}

pub fn djnz<Z: Z80>(z: &mut Z, e: i8) {
    let b = B.get(z);
    let new_b = b.wrapping_sub(1);
    B.set(z, new_b);
    if new_b != 0 {
        jr(z, e);
        z.advance_t_states(13);
    } else {
        z.advance_t_states(8);
    }
}

//// Call and Return Group
//////////////////////////

pub fn call<Z: Z80>(z: &mut Z, nn: u16) {
    let pch = PCH.get(z);
    let pcl = PCL.get(z);
    let sp = SP.get(z);
    Address(sp.wrapping_sub(1)).set(z, pch);
    Address(sp.wrapping_sub(2)).set(z, pcl);
    SP.set(z, sp.wrapping_sub(2));
    PC.set(z, nn);
}

pub fn callcc<Z: Z80>(z: &mut Z, cc: ConditionCode, nn: u16) {
    if cc.get(z) {
        call(z, nn);
        z.advance_t_states(17);
    } else {
        z.advance_t_states(10);
    }
}

pub fn ret<Z: Z80>(z: &mut Z) {
    let sp = SP.get(z);
    let n1 = Address(sp).get(z);
    PCL.set(z, n1);
    let n2 = Address(sp.wrapping_add(1)).get(z);
    PCH.set(z, n2);
    SP.set(z, sp.wrapping_add(2));
}

pub fn retcc<Z: Z80>(z: &mut Z, cc: ConditionCode) {
    if cc.get(z) {
        ret(z);
        z.advance_t_states(11);
    } else {
        z.advance_t_states(5);
    }
}

pub fn reti<Z: Z80>(z: &mut Z) {
    // XXX implement

    unimplemented!();
}

pub fn retn<Z: Z80>(z: &mut Z) {
    let iff2 = z.get_z80_hardware().iff2;
    z.get_mut_z80_hardware().iff1 = iff2;

    let sp = SP.get(z);
    let pcl = Address(sp).get(z);
    let pch = Address(sp.wrapping_add(1)).get(z);
    PCL.set(z, pcl);
    PCH.set(z, pch);
    SP.set(z, sp.wrapping_add(2));
}

//// Input and Output Group
///////////////////////////

pub fn in_n<Z: Z80, T1: Settable<u8>, T2: Gettable<u8>>(
    z: &mut Z,
    arg1: T1,
    arg2: T2,
) {
    let address_lo = arg2.get(z);
    let address_hi = arg1.get(z);
    let address = to16(address_lo, address_hi);
    z.get_mut_z80_hardware().address = address;
    let x = z.input(address);
    z.get_mut_z80_hardware().data = x;
    arg1.set(z, x);
}

pub fn in_f<Z: Z80, T1: Gettable<u8>>(z: &mut Z, arg: T1) -> u8 {
    let address_lo = arg.get(z);
    let address_hi = B.get(z);
    let address = to16(address_lo, address_hi);
    z.get_mut_z80_hardware().address = address;
    let x = z.input(address);
    z.get_mut_z80_hardware().data = x;
    let mut f = F.get(z);
    set_sign(&mut f, x);
    set_zero(&mut f, x);
    set_parity(&mut f, x);
    clear_bit(&mut f, HF);
    clear_bit(&mut f, NF);
    F.set(z, f);
    x
}

pub fn in_c<Z: Z80, T1: Settable<u8>, T2: Gettable<u8>>(
    z: &mut Z,
    arg1: T1,
    arg2: T2
) {
    let x = in_f(z, arg2);
    arg1.set(z, x);
}

/// The Z80 manual lists this instruction under IN r, (C) as "undefined"
/// It sets
pub fn in0<Z: Z80>(z: &mut Z) {
    log_minor!(z, "Z80: op: {:?}", "IN flag, (C)");

    let addr = BC.get(z);
    z.get_mut_z80_hardware().address = addr;
    let x = z.input(addr);
    z.get_mut_z80_hardware().data = x;
    let mut f = F.get(z);
    set_sign(&mut f, x);
    set_zero(&mut f, x);
    set_parity(&mut f, x);
    clear_bit(&mut f, HF);
    clear_bit(&mut f, NF);
    F.set(z, f);
}

fn inid_impl<Z: Z80>(z: &mut Z, inc: u16) -> u8 {
    let b = B.get(z);
    let hl = HL.get(z);
    let addr = BC.get(z);
    z.get_mut_z80_hardware().address = addr;
    let x = z.input(addr);
    z.get_mut_z80_hardware().data = x;
    // XXX - the Z80 manual says HL is put on the address bus, but I am
    // skeptical about that
    // z.set_address_bus(hl);
    Address(hl).set(z, x);
    B.set(z, b.wrapping_sub(1));
    HL.set(z, hl.wrapping_add(inc));
    b.wrapping_sub(1)
}

pub fn ini<Z: Z80>(z: &mut Z) {
    log_minor!(z, "Z80: op: INI");

    let new_b = inid_impl(z, 1);
    let mut f = F.get(z);
    set_zero(&mut f, new_b);
    set_bit(&mut f, NF);
    F.set(z, f);
}

pub fn inir<Z: Z80>(z: &mut Z) {
    while {
        z.advance_t_states(21);
        inid_impl(z, 1) != 0
    }
    {}

    let mut f = F.get(z);
    set_bit(&mut f, ZF);
    set_bit(&mut f, NF);
    F.set(z, f);

    z.advance_t_states(16);
}

pub fn ind<Z: Z80>(z: &mut Z) {
    let new_b = inid_impl(z, 0xFFFF);
    let mut f = F.get(z);
    set_zero(&mut f, new_b);
    set_bit(&mut f, NF);
    F.set(z, f);
}

pub fn indr<Z: Z80>(z: &mut Z) {
    while {
        z.advance_t_states(21);
        inid_impl(z, 0xFFFF) != 0
    }
    {}

    let mut f = F.get(z);
    set_bit(&mut f, ZF);
    set_bit(&mut f, NF);
    F.set(z, f);

    z.advance_t_states(16);
}

pub fn out_n<Z: Z80, T1: Gettable<u8>, T2: Gettable<u8>> (
    z: &mut Z,
    arg1: T1,
    arg2: T2,
) {
    let address_lo = arg1.get(z);
    let address_hi = A.get(z);
    let address = to16(address_lo, address_hi);
    let x = arg2.get(z);
    z.get_mut_z80_hardware().address = address;
    z.get_mut_z80_hardware().data = x;
    z.output(address, x);
}

pub fn out_c<Z: Z80, T1: Gettable<u8>, T2: Gettable<u8>> (
    z: &mut Z,
    arg1: T1,
    arg2: T2,
) {
    let address_lo = arg1.get(z);
    let address_hi = B.get(z);
    let address = to16(address_lo, address_hi);
    let x = arg2.get(z);
    z.get_mut_z80_hardware().address = address;
    z.get_mut_z80_hardware().data = x;
    z.output(address, x);
}

fn outid_impl<Z: Z80>(z: &mut Z, inc: u16) {
    // let hl = HL.get(z);
    // The Z80 manual says HL is put on the address bus, but I am skeptical
    // about that
    // z.set_address_bus(hl);
    let b = B.get(z);
    let new_b = b.wrapping_sub(1);
    B.set(z, new_b);
    let addr = BC.get(z);
    z.get_mut_z80_hardware().address = addr;
    let hl = HL.get(z);
    let x = Address(hl).get(z);
    z.get_mut_z80_hardware().data = x;
    z.output(addr, x);
    HL.set(z, hl.wrapping_add(inc));
}

pub fn outi<Z: Z80>(z: &mut Z) {
    outid_impl(z, 1);
    let new_b = B.get(z);
    let mut f = F.get(z);
    set_zero(&mut f, new_b);
    set_bit(&mut f, NF);
    F.set(z, f);
}

pub fn otir<Z: Z80>(z: &mut Z) {
    while {
        z.advance_t_states(21);
        outid_impl(z, 1);
        B.get(z) != 0
    }
    {}

    let mut f = F.get(z);
    set_bit(&mut f, ZF);
    set_bit(&mut f, NF);

    z.advance_t_states(16);
}

pub fn outd<Z: Z80>(z: &mut Z) {
    outid_impl(z, 0xFFFF);
    let new_b = B.get(z);
    let mut f = F.get(z);
    set_zero(&mut f, new_b);
    set_bit(&mut f, NF);
    F.set(z, f);
}

pub fn otdr<Z: Z80>(z: &mut Z) {
    while {
        z.advance_t_states(21);
        outid_impl(z, 0xFFFF);
        B.get(z) != 0
    }
    {}

    let mut f = F.get(z);
    set_bit(&mut f, ZF);
    set_bit(&mut f, NF);

    z.advance_t_states(16);
}
