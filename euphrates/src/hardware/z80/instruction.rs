//! This module contains traits and functions useful for implementing a Z80
//! emulator.

use super::*;

macro_rules! def0 {
    ($trait_name:ident; $fn_name:ident) => {
        pub trait $trait_name {
            fn $fn_name(&mut self);
        }
    };
}

macro_rules! def1 {
    ($trait_name:ident; $fn_name:ident) => {
        pub trait $trait_name<A> {
            fn $fn_name(&mut self, a: A);
        }
    };
}

macro_rules! def2 {
    ($trait_name:ident; $fn_name:ident) => {
        pub trait $trait_name<L, R> {
            fn $fn_name(&mut self, lhs: L, rhs: R);
        }
    };
}

macro_rules! def3 {
    ($trait_name:ident; $fn_name:ident) => {
        pub trait $trait_name<A, B, C> {
            fn $fn_name(&mut self, a: A, b: B, c: C);
        }
    };
}

pub mod instruction_traits {

    //// 8 bit load group

    def2!{Ld; ld}
    def2!{LdIr; ld_ir}

    //// 16 bit load group

    def2!{Ld16; ld16}
    def1!{Push; push}
    def1!{Pop; pop}

    //// Exchange, Block Transfer, and Search Group

    def2!{Ex; ex}
    def0!{Exx; exx}
    def0!{Ldi; ldi}
    def0!{Ldir; ldir}
    def0!{Ldd; ldd}
    def0!{Lddr; lddr}
    def0!{Cpi; cpi}
    def0!{Cpir; cpir}
    def0!{Cpd; cpd}
    def0!{Cpdr; cpdr}

    //// 8-Bit Arithmetic Group

    def2!{Add; add}
    def2!{Adc; adc}
    def2!{Sub; sub}
    def2!{Sbc; sbc}
    def1!{And; and}
    def1!{Or; or}
    def1!{Xor; xor}
    def1!{Cp; cp}
    def1!{Inc; inc}
    def1!{Dec; dec}

    //// General-Purpose Arithmetic and CPU Control Group

    def0!{Daa; daa}
    def0!{Cpl; cpl}
    def0!{Neg; neg}
    def0!{Ccf; ccf}
    def0!{Scf; scf}
    def0!{Nop; nop}
    def0!{Halt; halt}
    def0!{Di; di}
    def0!{Ei; ei}
    def1!{Im; im}

    //// 16-Bit Arithmetic Group

    def2!{Add16; add16}
    def2!{Adc16; adc16}
    def2!{Sbc16; sbc16}
    def1!{Inc16; inc16}
    def1!{Dec16; dec16}

    //// Rotate and Shift Group

    def0!{Rlca; rlca}
    def0!{Rla; rla}
    def0!{Rrca; rrca}
    def0!{Rra; rra}
    def1!{Rlc; rlc}
    def2!{RlcStore; rlc_store}
    def1!{Rl; rl}
    def2!{RlStore; rl_store}
    def1!{Rrc; rrc}
    def2!{RrcStore; rrc_store}
    def1!{Rr; rr}
    def2!{RrStore; rr_store}
    def1!{Sla; sla}
    def2!{SlaStore; sla_store}
    def1!{Sra; sra}
    def2!{SraStore; sra_store}
    def1!{Sll; sll}
    def2!{SllStore; sll_store}
    def1!{Srl; srl}
    def2!{SrlStore; srl_store}
    def0!{Rld; rld}
    def0!{Rrd; rrd}

    //// Bit Set, Reset, and Test Group

    def2!{Bit; bit}
    def3!{BitStore; bit_store}
    def2!{Set; set}
    def3!{SetStore; set_store}
    def2!{Res; res}
    def3!{ResStore; res_store}

    //// Jump Group

    def1!{Jp; jp}
    def2!{Jpcc; jpcc}
    def1!{Jr; jr}
    def2!{Jrcc; jrcc}
    def1!{Djnz; djnz}

    //// Call and Return Group

    def1!{Call; call}
    def2!{Callcc; callcc}
    def0!{Ret; ret}
    def1!{Retcc; retcc}
    def0!{Reti; reti}
    def0!{Retn; retn}
    def1!{Rst; rst}

    //// Input and Output Group

    def2!{InN; in_n}
    def2!{InC; in_c}
    def1!{InF; in_f}
    def0!{Ini; ini}
    def0!{Inir; inir}
    def0!{Ind; ind}
    def0!{Indr; indr}
    def2!{OutN; out_n}
    def2!{OutC; out_c}
    def0!{Outi; outi}
    def0!{Otir; otir}
    def0!{Outd; outd}
    def0!{Otdr; otdr}

    //// Prefix transitions

    def0!{Dd; dd}
    def0!{Fd; fd}
    def0!{Cb; cb}
    def0!{Ed; ed}
    def0!{Ddcb; ddcb}
    def0!{Fdcb; fdcb}
}

use self::instruction_traits::*;

pub trait No:
    Adc16<Reg16, Reg16>
    + Add16<Reg16, Reg16>
    + Ccf
    + Cpl
    + Daa
    + Dec16<Reg16>
    + Di
    + Djnz<i8>
    + Ei
    + Exx
    + Halt
    + Im<u8>
    + Inc16<Reg16>
    + Jpcc<ConditionCode, u16>
    + Jr<i8>
    + Jrcc<ConditionCode, i8>
    + LdIr<Reg8, Reg8>
    + Neg
    + Nop
    + Sbc16<Reg16, Reg16>
    + Scf
    + Dd
    + Fd
    + Cb
    + Ed
    + Ddcb
    + Fdcb
{
}

impl<T> No for T
where
    T: Adc16<Reg16, Reg16>
        + Add16<Reg16, Reg16>
        + Ccf
        + Cpl
        + Daa
        + Dec16<Reg16>
        + Di
        + Djnz<i8>
        + Ei
        + Exx
        + Halt
        + Im<u8>
        + Inc16<Reg16>
        + Jpcc<ConditionCode, u16>
        + Jr<i8>
        + Jrcc<ConditionCode, i8>
        + LdIr<Reg8, Reg8>
        + Neg
        + Nop
        + Sbc16<Reg16, Reg16>
        + Scf
        + Dd
        + Fd
        + Cb
        + Ed
        + Ddcb
        + Fdcb,
{
}

pub trait Mem:
    Adc<Reg8, Reg8>
    + Adc<Reg8, u8>
    + Adc<Reg8, Address<Reg16>>
    + Adc<Reg8, Shift>
    + Add<Reg8, Reg8>
    + Add<Reg8, u8>
    + Add<Reg8, Address<Reg16>>
    + Add<Reg8, Shift>
    + And<Reg8>
    + And<u8>
    + And<Address<Reg16>>
    + And<Shift>
    + Bit<u8, Reg8>
    + Bit<u8, Address<Reg16>>
    + Bit<u8, Shift>
    + Call<u16>
    + Callcc<ConditionCode, u16>
    + Cp<Reg8>
    + Cp<u8>
    + Cp<Address<Reg16>>
    + Cp<Shift>
    + Cpd
    + Cpdr
    + Cpi
    + Cpir
    + Dec<Reg8>
    + Dec<Address<Reg16>>
    + Dec<Shift>
    + Ex<Reg16, Reg16>
    + Ex<Address<Reg16>, Reg16>
    + Inc<Reg8>
    + Inc<Address<Reg16>>
    + Inc<Shift>
    + Jp<u16>
    + Jp<Reg16>
    + Ld<Reg8, Reg8>
    + Ld<Reg8, u8>
    + Ld<Reg8, Address<Reg16>>
    + Ld<Reg8, Shift>
    + Ld<Reg8, Address<u16>>
    + Ld<Address<Reg16>, Reg8>
    + Ld<Shift, Reg8>
    + Ld<Shift, u8>
    + Ld<Address<Reg16>, u8>
    + Ld<Address<u16>, Reg8>
    + Ld<Address<Reg16>, Reg8>
    + Ld16<Reg16, u16>
    + Ld16<Reg16, Reg16>
    + Ld16<Reg16, Address<u16>>
    + Ld16<Address<u16>, Reg16>
    + Ldd
    + Lddr
    + Ldi
    + Ldir
    + Or<Reg8>
    + Or<u8>
    + Or<Address<Reg16>>
    + Or<Shift>
    + Pop<Reg16>
    + Push<Reg16>
    + Res<u8, Reg8>
    + Res<u8, Address<Reg16>>
    + Res<u8, Shift>
    + ResStore<u8, Shift, Reg8>
    + Ret
    + Retcc<ConditionCode>
    + Reti
    + Retn
    + Rl<Reg8>
    + Rl<Address<Reg16>>
    + Rl<Shift>
    + RlStore<Shift, Reg8>
    + Rla
    + Rlc<Reg8>
    + Rlc<Address<Reg16>>
    + Rlc<Shift>
    + RlcStore<Shift, Reg8>
    + Rlca
    + Rld
    + Rr<Reg8>
    + Rr<Address<Reg16>>
    + Rr<Shift>
    + RrStore<Shift, Reg8>
    + Rra
    + Rrc<Reg8>
    + Rrc<Address<Reg16>>
    + Rrc<Shift>
    + RrcStore<Shift, Reg8>
    + Rrca
    + Rrd
    + Rst<u8>
    + Sbc<Reg8, Reg8>
    + Sbc<Reg8, u8>
    + Sbc<Reg8, Address<Reg16>>
    + Sbc<Reg8, Shift>
    + Set<u8, Reg8>
    + Set<u8, Address<Reg16>>
    + Set<u8, Shift>
    + SetStore<u8, Shift, Reg8>
    + Sla<Reg8>
    + Sla<Address<Reg16>>
    + Sla<Shift>
    + SlaStore<Shift, Reg8>
    + Sll<Reg8>
    + Sll<Address<Reg16>>
    + Sll<Shift>
    + SllStore<Shift, Reg8>
    + Sra<Reg8>
    + Sra<Address<Reg16>>
    + Sra<Shift>
    + SraStore<Shift, Reg8>
    + Srl<Reg8>
    + Srl<Address<Reg16>>
    + Srl<Shift>
    + SrlStore<Shift, Reg8>
    + Sub<Reg8, Reg8>
    + Sub<Reg8, u8>
    + Sub<Reg8, Address<Reg16>>
    + Sub<Reg8, Shift>
    + Xor<Reg8>
    + Xor<u8>
    + Xor<Address<Reg16>>
    + Xor<Shift>
{
    fn ld<L, R>(&mut self, lhs: L, rhs: R)
    where
        Self: Ld<L, R>,
    {
        Ld::<L, R>::ld(self, lhs, rhs);
    }
}

impl<T> Mem for T
where
    T: Adc<Reg8, Reg8>
        + Adc<Reg8, u8>
        + Adc<Reg8, Address<Reg16>>
        + Adc<Reg8, Shift>
        + Add<Reg8, Reg8>
        + Add<Reg8, u8>
        + Add<Reg8, Address<Reg16>>
        + Add<Reg8, Shift>
        + And<Reg8>
        + And<u8>
        + And<Address<Reg16>>
        + And<Shift>
        + Bit<u8, Reg8>
        + Bit<u8, Address<Reg16>>
        + Bit<u8, Shift>
        + Call<u16>
        + Callcc<ConditionCode, u16>
        + Cp<Reg8>
        + Cp<u8>
        + Cp<Address<Reg16>>
        + Cp<Shift>
        + Cpd
        + Cpdr
        + Cpi
        + Cpir
        + Dec<Reg8>
        + Dec<Address<Reg16>>
        + Dec<Shift>
        + Ex<Reg16, Reg16>
        + Ex<Address<Reg16>, Reg16>
        + Inc<Reg8>
        + Inc<Address<Reg16>>
        + Inc<Shift>
        + Jp<u16>
        + Jp<Reg16>
        + Ld<Reg8, Reg8>
        + Ld<Reg8, u8>
        + Ld<Reg8, Address<Reg16>>
        + Ld<Reg8, Shift>
        + Ld<Reg8, Address<u16>>
        + Ld<Address<Reg16>, Reg8>
        + Ld<Address<Reg16>, u8>
        + Ld<Shift, Reg8>
+ Ld<Shift, u8>
        + Ld<Address<u16>, Reg8>
        + Ld16<Reg16, u16>
        + Ld16<Reg16, Reg16>
        + Ld16<Reg16, Address<u16>>
        + Ld16<Address<u16>, Reg16>
        + Ldd
        + Lddr
        + Ldi
        + Ldir
        + Or<Reg8>
        + Or<u8>
        + Or<Address<Reg16>>
        + Or<Shift>
        + Pop<Reg16>
        + Push<Reg16>
        + Res<u8, Reg8>
        + Res<u8, Address<Reg16>>
        + Res<u8, Shift>
        + ResStore<u8, Shift, Reg8>
        + Ret
        + Retcc<ConditionCode>
        + Reti
        + Retn
        + Rl<Reg8>
        + Rl<Address<Reg16>>
        + Rl<Shift>
        + RlStore<Shift, Reg8>
        + Rla
        + Rlc<Reg8>
        + Rlc<Address<Reg16>>
        + Rlc<Shift>
        + RlcStore<Shift, Reg8>
        + Rlca
        + Rld
        + Rr<Reg8>
        + Rr<Address<Reg16>>
        + Rr<Shift>
        + RrStore<Shift, Reg8>
        + Rra
        + Rrc<Reg8>
        + Rrc<Address<Reg16>>
        + Rrc<Shift>
        + RrcStore<Shift, Reg8>
        + Rrca
        + Rrd
        + Rst<u8>
        + Sbc<Reg8, Reg8>
        + Sbc<Reg8, u8>
        + Sbc<Reg8, Address<Reg16>>
        + Sbc<Reg8, Shift>
        + Set<u8, Reg8>
        + Set<u8, Address<Reg16>>
        + Set<u8, Shift>
        + SetStore<u8, Shift, Reg8>
        + Sla<Reg8>
        + Sla<Address<Reg16>>
        + Sla<Shift>
        + SlaStore<Shift, Reg8>
        + Sll<Reg8>
        + Sll<Address<Reg16>>
        + Sll<Shift>
        + SllStore<Shift, Reg8>
        + Sra<Reg8>
        + Sra<Address<Reg16>>
        + Sra<Shift>
        + SraStore<Shift, Reg8>
        + Srl<Reg8>
        + Srl<Address<Reg16>>
        + Srl<Shift>
        + SrlStore<Shift, Reg8>
        + Sub<Reg8, Reg8>
        + Sub<Reg8, u8>
        + Sub<Reg8, Address<Reg16>>
        + Sub<Reg8, Shift>
        + Xor<Reg8>
        + Xor<u8>
        + Xor<Address<Reg16>>
        + Xor<Shift>,
{
}

pub trait Io:
    InC<Reg8, Reg8>
    + InF<Reg8>
    + InN<Reg8, u8>
    + Ind
    + Indr
    + Ini
    + Inir
    + Otdr
    + Otir
    + Outd
    + Outi
    + OutC<Reg8, Reg8>
    + OutC<Reg8, u8>
    + OutN<u8, Reg8>
{
}

impl<T> Io for T
where
    T: InC<Reg8, Reg8>
        + InF<Reg8>
        + InN<Reg8, u8>
        + Ind
        + Indr
        + Ini
        + Inir
        + Otdr
        + Otir
        + Outd
        + Outi
        + OutC<Reg8, Reg8>
        + OutC<Reg8, u8>
        + OutN<u8, Reg8>,
{
}

pub trait Z80Emulator {
    type No: No;
    type Mem: Mem;
    type Io: Io;
    fn no<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self::No);
    fn mem<F>(&mut self, f :F)
    where
        F: FnOnce(&mut Self::Mem);
    fn io<F>(&mut self, f : F)
    where
        F: FnOnce(&mut Self::Io);
    /// Read the byte at the memory location addressed by PC.
    fn read_pc(&mut self) -> u8;
    fn inc_pc(&mut self);
    fn inc_cycles(&mut self, c: u64);
}

macro_rules! typof {
    (A) => { Reg8 };
    (B) => { Reg8 };
    (C) => { Reg8 };
    (D) => { Reg8 };
    (E) => { Reg8 };
    (F) => { Reg8 };
    (H) => { Reg8 };
    (L) => { Reg8 };
    (IXH) => { Reg8 };
    (IXL) => { Reg8 };
    (IYH) => { Reg8 };
    (IYL) => { Reg8 };
    (I) => { Reg8 };
    (R) => { Reg8 };
    (AF) => { Reg16 };
    (BC) => { Reg16 };
    (DE) => { Reg16 };
    (HL) => { Reg16 };
    (AF0) => { Reg16 };
    (BC0) => { Reg16 };
    (DE0) => { Reg16 };
    (HL0) => { Reg16 };
    (PC) => { Reg16 };
    (SP) => { Reg16 };
    (IX) => { Reg16 };
    (IY) => { Reg16 };
    (nn) => { u16 };
    (n) => { u8 };
    (d) => { i8 };
    (e) => { i8 };
    (Shift($($x: tt)*)) => { Shift };
    (Address($x: tt)) => { Address<typof!{$x}> };
    (NZcc) => { ConditionCode };
    (Zcc) => { ConditionCode };
    (NCcc) => { ConditionCode };
    (Ccc) => { ConditionCode };
    (POcc) => { ConditionCode };
    (PEcc) => { ConditionCode };
    (Pcc) => { ConditionCode };
    (Mcc) => { ConditionCode };
    (0x00) => { u8 };
    (0x08) => { u8 };
    (0x10) => { u8 };
    (0x18) => { u8 };
    (0x20) => { u8 };
    (0x28) => { u8 };
    (0x30) => { u8 };
    (0x38) => { u8 };
    (0) => { u8 };
    (1) => { u8 };
    (2) => { u8 };
    (3) => { u8 };
    (4) => { u8 };
    (5) => { u8 };
    (6) => { u8 };
    (7) => { u8 };
}

// in the macros below, the `args` argument must be `tt` so that we can pass it
// to `typof`, but then it becomes hard to parse compound expressions / like
// `Address(BC)`. Ugly solution: just put each argument in closed brackets like
// ([Address(BC)] [A]), and treat each argument like a sequence of tokens.

macro_rules! regular {
    ($z: ident, $cycles: expr, $subtype: ident, $cap: ident, $small: ident,
     ($([$($args: tt)*])*)) => {{
        $z.$subtype(|r|
            $cap::<
                $(
                    typof!{$($args)*}
                ),*
                >::$small(
                    r,
                    $(
                        $($args)*
                    ),*
                )
        );
        $z.inc_cycles($cycles);
    }}
}

macro_rules! nn_inst {
    ($z: ident, $nn: ident, $cycles: expr, $subtype: ident, $cap: ident, $small: ident,
     ($([$($args: tt)*])*)) => {{
         let lo = $z.read_pc();
         $z.inc_pc();
         let hi = $z.read_pc();
         $z.inc_pc();
         let $nn: u16 = (hi as u16) << 8 | (lo as u16);
        $z.$subtype(|r|
            $cap::<
                $(
                    typof!{$($args)*}
                ),*
                >::$small(
                    r,
                    $(
                        $($args)*
                    ),*
                )
        );
         $z.inc_cycles($cycles);
    }}
}

macro_rules! n_inst {
    ($z: ident, $n: ident, $cycles: expr, $subtype: ident, $cap: ident, $small: ident,
     ($([$($args: tt)*])*)) => {{
         let $n: u8 = $z.read_pc();
         $z.inc_pc();
        $z.$subtype(|r|
            $cap::<
                $(
                    typof!{$($args)*}
                ),*
                >::$small(
                    r,
                    $(
                        $($args)*
                    ),*
                )
        );
         $z.inc_cycles($cycles);
    }}
}

macro_rules! en_inst {
    ($z: ident, $e: ident, $n: ident, $cycles: expr, $subtype: ident, $cap: ident, $small: ident,
     ($([$($args: tt)*])*)) => {{
         let $e: i8 = $z.read_pc() as i8;
         $z.inc_pc();
         let $n: u8 = $z.read_pc();
         $z.inc_pc();
        $z.$subtype(|r|
            $cap::<
                $(
                    typof!{$($args)*}
                ),*
                >::$small(
                    r,
                    $(
                        $($args)*
                    ),*
                )
        );
         $z.inc_cycles($cycles);
    }}
}

macro_rules! e_inst {
    ($z: ident, $e: ident, $cycles: expr, $subtype: ident, $cap: ident, $small: ident,
     ($([$($args: tt)*])*)) => {{
         let $e: i8 = $z.read_pc() as i8;
         $z.inc_pc();
        $z.$subtype(|r|
            $cap::<
                $(
                    typof!{$($args)*}
                ),*
                >::$small(
                    r,
                    $(
                        $($args)*
                    ),*
                )
        );
         $z.inc_cycles($cycles);
    }}
}

pub fn noprefix<Z>(z: &mut Z)
where
    Z: Z80Emulator,
{
    use self::instruction_traits::*;

    use self::ConditionCode::*;
    use self::Reg16::*;
    use self::Reg8::*;

    // z.inc_r(1); XXX
    let pc = z.read_pc();
    z.inc_pc();
    match pc {
        0x00 => regular!{z,      4,  no,     Nop,     nop, () },
        0x01 => nn_inst!{z, nn, 10, mem,    Ld16,    ld16, ([BC] [nn]) },
        0x02 => regular!{z,      7, mem,      Ld,      ld, ([Address(BC)] [A]) },
        0x03 => regular!{z,      6,  no,   Inc16,   inc16, ([BC]) },
        0x04 => regular!{z,      4, mem,     Inc,     inc, ([B]) },
        0x05 => regular!{z,      4, mem,     Dec,     dec, ([B]) },
        0x06 => n_inst! {z,  n,  7, mem,      Ld,      ld, ([B] [n]) },
        0x07 => regular!{z,      4, mem,    Rlca,    rlca, () },
        0x08 => regular!{z,      4, mem,      Ex,      ex, ([AF] [BC0]) },
        0x09 => regular!{z,     11,  no,   Add16,   add16, ([HL] [BC]) },
        0x0A => regular!{z,      7, mem,      Ld,      ld, ([A] [Address(BC)]) },
        0x0B => regular!{z,      6,  no,   Dec16,   dec16, ([BC]) },
        0x0C => regular!{z,      4, mem,     Inc,     inc, ([C]) },
        0x0D => regular!{z,      4, mem,     Dec,     dec, ([C]) },
        0x0E => n_inst! {z,  n,  7, mem,      Ld,      ld, ([C] [n]) },
        0x0F => regular!{z,      4, mem,    Rrca,    rrca, () },
        0x10 => e_inst! {z,  e,  8,  no,    Djnz,    djnz, ([e]) },
        0x11 => nn_inst!{z, nn, 10, mem,    Ld16,    ld16, ([DE] [nn]) },
        0x12 => regular!{z,      7, mem,      Ld,      ld, ([Address(DE)] [A]) },
        0x13 => regular!{z,      6,  no,   Inc16,   inc16, ([DE]) },
        0x14 => regular!{z,      4, mem,     Inc,     inc, ([D]) },
        0x15 => regular!{z,      4, mem,     Dec,     dec, ([D]) },
        0x16 => n_inst! {z,  n,  7, mem,      Ld,      ld, ([D] [n]) },
        0x17 => regular!{z,      4, mem,     Rla,     rla, () },
        0x18 => e_inst! {z,  e, 12,  no,      Jr,      jr, ([e]) },
        0x19 => regular!{z,     11,  no,   Add16,   add16, ([HL] [DE]) },
        0x1A => regular!{z,      7, mem,      Ld,      ld, ([A] [Address(DE)]) },
        0x1B => regular!{z,      6,  no,   Dec16,   dec16, ([DE]) },
        0x1C => regular!{z,      4, mem,     Inc,     inc, ([E]) },
        0x1D => regular!{z,      4, mem,     Dec,     dec, ([E]) },
        0x1E => n_inst! {z,  n,  7, mem,      Ld,      ld, ([E] [n]) },
        0x1F => regular!{z,      4, mem,     Rra,     rra, () },
        0x20 => e_inst! {z,  e,  0,  no,    Jrcc,    jrcc, ([NZcc] [e]) },
        0x21 => nn_inst!{z, nn, 10, mem,    Ld16,    ld16, ([HL] [nn]) },
        0x22 => nn_inst!{z, nn, 16, mem,    Ld16,    ld16, ([Address(nn)] [HL]) },
        0x23 => regular!{z,      6,  no,   Inc16,   inc16, ([HL]) },
        0x24 => regular!{z,      4, mem,     Inc,     inc, ([H]) },
        0x25 => regular!{z,      4, mem,     Dec,     dec, ([H]) },
        0x26 => n_inst! {z,  n,  7, mem,      Ld,      ld, ([H] [n]) },
        0x27 => regular!{z,      4,  no,     Daa,     daa, () },
        0x28 => e_inst! {z,  e,  0,  no,    Jrcc,    jrcc, ([Zcc] [e]) },
        0x29 => regular!{z,     11,  no,   Add16,   add16, ([HL] [HL]) },
        0x2A => nn_inst!{z, nn, 16, mem,    Ld16,    ld16, ([HL] [Address(nn)]) },
        0x2B => regular!{z,      6,  no,   Dec16,   dec16, ([HL]) },
        0x2C => regular!{z,      4, mem,     Inc,     inc, ([L]) },
        0x2D => regular!{z,      4, mem,     Dec,     dec, ([L]) },
        0x2E => n_inst! {z,  n,  7, mem,      Ld,      ld, ([L] [n]) },
        0x2F => regular!{z,      4,  no,     Cpl,     cpl, () },
        0x30 => e_inst! {z,  e,  0,  no,    Jrcc,    jrcc, ([NCcc] [e]) },
        0x31 => nn_inst!{z, nn, 10, mem,    Ld16,    ld16, ([SP] [nn]) },
        0x32 => nn_inst!{z, nn, 13, mem,      Ld,      ld, ([Address(nn)] [A]) },
        0x33 => regular!{z,      6,  no,   Inc16,   inc16, ([SP]) },
        0x34 => regular!{z,     11, mem,     Inc,     inc, ([Address(HL)]) },
        0x35 => regular!{z,     11, mem,     Dec,     dec, ([Address(HL)]) },
        0x36 => n_inst! {z,  n, 10, mem,      Ld,      ld, ([Address(HL)] [n]) },
        0x37 => regular!{z,      4,  no,     Scf,     scf, () },
        0x38 => e_inst! {z,  e,  0,  no,    Jrcc,    jrcc, ([Ccc] [e]) },
        0x39 => regular!{z,     11,  no,   Add16,   add16, ([HL] [SP]) },
        0x3A => nn_inst!{z, nn, 13, mem,      Ld,      ld, ([A] [Address(nn)]) },
        0x3B => regular!{z,      6,  no,   Dec16,   dec16, ([SP]) },
        0x3C => regular!{z,      4, mem,     Inc,     inc, ([A]) },
        0x3D => regular!{z,      4, mem,     Dec,     dec, ([A]) },
        0x3E => n_inst! {z,  n,  7, mem,      Ld,      ld, ([A] [n]) },
        0x3F => regular!{z,      4,  no,     Ccf,     ccf, () },
        0x40 => regular!{z,      4, mem,      Ld,      ld, ([B] [B]) },
        0x41 => regular!{z,      4, mem,      Ld,      ld, ([B] [C]) },
        0x42 => regular!{z,      4, mem,      Ld,      ld, ([B] [D]) },
        0x43 => regular!{z,      4, mem,      Ld,      ld, ([B] [E]) },
        0x44 => regular!{z,      4, mem,      Ld,      ld, ([B] [H]) },
        0x45 => regular!{z,      4, mem,      Ld,      ld, ([B] [L]) },
        0x46 => regular!{z,      7, mem,      Ld,      ld, ([B] [Address(HL)]) },
        0x47 => regular!{z,      4, mem,      Ld,      ld, ([B] [A]) },
        0x48 => regular!{z,      4, mem,      Ld,      ld, ([C] [B]) },
        0x49 => regular!{z,      4, mem,      Ld,      ld, ([C] [C]) },
        0x4A => regular!{z,      4, mem,      Ld,      ld, ([C] [D]) },
        0x4B => regular!{z,      4, mem,      Ld,      ld, ([C] [E]) },
        0x4C => regular!{z,      4, mem,      Ld,      ld, ([C] [H]) },
        0x4D => regular!{z,      4, mem,      Ld,      ld, ([C] [L]) },
        0x4E => regular!{z,      7, mem,      Ld,      ld, ([C] [Address(HL)]) },
        0x4F => regular!{z,      4, mem,      Ld,      ld, ([C] [A]) },
        0x50 => regular!{z,      4, mem,      Ld,      ld, ([D] [B]) },
        0x51 => regular!{z,      4, mem,      Ld,      ld, ([D] [C]) },
        0x52 => regular!{z,      4, mem,      Ld,      ld, ([D] [D]) },
        0x53 => regular!{z,      4, mem,      Ld,      ld, ([D] [E]) },
        0x54 => regular!{z,      4, mem,      Ld,      ld, ([D] [H]) },
        0x55 => regular!{z,      4, mem,      Ld,      ld, ([D] [L]) },
        0x56 => regular!{z,      7, mem,      Ld,      ld, ([D] [Address(HL)]) },
        0x57 => regular!{z,      4, mem,      Ld,      ld, ([D] [A]) },
        0x58 => regular!{z,      4, mem,      Ld,      ld, ([E] [B]) },
        0x59 => regular!{z,      4, mem,      Ld,      ld, ([E] [C]) },
        0x5A => regular!{z,      4, mem,      Ld,      ld, ([E] [D]) },
        0x5B => regular!{z,      4, mem,      Ld,      ld, ([E] [E]) },
        0x5C => regular!{z,      4, mem,      Ld,      ld, ([E] [H]) },
        0x5D => regular!{z,      4, mem,      Ld,      ld, ([E] [L]) },
        0x5E => regular!{z,      7, mem,      Ld,      ld, ([E] [Address(HL)]) },
        0x5F => regular!{z,      4, mem,      Ld,      ld, ([E] [A]) },
        0x60 => regular!{z,      4, mem,      Ld,      ld, ([H] [B]) },
        0x61 => regular!{z,      4, mem,      Ld,      ld, ([H] [C]) },
        0x62 => regular!{z,      4, mem,      Ld,      ld, ([H] [D]) },
        0x63 => regular!{z,      4, mem,      Ld,      ld, ([H] [E]) },
        0x64 => regular!{z,      4, mem,      Ld,      ld, ([H] [H]) },
        0x65 => regular!{z,      4, mem,      Ld,      ld, ([H] [L]) },
        0x66 => regular!{z,      7, mem,      Ld,      ld, ([H] [Address(HL)]) },
        0x67 => regular!{z,      4, mem,      Ld,      ld, ([H] [A]) },
        0x68 => regular!{z,      4, mem,      Ld,      ld, ([L] [B]) },
        0x69 => regular!{z,      4, mem,      Ld,      ld, ([L] [C]) },
        0x6A => regular!{z,      4, mem,      Ld,      ld, ([L] [D]) },
        0x6B => regular!{z,      4, mem,      Ld,      ld, ([L] [E]) },
        0x6C => regular!{z,      4, mem,      Ld,      ld, ([L] [H]) },
        0x6D => regular!{z,      4, mem,      Ld,      ld, ([L] [L]) },
        0x6E => regular!{z,      7, mem,      Ld,      ld, ([L] [Address(HL)]) },
        0x6F => regular!{z,      4, mem,      Ld,      ld, ([L] [A]) },
        0x70 => regular!{z,      7, mem,      Ld,      ld, ([Address(HL)] [B]) },
        0x71 => regular!{z,      7, mem,      Ld,      ld, ([Address(HL)] [C]) },
        0x72 => regular!{z,      7, mem,      Ld,      ld, ([Address(HL)] [D]) },
        0x73 => regular!{z,      7, mem,      Ld,      ld, ([Address(HL)] [E]) },
        0x74 => regular!{z,      7, mem,      Ld,      ld, ([Address(HL)] [H]) },
        0x75 => regular!{z,      7, mem,      Ld,      ld, ([Address(HL)] [L]) },
        0x76 => regular!{z,      4,  no,    Halt,    halt, () },
        0x77 => regular!{z,      7, mem,      Ld,      ld, ([Address(HL)] [A]) },
        0x78 => regular!{z,      4, mem,      Ld,      ld, ([A] [B]) },
        0x79 => regular!{z,      4, mem,      Ld,      ld, ([A] [C]) },
        0x7A => regular!{z,      4, mem,      Ld,      ld, ([A] [D]) },
        0x7B => regular!{z,      4, mem,      Ld,      ld, ([A] [E]) },
        0x7C => regular!{z,      4, mem,      Ld,      ld, ([A] [H]) },
        0x7D => regular!{z,      4, mem,      Ld,      ld, ([A] [L]) },
        0x7E => regular!{z,      7, mem,      Ld,      ld, ([A] [Address(HL)]) },
        0x7F => regular!{z,      4, mem,      Ld,      ld, ([A] [A]) },
        0x80 => regular!{z,      4, mem,     Add,     add, ([A] [B]) },
        0x81 => regular!{z,      4, mem,     Add,     add, ([A] [C]) },
        0x82 => regular!{z,      4, mem,     Add,     add, ([A] [D]) },
        0x83 => regular!{z,      4, mem,     Add,     add, ([A] [E]) },
        0x84 => regular!{z,      4, mem,     Add,     add, ([A] [H]) },
        0x85 => regular!{z,      4, mem,     Add,     add, ([A] [L]) },
        0x86 => regular!{z,      7, mem,     Add,     add, ([A] [Address(HL)]) },
        0x87 => regular!{z,      4, mem,     Add,     add, ([A] [A]) },
        0x88 => regular!{z,      4, mem,     Adc,     adc, ([A] [B]) },
        0x89 => regular!{z,      4, mem,     Adc,     adc, ([A] [C]) },
        0x8A => regular!{z,      4, mem,     Adc,     adc, ([A] [D]) },
        0x8B => regular!{z,      4, mem,     Adc,     adc, ([A] [E]) },
        0x8C => regular!{z,      4, mem,     Adc,     adc, ([A] [H]) },
        0x8D => regular!{z,      4, mem,     Adc,     adc, ([A] [L]) },
        0x8E => regular!{z,      7, mem,     Adc,     adc, ([A] [Address(HL)]) },
        0x8F => regular!{z,      4, mem,     Adc,     adc, ([A] [A]) },
        0x90 => regular!{z,      4, mem,     Sub,     sub, ([A] [B]) },
        0x91 => regular!{z,      4, mem,     Sub,     sub, ([A] [C]) },
        0x92 => regular!{z,      4, mem,     Sub,     sub, ([A] [D]) },
        0x93 => regular!{z,      4, mem,     Sub,     sub, ([A] [E]) },
        0x94 => regular!{z,      4, mem,     Sub,     sub, ([A] [H]) },
        0x95 => regular!{z,      4, mem,     Sub,     sub, ([A] [L]) },
        0x96 => regular!{z,      7, mem,     Sub,     sub, ([A] [Address(HL)]) },
        0x97 => regular!{z,      4, mem,     Sub,     sub, ([A] [A]) },
        0x98 => regular!{z,      4, mem,     Sbc,     sbc, ([A] [B]) },
        0x99 => regular!{z,      4, mem,     Sbc,     sbc, ([A] [C]) },
        0x9A => regular!{z,      4, mem,     Sbc,     sbc, ([A] [D]) },
        0x9B => regular!{z,      4, mem,     Sbc,     sbc, ([A] [E]) },
        0x9C => regular!{z,      4, mem,     Sbc,     sbc, ([A] [H]) },
        0x9D => regular!{z,      4, mem,     Sbc,     sbc, ([A] [L]) },
        0x9E => regular!{z,      7, mem,     Sbc,     sbc, ([A] [Address(HL)]) },
        0x9F => regular!{z,      4, mem,     Sbc,     sbc, ([A] [A]) },
        0xA0 => regular!{z,      4, mem,     And,     and, ([B]) },
        0xA1 => regular!{z,      4, mem,     And,     and, ([C]) },
        0xA2 => regular!{z,      4, mem,     And,     and, ([D]) },
        0xA3 => regular!{z,      4, mem,     And,     and, ([E]) },
        0xA4 => regular!{z,      4, mem,     And,     and, ([H]) },
        0xA5 => regular!{z,      4, mem,     And,     and, ([L]) },
        0xA6 => regular!{z,      7, mem,     And,     and, ([Address(HL)]) },
        0xA7 => regular!{z,      4, mem,     And,     and, ([A]) },
        0xA8 => regular!{z,      4, mem,     Xor,     xor, ([B]) },
        0xA9 => regular!{z,      4, mem,     Xor,     xor, ([C]) },
        0xAA => regular!{z,      4, mem,     Xor,     xor, ([D]) },
        0xAB => regular!{z,      4, mem,     Xor,     xor, ([E]) },
        0xAC => regular!{z,      4, mem,     Xor,     xor, ([H]) },
        0xAD => regular!{z,      4, mem,     Xor,     xor, ([L]) },
        0xAE => regular!{z,      7, mem,     Xor,     xor, ([Address(HL)]) },
        0xAF => regular!{z,      4, mem,     Xor,     xor, ([A]) },
        0xB0 => regular!{z,      4, mem,      Or,     or, ([B]) },
        0xB1 => regular!{z,      4, mem,      Or,     or, ([C]) },
        0xB2 => regular!{z,      4, mem,      Or,     or, ([D]) },
        0xB3 => regular!{z,      4, mem,      Or,     or, ([E]) },
        0xB4 => regular!{z,      4, mem,      Or,     or, ([H]) },
        0xB5 => regular!{z,      4, mem,      Or,     or, ([L]) },
        0xB6 => regular!{z,      7, mem,      Or,     or, ([Address(HL)]) },
        0xB7 => regular!{z,      4, mem,      Or,     or, ([A]) },
        0xB8 => regular!{z,      4, mem,      Cp,     cp, ([B]) },
        0xB9 => regular!{z,      4, mem,      Cp,     cp, ([C]) },
        0xBA => regular!{z,      4, mem,      Cp,     cp, ([D]) },
        0xBB => regular!{z,      4, mem,      Cp,     cp, ([E]) },
        0xBC => regular!{z,      4, mem,      Cp,     cp, ([H]) },
        0xBD => regular!{z,      4, mem,      Cp,     cp, ([L]) },
        0xBE => regular!{z,      7, mem,      Cp,     cp, ([Address(HL)]) },
        0xBF => regular!{z,      4, mem,      Cp,     cp, ([A]) },
        0xC0 => regular!{z,      0, mem,   Retcc,  retcc, ([NZcc]) },
        0xC1 => regular!{z,     10, mem,     Pop,    pop, ([BC]) },
        0xC2 => nn_inst!{z, nn, 10,  no,    Jpcc,   jpcc, ([NZcc] [nn]) },
        0xC3 => nn_inst!{z, nn, 10, mem,      Jp,     jp, ([nn]) },
        0xC4 => nn_inst!{z, nn,  0, mem,  Callcc, callcc, ([NZcc] [nn]) },
        0xC5 => regular!{z,     11, mem,    Push,   push, ([BC]) },
        0xC6 => n_inst! {z,  n,  7, mem,     Add,    add, ([A] [n]) },
        0xC7 => regular!{z,     11, mem,     Rst,    rst, ([0x00]) },
        0xC8 => regular!{z,      0, mem,   Retcc,  retcc, ([Zcc]) },
        0xC9 => regular!{z,     10, mem,     Ret,    ret, () },
        0xCA => nn_inst!{z, nn, 10,  no,    Jpcc,   jpcc, ([Zcc] [nn]) },
        0xCB => regular!{z,      0,  no,      Cb,     cb, () },
        0xCC => nn_inst!{z, nn,  0, mem,  Callcc, callcc, ([Zcc] [nn]) },
        0xCD => nn_inst!{z, nn,  0, mem,    Call,   call, ([nn]) },
        0xCE => n_inst! {z,  n,  7, mem,     Adc,    adc, ([A] [n]) },
        0xCF => regular!{z,     11, mem,    Rst,     rst, ([0x08]) },
        0xD0 => regular!{z,      0, mem,  Retcc,   retcc, ([NCcc]) },
        0xD1 => regular!{z,     10, mem,    Pop,     pop, ([DE]) },
        0xD2 => nn_inst!{z, nn, 10,  no,   Jpcc,    jpcc, ([NCcc] [nn]) },
        0xD3 => n_inst! {z,  n, 11,  io,   OutN,   out_n, ([n] [A]) },
        0xD4 => nn_inst!{z, nn,  0, mem, Callcc,  callcc, ([NCcc] [nn]) },
        0xD5 => regular!{z,     11, mem,   Push,    push, ([DE]) },
        0xD6 => n_inst! {z,  n,  7, mem,    Sub,     sub, ([A] [n]) },
        0xD7 => regular!{z,     11, mem,    Rst,     rst, ([0x10]) },
        0xD8 => regular!{z,      0, mem,  Retcc,   retcc, ([Ccc]) },
        0xD9 => regular!{z,      4,  no,    Exx,     exx, () },
        0xDA => nn_inst!{z, nn, 10,  no,   Jpcc,    jpcc, ([Ccc] [nn]) },
        0xDB => n_inst! {z,  n, 11,  io,    InN,    in_n, ([A] [n]) },
        0xDC => nn_inst!{z, nn,  0, mem, Callcc,  callcc, ([Ccc] [nn]) },
        0xDD => regular!{z,      4,  no,     Dd,      dd, () },
        0xDE => n_inst! {z,  n,  7, mem,    Sbc,     sbc, ([A] [n]) },
        0xDF => regular!{z,     11, mem,    Rst,     rst, ([0x18]) },
        0xE0 => regular!{z,      0, mem,  Retcc,   retcc, ([POcc]) },
        0xE1 => regular!{z,     10, mem,    Pop,     pop, ([HL]) },
        0xE2 => nn_inst!{z, nn, 10,  no,   Jpcc,    jpcc, ([POcc] [nn]) },
        0xE3 => regular!{z,     19, mem,     Ex,      ex, ([Address(SP)] [HL]) },
        0xE4 => nn_inst!{z, nn,  0, mem, Callcc,  callcc, ([POcc] [nn]) },
        0xE5 => regular!{z,     11, mem,   Push,    push, ([HL]) },
        0xE6 => n_inst! {z,  n,  7, mem,    And,     and, ([n]) },
        0xE7 => regular!{z,     11, mem,    Rst,     rst, ([0x20]) },
        0xE8 => regular!{z,      0, mem,  Retcc,   retcc, ([PEcc]) },
        0xE9 => regular!{z,      4, mem,     Jp,      jp, ([HL]) },
        0xEA => nn_inst!{z, nn, 10,  no,   Jpcc,    jpcc, ([PEcc] [nn]) },
        0xEB => regular!{z,      4, mem,     Ex,      ex, ([DE] [HL]) },
        0xEC => nn_inst!{z, nn,  0, mem, Callcc,  callcc, ([PEcc] [nn]) },
        0xED => regular!{z,      0,  no,     Ed,      ed, () },
        0xEE => n_inst! {z,  n,  7, mem,    Xor,     xor, ([n]) },
        0xEF => regular!{z,     11, mem,    Rst,     rst, ([0x28]) },
        0xF0 => regular!{z,      0, mem,  Retcc,   retcc, ([Pcc]) },
        0xF1 => regular!{z,     10, mem,    Pop,     pop, ([AF]) },
        0xF2 => nn_inst!{z, nn, 10,  no,   Jpcc,    jpcc, ([Pcc] [nn]) },
        0xF3 => regular!{z,      4,  no,     Di,      di, () },
        0xF4 => nn_inst!{z, nn,  0, mem, Callcc,  callcc, ([Pcc] [nn]) },
        0xF5 => regular!{z,     11, mem,   Push,    push, ([AF]) },
        0xF6 => n_inst! {z,  n,  7, mem,     Or,      or, ([n]) },
        0xF7 => regular!{z,     11, mem,    Rst,     rst, ([0x30]) },
        0xF8 => regular!{z,      0, mem,  Retcc,   retcc, ([Mcc]) },
        0xF9 => regular!{z,      6, mem,   Ld16,    ld16, ([SP] [HL]) },
        0xFA => nn_inst!{z, nn, 10,  no,   Jpcc,    jpcc, ([Mcc] [nn]) },
        0xFB => regular!{z,      4,  no,     Ei,      ei, () },
        0xFC => nn_inst!{z, nn,  0, mem, Callcc,  callcc, ([Mcc] [nn]) },
        0xFD => regular!{z,      4,  no,     Fd,      fd, () },
        0xFE => n_inst! {z,  n,  7, mem,     Cp,      cp, ([n]) },
        0xFF => regular!{z,     11, mem,    Rst,     rst, ([0x38]) },
        _ => unimplemented!(),
    }
}

pub fn ed<Z>(z: &mut Z)
where
    Z: Z80Emulator,
{
    use self::instruction_traits::*;

    use self::Reg16::*;
    use self::Reg8::*;

    let pc = z.read_pc();
    z.inc_pc();
    match pc {
        0x00 => regular!{z,  8, no, Nop, nop, () },
        0x01 => regular!{z,  8, no, Nop, nop, () },
        0x02 => regular!{z,  8, no, Nop, nop, () },
        0x03 => regular!{z,  8, no, Nop, nop, () },
        0x04 => regular!{z,  8, no, Nop, nop, () },
        0x05 => regular!{z,  8, no, Nop, nop, () },
        0x06 => regular!{z,  8, no, Nop, nop, () },
        0x07 => regular!{z,  8, no, Nop, nop, () },
        0x08 => regular!{z,  8, no, Nop, nop, () },
        0x09 => regular!{z,  8, no, Nop, nop, () },
        0x0A => regular!{z,  8, no, Nop, nop, () },
        0x0B => regular!{z,  8, no, Nop, nop, () },
        0x0C => regular!{z,  8, no, Nop, nop, () },
        0x0D => regular!{z,  8, no, Nop, nop, () },
        0x0E => regular!{z,  8, no, Nop, nop, () },
        0x0F => regular!{z,  8, no, Nop, nop, () },
        0x10 => regular!{z,  8, no, Nop, nop, () },
        0x11 => regular!{z,  8, no, Nop, nop, () },
        0x12 => regular!{z,  8, no, Nop, nop, () },
        0x13 => regular!{z,  8, no, Nop, nop, () },
        0x14 => regular!{z,  8, no, Nop, nop, () },
        0x15 => regular!{z,  8, no, Nop, nop, () },
        0x16 => regular!{z,  8, no, Nop, nop, () },
        0x17 => regular!{z,  8, no, Nop, nop, () },
        0x18 => regular!{z,  8, no, Nop, nop, () },
        0x19 => regular!{z,  8, no, Nop, nop, () },
        0x1A => regular!{z,  8, no, Nop, nop, () },
        0x1B => regular!{z,  8, no, Nop, nop, () },
        0x1C => regular!{z,  8, no, Nop, nop, () },
        0x1D => regular!{z,  8, no, Nop, nop, () },
        0x1E => regular!{z,  8, no, Nop, nop, () },
        0x1F => regular!{z,  8, no, Nop, nop, () },
        0x20 => regular!{z,  8, no, Nop, nop, () },
        0x21 => regular!{z,  8, no, Nop, nop, () },
        0x22 => regular!{z,  8, no, Nop, nop, () },
        0x23 => regular!{z,  8, no, Nop, nop, () },
        0x24 => regular!{z,  8, no, Nop, nop, () },
        0x25 => regular!{z,  8, no, Nop, nop, () },
        0x26 => regular!{z,  8, no, Nop, nop, () },
        0x27 => regular!{z,  8, no, Nop, nop, () },
        0x28 => regular!{z,  8, no, Nop, nop, () },
        0x29 => regular!{z,  8, no, Nop, nop, () },
        0x2A => regular!{z,  8, no, Nop, nop, () },
        0x2B => regular!{z,  8, no, Nop, nop, () },
        0x2C => regular!{z,  8, no, Nop, nop, () },
        0x2D => regular!{z,  8, no, Nop, nop, () },
        0x2E => regular!{z,  8, no, Nop, nop, () },
        0x2F => regular!{z,  8, no, Nop, nop, () },
        0x30 => regular!{z,  8, no, Nop, nop, () },
        0x31 => regular!{z,  8, no, Nop, nop, () },
        0x32 => regular!{z,  8, no, Nop, nop, () },
        0x33 => regular!{z,  8, no, Nop, nop, () },
        0x34 => regular!{z,  8, no, Nop, nop, () },
        0x35 => regular!{z,  8, no, Nop, nop, () },
        0x36 => regular!{z,  8, no, Nop, nop, () },
        0x37 => regular!{z,  8, no, Nop, nop, () },
        0x38 => regular!{z,  8, no, Nop, nop, () },
        0x39 => regular!{z,  8, no, Nop, nop, () },
        0x3A => regular!{z,  8, no, Nop, nop, () },
        0x3B => regular!{z,  8, no, Nop, nop, () },
        0x3C => regular!{z,  8, no, Nop, nop, () },
        0x3D => regular!{z,  8, no, Nop, nop, () },
        0x3E => regular!{z,  8, no, Nop, nop, () },
        0x3F => regular!{z,  8, no, Nop, nop, () },

        0x40 => regular!{z,     12,  io,   InC,  in_c, ([B] [C]) },
        0x41 => regular!{z,     12,  io,  OutC, out_c, ([C] [B]) },
        0x42 => regular!{z,     15,  no, Sbc16, sbc16, ([HL] [BC]) },
        0x43 => nn_inst!{z, nn, 20, mem,  Ld16,  ld16, ([Address(nn)] [BC]) },
        0x44 => regular!{z,      8,  no,   Neg,   neg, () },
        0x45 => regular!{z,     14, mem,  Retn,  retn, () },
        0x46 => regular!{z,      8,  no,    Im,    im, ([0]) },
        0x47 => regular!{z,      9, mem,    Ld,    ld, ([I] [A]) },
        0x48 => regular!{z,     12,  io,   InC,  in_c, ([C] [C]) },
        0x49 => regular!{z,     12,  io,  OutC, out_c, ([C] [C]) },
        0x4A => regular!{z,     15,  no, Adc16, adc16, ([HL] [BC]) },
        0x4B => nn_inst!{z, nn, 20, mem,  Ld16,  ld16, ([BC] [Address(nn)]) },
        0x4C => regular!{z,      8,  no,   Neg,   neg, () },
        0x4D => regular!{z,     14, mem,  Reti,  reti, () },
        0x4E => regular!{z,      8,  no,    Im,    im, ([0]) },
        0x4F => regular!{z,      9, mem,    Ld,    ld, ([R] [A]) },
        0x50 => regular!{z,     12,  io,   InC,  in_c, ([D] [C]) },
        0x51 => regular!{z,     12,  io,  OutC, out_c, ([C] [D]) },
        0x52 => regular!{z,     15,  no, Sbc16, sbc16, ([HL] [DE]) },
        0x53 => nn_inst!{z, nn, 20, mem,  Ld16,  ld16, ([Address(nn)] [DE]) },
        0x54 => regular!{z,      8,  no,   Neg, neg, () },
        0x55 => regular!{z,     14, mem,  Retn, retn, () },
        0x56 => regular!{z,      8,  no,    Im, im, ([1]) },
        0x57 => regular!{z,      9,  no,  LdIr, ld_ir, ([A] [I]) },
        0x58 => regular!{z,     12,  io,   InC, in_c, ([E] [C]) },
        0x59 => regular!{z,     12,  io,  OutC, out_c, ([C] [E]) },
        0x5A => regular!{z,     15,  no, Adc16, adc16, ([HL] [DE]) },
        0x5B => nn_inst!{z, nn, 20, mem,  Ld16, ld16, ([DE] [Address(nn)]) },
        0x5C => regular!{z,      8,  no,   Neg, neg, () },
        0x5D => regular!{z,     14, mem,  Retn, retn, () },
        0x5E => regular!{z,      8,  no,    Im, im, ([2]) },
        0x5F => regular!{z,      9,  no,  LdIr, ld_ir, ([A] [R]) },
        0x60 => regular!{z,     12,  io,   InC, in_c, ([H] [C]) },
        0x61 => regular!{z,     12,  io,  OutC, out_c, ([C] [H]) },
        0x62 => regular!{z,     15,  no, Sbc16, sbc16, ([HL] [HL]) },
        0x63 => nn_inst!{z, nn, 20, mem,  Ld16, ld16, ([Address(nn)] [HL]) },
        0x64 => regular!{z,      8,  no,   Neg, neg, () },
        0x65 => regular!{z,     14, mem,  Retn, retn, () },
        0x66 => regular!{z,      8,  no,    Im, im, ([0]) },
        0x67 => regular!{z,     18, mem,   Rrd, rrd, () },
        0x68 => regular!{z,     12,  io,   InC, in_c, ([L] [C]) },
        0x69 => regular!{z,     12,  io,  OutC, out_c, ([C] [L]) },
        0x6A => regular!{z,     15,  no, Adc16, adc16, ([HL] [HL]) },
        0x6B => nn_inst!{z, nn, 20, mem,  Ld16, ld16, ([HL] [Address(nn)]) },
        0x6C => regular!{z,      8,  no,   Neg,   neg, () },
        0x6D => regular!{z,     14, mem,  Retn,  retn, () },
        0x6E => regular!{z,      8,  no,    Im,    im, ([0]) },
        0x6F => regular!{z,     18, mem,   Rld,   rld, () },
        0x70 => regular!{z,     12,  io,   InF,  in_f, ([C]) },
        0x71 => regular!{z,     12,  io,  OutC, out_c, ([C] [0]) },
        0x72 => regular!{z,     15,  no, Sbc16, sbc16, ([HL] [SP]) },
        0x73 => nn_inst!{z, nn, 20, mem,  Ld16,  ld16, ([Address(nn)] [SP]) },
        0x74 => regular!{z,      8,  no,   Neg,  neg, () },
        0x75 => regular!{z,     14, mem,  Retn, retn, () },
        0x76 => regular!{z,      8,  no,    Im,   im, ([1]) },

        0x77 => regular!{z,      8,  no,   Nop,  nop, () },

        0x78 => regular!{z,     12,  io,   InC,  in_c, ([A] [C]) },
        0x79 => regular!{z,     12,  io,  OutC, out_c, ([C] [A]) },
        0x7A => regular!{z,     15,  no, Adc16, adc16, ([HL] [SP]) },
        0x7B => nn_inst!{z, nn, 20, mem,  Ld16,  ld16, ([SP] [Address(nn)]) },
        0x7C => regular!{z,      8,  no,   Neg,   neg, () },
        0x7D => regular!{z,     14, mem,  Retn,  retn, () },
        0x7E => regular!{z,      8,  no,    Im,    im, ([2]) },

        0x7F => regular!{z,  8, no, Nop, nop, () },
        0x80 => regular!{z,  8, no, Nop, nop, () },
        0x81 => regular!{z,  8, no, Nop, nop, () },
        0x82 => regular!{z,  8, no, Nop, nop, () },
        0x83 => regular!{z,  8, no, Nop, nop, () },
        0x84 => regular!{z,  8, no, Nop, nop, () },
        0x85 => regular!{z,  8, no, Nop, nop, () },
        0x86 => regular!{z,  8, no, Nop, nop, () },
        0x87 => regular!{z,  8, no, Nop, nop, () },
        0x88 => regular!{z,  8, no, Nop, nop, () },
        0x89 => regular!{z,  8, no, Nop, nop, () },
        0x8A => regular!{z,  8, no, Nop, nop, () },
        0x8B => regular!{z,  8, no, Nop, nop, () },
        0x8C => regular!{z,  8, no, Nop, nop, () },
        0x8D => regular!{z,  8, no, Nop, nop, () },
        0x8E => regular!{z,  8, no, Nop, nop, () },
        0x8F => regular!{z,  8, no, Nop, nop, () },
        0x90 => regular!{z,  8, no, Nop, nop, () },
        0x91 => regular!{z,  8, no, Nop, nop, () },
        0x92 => regular!{z,  8, no, Nop, nop, () },
        0x93 => regular!{z,  8, no, Nop, nop, () },
        0x94 => regular!{z,  8, no, Nop, nop, () },
        0x95 => regular!{z,  8, no, Nop, nop, () },
        0x96 => regular!{z,  8, no, Nop, nop, () },
        0x97 => regular!{z,  8, no, Nop, nop, () },
        0x98 => regular!{z,  8, no, Nop, nop, () },
        0x99 => regular!{z,  8, no, Nop, nop, () },
        0x9A => regular!{z,  8, no, Nop, nop, () },
        0x9B => regular!{z,  8, no, Nop, nop, () },
        0x9C => regular!{z,  8, no, Nop, nop, () },
        0x9D => regular!{z,  8, no, Nop, nop, () },
        0x9E => regular!{z,  8, no, Nop, nop, () },
        0x9F => regular!{z,  8, no, Nop, nop, () },

        0xA0 => regular!{z, 16, mem,  Ldi,  ldi, () },
        0xA1 => regular!{z, 16, mem,  Cpi,  cpi, () },
        0xA2 => regular!{z, 16,  io,  Ini,  ini, () },
        0xA3 => regular!{z, 16,  io, Outi, outi, () },

        0xA4 => regular!{z,  8, no, Nop, nop, () },
        0xA5 => regular!{z,  8, no, Nop, nop, () },
        0xA6 => regular!{z,  8, no, Nop, nop, () },
        0xA7 => regular!{z,  8, no, Nop, nop, () },

        0xA8 => regular!{z, 16, mem,  Ldd,  ldd, () },
        0xA9 => regular!{z, 16, mem,  Cpd,  cpd, () },
        0xAA => regular!{z, 16,  io,  Ind,  ind, () },
        0xAB => regular!{z, 16,  io, Outd, outd, () },

        0xAC => regular!{z,  8, no, Nop, nop, () },
        0xAD => regular!{z,  8, no, Nop, nop, () },
        0xAE => regular!{z,  8, no, Nop, nop, () },
        0xAF => regular!{z,  8, no, Nop, nop, () },

        0xB0 => regular!{z,  0, mem, Ldir, ldir, () },
        0xB1 => regular!{z,  0, mem, Cpir, cpir, () },
        0xB2 => regular!{z,  0,  io, Inir, inir, () },
        0xB3 => regular!{z,  0,  io, Otir, otir, () },

        0xB4 => regular!{z,  8, no, Nop, nop, () },
        0xB5 => regular!{z,  8, no, Nop, nop, () },
        0xB6 => regular!{z,  8, no, Nop, nop, () },
        0xB7 => regular!{z,  8, no, Nop, nop, () },

        0xB8 => regular!{z,  0, mem, Lddr, lddr, () },
        0xB9 => regular!{z,  0, mem, Cpdr, cpdr, () },
        0xBA => regular!{z,  0,  io, Indr, indr, () },
        0xBB => regular!{z,  0,  io, Otdr, otdr, () },

        0xBC => regular!{z,  8, no, Nop, nop, () },
        0xBD => regular!{z,  8, no, Nop, nop, () },
        0xBE => regular!{z,  8, no, Nop, nop, () },
        0xBF => regular!{z,  8, no, Nop, nop, () },
        0xC0 => regular!{z,  8, no, Nop, nop, () },
        0xC1 => regular!{z,  8, no, Nop, nop, () },
        0xC2 => regular!{z,  8, no, Nop, nop, () },
        0xC3 => regular!{z,  8, no, Nop, nop, () },
        0xC4 => regular!{z,  8, no, Nop, nop, () },
        0xC5 => regular!{z,  8, no, Nop, nop, () },
        0xC6 => regular!{z,  8, no, Nop, nop, () },
        0xC7 => regular!{z,  8, no, Nop, nop, () },
        0xC8 => regular!{z,  8, no, Nop, nop, () },
        0xC9 => regular!{z,  8, no, Nop, nop, () },
        0xCA => regular!{z,  8, no, Nop, nop, () },
        0xCB => regular!{z,  8, no, Nop, nop, () },
        0xCC => regular!{z,  8, no, Nop, nop, () },
        0xCD => regular!{z,  8, no, Nop, nop, () },
        0xCE => regular!{z,  8, no, Nop, nop, () },
        0xCF => regular!{z,  8, no, Nop, nop, () },
        0xD0 => regular!{z,  8, no, Nop, nop, () },
        0xD1 => regular!{z,  8, no, Nop, nop, () },
        0xD2 => regular!{z,  8, no, Nop, nop, () },
        0xD3 => regular!{z,  8, no, Nop, nop, () },
        0xD4 => regular!{z,  8, no, Nop, nop, () },
        0xD5 => regular!{z,  8, no, Nop, nop, () },
        0xD6 => regular!{z,  8, no, Nop, nop, () },
        0xD7 => regular!{z,  8, no, Nop, nop, () },
        0xD8 => regular!{z,  8, no, Nop, nop, () },
        0xD9 => regular!{z,  8, no, Nop, nop, () },
        0xDA => regular!{z,  8, no, Nop, nop, () },
        0xDB => regular!{z,  8, no, Nop, nop, () },
        0xDC => regular!{z,  8, no, Nop, nop, () },
        0xDD => regular!{z,  8, no, Nop, nop, () },
        0xDE => regular!{z,  8, no, Nop, nop, () },
        0xDF => regular!{z,  8, no, Nop, nop, () },
        0xE0 => regular!{z,  8, no, Nop, nop, () },
        0xE1 => regular!{z,  8, no, Nop, nop, () },
        0xE2 => regular!{z,  8, no, Nop, nop, () },
        0xE3 => regular!{z,  8, no, Nop, nop, () },
        0xE4 => regular!{z,  8, no, Nop, nop, () },
        0xE5 => regular!{z,  8, no, Nop, nop, () },
        0xE6 => regular!{z,  8, no, Nop, nop, () },
        0xE7 => regular!{z,  8, no, Nop, nop, () },
        0xE8 => regular!{z,  8, no, Nop, nop, () },
        0xE9 => regular!{z,  8, no, Nop, nop, () },
        0xEA => regular!{z,  8, no, Nop, nop, () },
        0xEB => regular!{z,  8, no, Nop, nop, () },
        0xEC => regular!{z,  8, no, Nop, nop, () },
        0xED => regular!{z,  8, no, Nop, nop, () },
        0xEE => regular!{z,  8, no, Nop, nop, () },
        0xEF => regular!{z,  8, no, Nop, nop, () },
        0xF0 => regular!{z,  8, no, Nop, nop, () },
        0xF1 => regular!{z,  8, no, Nop, nop, () },
        0xF2 => regular!{z,  8, no, Nop, nop, () },
        0xF3 => regular!{z,  8, no, Nop, nop, () },
        0xF4 => regular!{z,  8, no, Nop, nop, () },
        0xF5 => regular!{z,  8, no, Nop, nop, () },
        0xF6 => regular!{z,  8, no, Nop, nop, () },
        0xF7 => regular!{z,  8, no, Nop, nop, () },
        0xF8 => regular!{z,  8, no, Nop, nop, () },
        0xF9 => regular!{z,  8, no, Nop, nop, () },
        0xFA => regular!{z,  8, no, Nop, nop, () },
        0xFB => regular!{z,  8, no, Nop, nop, () },
        0xFC => regular!{z,  8, no, Nop, nop, () },
        0xFD => regular!{z,  8, no, Nop, nop, () },
        0xFE => regular!{z,  8, no, Nop, nop, () },
        0xFF => regular!{z,  8, no, Nop, nop, () },
        _ => unimplemented!(),
    }
}

pub fn cb<Z>(z: &mut Z)
where
    Z: Z80Emulator,
{
    use self::instruction_traits::*;

    use self::Reg16::*;
    use self::Reg8::*;

    let pc = z.read_pc();
    z.inc_pc();
    match pc {
        0x00 => regular!{z,  8, mem, Rlc, rlc, ([B]) },
        0x01 => regular!{z,  8, mem, Rlc, rlc, ([C]) },
        0x02 => regular!{z,  8, mem, Rlc, rlc, ([D]) },
        0x03 => regular!{z,  8, mem, Rlc, rlc, ([E]) },
        0x04 => regular!{z,  8, mem, Rlc, rlc, ([H]) },
        0x05 => regular!{z,  8, mem, Rlc, rlc, ([L]) },
        0x06 => regular!{z, 15, mem, Rlc, rlc, ([Address(HL)]) },
        0x07 => regular!{z,  8, mem, Rlc, rlc, ([A]) },
        0x08 => regular!{z,  8, mem, Rrc, rrc, ([B]) },
        0x09 => regular!{z,  8, mem, Rrc, rrc, ([C]) },
        0x0A => regular!{z,  8, mem, Rrc, rrc, ([D]) },
        0x0B => regular!{z,  8, mem, Rrc, rrc, ([E]) },
        0x0C => regular!{z,  8, mem, Rrc, rrc, ([H]) },
        0x0D => regular!{z,  8, mem, Rrc, rrc, ([L]) },
        0x0E => regular!{z, 15, mem, Rrc, rrc, ([Address(HL)]) },
        0x0F => regular!{z,  8, mem, Rrc, rrc, ([A]) },
        0x10 => regular!{z,  8, mem, Rl, rl, ([B]) },
        0x11 => regular!{z,  8, mem, Rl, rl, ([C]) },
        0x12 => regular!{z,  8, mem, Rl, rl, ([D]) },
        0x13 => regular!{z,  8, mem, Rl, rl, ([E]) },
        0x14 => regular!{z,  8, mem, Rl, rl, ([H]) },
        0x15 => regular!{z,  8, mem, Rl, rl, ([L]) },
        0x16 => regular!{z, 15, mem, Rl, rl, ([Address(HL)]) },
        0x17 => regular!{z,  8, mem, Rl, rl, ([A]) },
        0x18 => regular!{z,  8, mem, Rr, rr, ([B]) },
        0x19 => regular!{z,  8, mem, Rr, rr, ([C]) },
        0x1A => regular!{z,  8, mem, Rr, rr, ([D]) },
        0x1B => regular!{z,  8, mem, Rr, rr, ([E]) },
        0x1C => regular!{z,  8, mem, Rr, rr, ([H]) },
        0x1D => regular!{z,  8, mem, Rr, rr, ([L]) },
        0x1E => regular!{z, 15, mem, Rr, rr, ([Address(HL)]) },
        0x1F => regular!{z,  8, mem, Rr, rr, ([A]) },
        0x20 => regular!{z,  8, mem, Sla, sla, ([B]) },
        0x21 => regular!{z,  8, mem, Sla, sla, ([C]) },
        0x22 => regular!{z,  8, mem, Sla, sla, ([D]) },
        0x23 => regular!{z,  8, mem, Sla, sla, ([E]) },
        0x24 => regular!{z,  8, mem, Sla, sla, ([H]) },
        0x25 => regular!{z,  8, mem, Sla, sla, ([L]) },
        0x26 => regular!{z, 15, mem, Sla, sla, ([Address(HL)]) },
        0x27 => regular!{z,  8, mem, Sla, sla, ([A]) },
        0x28 => regular!{z,  8, mem, Sra, sra, ([B]) },
        0x29 => regular!{z,  8, mem, Sra, sra, ([C]) },
        0x2A => regular!{z,  8, mem, Sra, sra, ([D]) },
        0x2B => regular!{z,  8, mem, Sra, sra, ([E]) },
        0x2C => regular!{z,  8, mem, Sra, sra, ([H]) },
        0x2D => regular!{z,  8, mem, Sra, sra, ([L]) },
        0x2E => regular!{z, 15, mem, Sra, sra, ([Address(HL)]) },
        0x2F => regular!{z,  8, mem, Sra, sra, ([A]) },
        0x30 => regular!{z,  8, mem, Sll, sll, ([B]) },
        0x31 => regular!{z,  8, mem, Sll, sll, ([C]) },
        0x32 => regular!{z,  8, mem, Sll, sll, ([D]) },
        0x33 => regular!{z,  8, mem, Sll, sll, ([E]) },
        0x34 => regular!{z,  8, mem, Sll, sll, ([H]) },
        0x35 => regular!{z,  8, mem, Sll, sll, ([L]) },
        0x36 => regular!{z, 15, mem, Sll, sll, ([Address(HL)]) },
        0x37 => regular!{z,  8, mem, Sll, sll, ([A]) },
        0x38 => regular!{z,  8, mem, Srl, srl, ([B]) },
        0x39 => regular!{z,  8, mem, Srl, srl, ([C]) },
        0x3A => regular!{z,  8, mem, Srl, srl, ([D]) },
        0x3B => regular!{z,  8, mem, Srl, srl, ([E]) },
        0x3C => regular!{z,  8, mem, Srl, srl, ([H]) },
        0x3D => regular!{z,  8, mem, Srl, srl, ([L]) },
        0x3E => regular!{z, 15, mem, Srl, srl, ([Address(HL)]) },
        0x3F => regular!{z,  8, mem, Srl, srl, ([A]) },
        0x40 => regular!{z,  8, mem, Bit, bit, ([0] [B]) },
        0x41 => regular!{z,  8, mem, Bit, bit, ([0] [C]) },
        0x42 => regular!{z,  8, mem, Bit, bit, ([0] [D]) },
        0x43 => regular!{z,  8, mem, Bit, bit, ([0] [E]) },
        0x44 => regular!{z,  8, mem, Bit, bit, ([0] [H]) },
        0x45 => regular!{z,  8, mem, Bit, bit, ([0] [L]) },
        0x46 => regular!{z, 12, mem, Bit, bit, ([0] [Address(HL)]) },
        0x47 => regular!{z,  8, mem, Bit, bit, ([0] [A]) },
        0x48 => regular!{z,  8, mem, Bit, bit, ([1] [B]) },
        0x49 => regular!{z,  8, mem, Bit, bit, ([1] [C]) },
        0x4A => regular!{z,  8, mem, Bit, bit, ([1] [D]) },
        0x4B => regular!{z,  8, mem, Bit, bit, ([1] [E]) },
        0x4C => regular!{z,  8, mem, Bit, bit, ([1] [H]) },
        0x4D => regular!{z,  8, mem, Bit, bit, ([1] [L]) },
        0x4E => regular!{z, 12, mem, Bit, bit, ([1] [Address(HL)]) },
        0x4F => regular!{z,  8, mem, Bit, bit, ([1] [A]) },
        0x50 => regular!{z,  8, mem, Bit, bit, ([2] [B]) },
        0x51 => regular!{z,  8, mem, Bit, bit, ([2] [C]) },
        0x52 => regular!{z,  8, mem, Bit, bit, ([2] [D]) },
        0x53 => regular!{z,  8, mem, Bit, bit, ([2] [E]) },
        0x54 => regular!{z,  8, mem, Bit, bit, ([2] [H]) },
        0x55 => regular!{z,  8, mem, Bit, bit, ([2] [L]) },
        0x56 => regular!{z, 12, mem, Bit, bit, ([2] [Address(HL)]) },
        0x57 => regular!{z,  8, mem, Bit, bit, ([2] [A]) },
        0x58 => regular!{z,  8, mem, Bit, bit, ([3] [B]) },
        0x59 => regular!{z,  8, mem, Bit, bit, ([3] [C]) },
        0x5A => regular!{z,  8, mem, Bit, bit, ([3] [D]) },
        0x5B => regular!{z,  8, mem, Bit, bit, ([3] [E]) },
        0x5C => regular!{z,  8, mem, Bit, bit, ([3] [H]) },
        0x5D => regular!{z,  8, mem, Bit, bit, ([3] [L]) },
        0x5E => regular!{z, 12, mem, Bit, bit, ([3] [Address(HL)]) },
        0x5F => regular!{z,  8, mem, Bit, bit, ([3] [A]) },
        0x60 => regular!{z,  8, mem, Bit, bit, ([4] [B]) },
        0x61 => regular!{z,  8, mem, Bit, bit, ([4] [C]) },
        0x62 => regular!{z,  8, mem, Bit, bit, ([4] [D]) },
        0x63 => regular!{z,  8, mem, Bit, bit, ([4] [E]) },
        0x64 => regular!{z,  8, mem, Bit, bit, ([4] [H]) },
        0x65 => regular!{z,  8, mem, Bit, bit, ([4] [L]) },
        0x66 => regular!{z, 12, mem, Bit, bit, ([4] [Address(HL)]) },
        0x67 => regular!{z,  8, mem, Bit, bit, ([4] [A]) },
        0x68 => regular!{z,  8, mem, Bit, bit, ([5] [B]) },
        0x69 => regular!{z,  8, mem, Bit, bit, ([5] [C]) },
        0x6A => regular!{z,  8, mem, Bit, bit, ([5] [D]) },
        0x6B => regular!{z,  8, mem, Bit, bit, ([5] [E]) },
        0x6C => regular!{z,  8, mem, Bit, bit, ([5] [H]) },
        0x6D => regular!{z,  8, mem, Bit, bit, ([5] [L]) },
        0x6E => regular!{z, 12, mem, Bit, bit, ([5] [Address(HL)]) },
        0x6F => regular!{z,  8, mem, Bit, bit, ([5] [A]) },
        0x70 => regular!{z,  8, mem, Bit, bit, ([6] [B]) },
        0x71 => regular!{z,  8, mem, Bit, bit, ([6] [C]) },
        0x72 => regular!{z,  8, mem, Bit, bit, ([6] [D]) },
        0x73 => regular!{z,  8, mem, Bit, bit, ([6] [E]) },
        0x74 => regular!{z,  8, mem, Bit, bit, ([6] [H]) },
        0x75 => regular!{z,  8, mem, Bit, bit, ([6] [L]) },
        0x76 => regular!{z, 12, mem, Bit, bit, ([6] [Address(HL)]) },
        0x77 => regular!{z,  8, mem, Bit, bit, ([6] [A]) },
        0x78 => regular!{z,  8, mem, Bit, bit, ([7] [B]) },
        0x79 => regular!{z,  8, mem, Bit, bit, ([7] [C]) },
        0x7A => regular!{z,  8, mem, Bit, bit, ([7] [D]) },
        0x7B => regular!{z,  8, mem, Bit, bit, ([7] [E]) },
        0x7C => regular!{z,  8, mem, Bit, bit, ([7] [H]) },
        0x7D => regular!{z,  8, mem, Bit, bit, ([7] [L]) },
        0x7E => regular!{z, 12, mem, Bit, bit, ([7] [Address(HL)]) },
        0x7F => regular!{z,  8, mem, Bit, bit, ([7] [A]) },
        0x80 => regular!{z,  8, mem, Res, res, ([0] [B]) },
        0x81 => regular!{z,  8, mem, Res, res, ([0] [C]) },
        0x82 => regular!{z,  8, mem, Res, res, ([0] [D]) },
        0x83 => regular!{z,  8, mem, Res, res, ([0] [E]) },
        0x84 => regular!{z,  8, mem, Res, res, ([0] [H]) },
        0x85 => regular!{z,  8, mem, Res, res, ([0] [L]) },
        0x86 => regular!{z, 15, mem, Res, res, ([0] [Address(HL)]) },
        0x87 => regular!{z,  8, mem, Res, res, ([0] [A]) },
        0x88 => regular!{z,  8, mem, Res, res, ([1] [B]) },
        0x89 => regular!{z,  8, mem, Res, res, ([1] [C]) },
        0x8A => regular!{z,  8, mem, Res, res, ([1] [D]) },
        0x8B => regular!{z,  8, mem, Res, res, ([1] [E]) },
        0x8C => regular!{z,  8, mem, Res, res, ([1] [H]) },
        0x8D => regular!{z,  8, mem, Res, res, ([1] [L]) },
        0x8E => regular!{z, 15, mem, Res, res, ([1] [Address(HL)]) },
        0x8F => regular!{z,  8, mem, Res, res, ([1] [A]) },
        0x90 => regular!{z,  8, mem, Res, res, ([2] [B]) },
        0x91 => regular!{z,  8, mem, Res, res, ([2] [C]) },
        0x92 => regular!{z,  8, mem, Res, res, ([2] [D]) },
        0x93 => regular!{z,  8, mem, Res, res, ([2] [E]) },
        0x94 => regular!{z,  8, mem, Res, res, ([2] [H]) },
        0x95 => regular!{z,  8, mem, Res, res, ([2] [L]) },
        0x96 => regular!{z, 15, mem, Res, res, ([2] [Address(HL)]) },
        0x97 => regular!{z,  8, mem, Res, res, ([2] [A]) },
        0x98 => regular!{z,  8, mem, Res, res, ([3] [B]) },
        0x99 => regular!{z,  8, mem, Res, res, ([3] [C]) },
        0x9A => regular!{z,  8, mem, Res, res, ([3] [D]) },
        0x9B => regular!{z,  8, mem, Res, res, ([3] [E]) },
        0x9C => regular!{z,  8, mem, Res, res, ([3] [H]) },
        0x9D => regular!{z,  8, mem, Res, res, ([3] [L]) },
        0x9E => regular!{z, 15, mem, Res, res, ([3] [Address(HL)]) },
        0x9F => regular!{z,  8, mem, Res, res, ([3] [A]) },
        0xA0 => regular!{z,  8, mem, Res, res, ([4] [B]) },
        0xA1 => regular!{z,  8, mem, Res, res, ([4] [C]) },
        0xA2 => regular!{z,  8, mem, Res, res, ([4] [D]) },
        0xA3 => regular!{z,  8, mem, Res, res, ([4] [E]) },
        0xA4 => regular!{z,  8, mem, Res, res, ([4] [H]) },
        0xA5 => regular!{z,  8, mem, Res, res, ([4] [L]) },
        0xA6 => regular!{z, 15, mem, Res, res, ([4] [Address(HL)]) },
        0xA7 => regular!{z,  8, mem, Res, res, ([4] [A]) },
        0xA8 => regular!{z,  8, mem, Res, res, ([5] [B]) },
        0xA9 => regular!{z,  8, mem, Res, res, ([5] [C]) },
        0xAA => regular!{z,  8, mem, Res, res, ([5] [D]) },
        0xAB => regular!{z,  8, mem, Res, res, ([5] [E]) },
        0xAC => regular!{z,  8, mem, Res, res, ([5] [H]) },
        0xAD => regular!{z,  8, mem, Res, res, ([5] [L]) },
        0xAE => regular!{z, 15, mem, Res, res, ([5] [Address(HL)]) },
        0xAF => regular!{z,  8, mem, Res, res, ([5] [A]) },
        0xB0 => regular!{z,  8, mem, Res, res, ([6] [B]) },
        0xB1 => regular!{z,  8, mem, Res, res, ([6] [C]) },
        0xB2 => regular!{z,  8, mem, Res, res, ([6] [D]) },
        0xB3 => regular!{z,  8, mem, Res, res, ([6] [E]) },
        0xB4 => regular!{z,  8, mem, Res, res, ([6] [H]) },
        0xB5 => regular!{z,  8, mem, Res, res, ([6] [L]) },
        0xB6 => regular!{z, 15, mem, Res, res, ([6] [Address(HL)]) },
        0xB7 => regular!{z,  8, mem, Res, res, ([6] [A]) },
        0xB8 => regular!{z,  8, mem, Res, res, ([7] [B]) },
        0xB9 => regular!{z,  8, mem, Res, res, ([7] [C]) },
        0xBA => regular!{z,  8, mem, Res, res, ([7] [D]) },
        0xBB => regular!{z,  8, mem, Res, res, ([7] [E]) },
        0xBC => regular!{z,  8, mem, Res, res, ([7] [H]) },
        0xBD => regular!{z,  8, mem, Res, res, ([7] [L]) },
        0xBE => regular!{z, 15, mem, Res, res, ([7] [Address(HL)]) },
        0xBF => regular!{z,  8, mem, Res, res, ([7] [A]) },
        0xC0 => regular!{z,  8, mem, Set, set, ([0] [B]) },
        0xC1 => regular!{z,  8, mem, Set, set, ([0] [C]) },
        0xC2 => regular!{z,  8, mem, Set, set, ([0] [D]) },
        0xC3 => regular!{z,  8, mem, Set, set, ([0] [E]) },
        0xC4 => regular!{z,  8, mem, Set, set, ([0] [H]) },
        0xC5 => regular!{z,  8, mem, Set, set, ([0] [L]) },
        0xC6 => regular!{z, 15, mem, Set, set, ([0] [Address(HL)]) },
        0xC7 => regular!{z,  8, mem, Set, set, ([0] [A]) },
        0xC8 => regular!{z,  8, mem, Set, set, ([1] [B]) },
        0xC9 => regular!{z,  8, mem, Set, set, ([1] [C]) },
        0xCA => regular!{z,  8, mem, Set, set, ([1] [D]) },
        0xCB => regular!{z,  8, mem, Set, set, ([1] [E]) },
        0xCC => regular!{z,  8, mem, Set, set, ([1] [H]) },
        0xCD => regular!{z,  8, mem, Set, set, ([1] [L]) },
        0xCE => regular!{z, 15, mem, Set, set, ([1] [Address(HL)]) },
        0xCF => regular!{z,  8, mem, Set, set, ([1] [A]) },
        0xD0 => regular!{z,  8, mem, Set, set, ([2] [B]) },
        0xD1 => regular!{z,  8, mem, Set, set, ([2] [C]) },
        0xD2 => regular!{z,  8, mem, Set, set, ([2] [D]) },
        0xD3 => regular!{z,  8, mem, Set, set, ([2] [E]) },
        0xD4 => regular!{z,  8, mem, Set, set, ([2] [H]) },
        0xD5 => regular!{z,  8, mem, Set, set, ([2] [L]) },
        0xD6 => regular!{z, 15, mem, Set, set, ([2] [Address(HL)]) },
        0xD7 => regular!{z,  8, mem, Set, set, ([2] [A]) },
        0xD8 => regular!{z,  8, mem, Set, set, ([3] [B]) },
        0xD9 => regular!{z,  8, mem, Set, set, ([3] [C]) },
        0xDA => regular!{z,  8, mem, Set, set, ([3] [D]) },
        0xDB => regular!{z,  8, mem, Set, set, ([3] [E]) },
        0xDC => regular!{z,  8, mem, Set, set, ([3] [H]) },
        0xDD => regular!{z,  8, mem, Set, set, ([3] [L]) },
        0xDE => regular!{z, 15, mem, Set, set, ([3] [Address(HL)]) },
        0xDF => regular!{z,  8, mem, Set, set, ([3] [A]) },
        0xE0 => regular!{z,  8, mem, Set, set, ([4] [B]) },
        0xE1 => regular!{z,  8, mem, Set, set, ([4] [C]) },
        0xE2 => regular!{z,  8, mem, Set, set, ([4] [D]) },
        0xE3 => regular!{z,  8, mem, Set, set, ([4] [E]) },
        0xE4 => regular!{z,  8, mem, Set, set, ([4] [H]) },
        0xE5 => regular!{z,  8, mem, Set, set, ([4] [L]) },
        0xE6 => regular!{z, 15, mem, Set, set, ([4] [Address(HL)]) },
        0xE7 => regular!{z,  8, mem, Set, set, ([4] [A]) },
        0xE8 => regular!{z,  8, mem, Set, set, ([5] [B]) },
        0xE9 => regular!{z,  8, mem, Set, set, ([5] [C]) },
        0xEA => regular!{z,  8, mem, Set, set, ([5] [D]) },
        0xEB => regular!{z,  8, mem, Set, set, ([5] [E]) },
        0xEC => regular!{z,  8, mem, Set, set, ([5] [H]) },
        0xED => regular!{z,  8, mem, Set, set, ([5] [L]) },
        0xEE => regular!{z, 15, mem, Set, set, ([5] [Address(HL)]) },
        0xEF => regular!{z,  8, mem, Set, set, ([5] [A]) },
        0xF0 => regular!{z,  8, mem, Set, set, ([6] [B]) },
        0xF1 => regular!{z,  8, mem, Set, set, ([6] [C]) },
        0xF2 => regular!{z,  8, mem, Set, set, ([6] [D]) },
        0xF3 => regular!{z,  8, mem, Set, set, ([6] [E]) },
        0xF4 => regular!{z,  8, mem, Set, set, ([6] [H]) },
        0xF5 => regular!{z,  8, mem, Set, set, ([6] [L]) },
        0xF6 => regular!{z, 15, mem, Set, set, ([6] [Address(HL)]) },
        0xF7 => regular!{z,  8, mem, Set, set, ([6] [A]) },
        0xF8 => regular!{z,  8, mem, Set, set, ([7] [B]) },
        0xF9 => regular!{z,  8, mem, Set, set, ([7] [C]) },
        0xFA => regular!{z,  8, mem, Set, set, ([7] [D]) },
        0xFB => regular!{z,  8, mem, Set, set, ([7] [E]) },
        0xFC => regular!{z,  8, mem, Set, set, ([7] [H]) },
        0xFD => regular!{z,  8, mem, Set, set, ([7] [L]) },
        0xFE => regular!{z, 15, mem, Set, set, ([7] [Address(HL)]) },
        0xFF => regular!{z,  8, mem, Set, set, ([7] [A]) },
        _ => unimplemented!(),
    }
}

pub fn dd<Z>(z: &mut Z)
where
    Z: Z80Emulator,
{
    use self::instruction_traits::*;

    use self::ConditionCode::*;
    use self::Reg16::*;
    use self::Reg8::*;

    let pc = z.read_pc();
    z.inc_pc();
    match pc {
        0x00 => regular!{z,        4,  no,    Nop,    nop, () },
        0x01 => nn_inst!{z,   nn, 10, mem,   Ld16,   ld16, ([BC] [nn]) },
        0x02 => regular!{z,        7, mem,     Ld,     ld, ([Address(BC)] [A]) },
        0x03 => regular!{z,        6,  no,  Inc16,  inc16, ([BC]) },
        0x04 => regular!{z,        4, mem,    Inc,    inc, ([B]) },
        0x05 => regular!{z,        4, mem,    Dec,    dec, ([B]) },
        0x06 => n_inst! {z,    n,  7, mem,     Ld,     ld, ([B] [n]) },
        0x07 => regular!{z,        4, mem,   Rlca,   rlca, () },
        0x08 => regular!{z,        4, mem,     Ex,     ex, ([AF] [AF0]) },
        0x09 => regular!{z,       11,  no,  Add16,  add16, ([IX] [BC]) },
        0x0A => regular!{z,        7, mem,     Ld,     ld, ([A] [Address(BC)]) },
        0x0B => regular!{z,        6,  no,  Dec16,  dec16, ([BC]) },
        0x0C => regular!{z,        4, mem,    Inc,    inc, ([C]) },
        0x0D => regular!{z,        4, mem,    Dec,    dec, ([C]) },
        0x0E => n_inst! {z,    n,  7, mem,     Ld,     ld, ([C] [n]) },
        0x0F => regular!{z,        4, mem,   Rrca,   rrca, () },
        0x10 => e_inst! {z,    e,  8,  no,   Djnz,   djnz, ([e]) },
        0x11 => nn_inst!{z,   nn, 10, mem,   Ld16,   ld16, ([DE] [nn]) },
        0x12 => regular!{z,        7, mem,     Ld,     ld, ([Address(DE)] [A]) },
        0x13 => regular!{z,        6,  no,  Inc16,  inc16, ([DE]) },
        0x14 => regular!{z,        4, mem,    Inc,    inc, ([D]) },
        0x15 => regular!{z,        4, mem,    Dec,    dec, ([D]) },
        0x16 => n_inst! {z,    n,  7, mem,     Ld,     ld, ([D] [n]) },
        0x17 => regular!{z,        4, mem,    Rla,    rla, () },
        0x18 => e_inst! {z,    e, 12,  no,     Jr,     jr, ([e]) },
        0x19 => regular!{z,       11,  no,  Add16,  add16, ([IX] [DE]) },
        0x1A => regular!{z,        7, mem,     Ld,     ld, ([A] [Address(DE)]) },
        0x1B => regular!{z,        6,  no,  Dec16,  dec16, ([DE]) },
        0x1C => regular!{z,        4, mem,    Inc,    inc, ([E]) },
        0x1D => regular!{z,        4, mem,    Dec,    dec, ([E]) },
        0x1E => n_inst! {z,    n,  7, mem,     Ld,     ld, ([E] [n]) },
        0x1F => regular!{z,        4, mem,    Rra,    rra, () },
        0x20 => e_inst! {z,    e,  0,  no,   Jrcc,   jrcc, ([NZcc] [e]) },
        0x21 => nn_inst!{z,   nn, 10, mem,   Ld16,   ld16, ([IX] [nn]) },
        0x22 => nn_inst!{z,   nn, 16, mem,   Ld16,   ld16, ([Address(nn)] [IX]) },
        0x23 => regular!{z,        6,  no,  Inc16,  inc16, ([IX]) },
        0x24 => regular!{z,        4, mem,    Inc,    inc, ([IXH]) },
        0x25 => regular!{z,        4, mem,    Dec,    dec, ([IXH]) },
        0x26 => n_inst! {z,    n,  7, mem,     Ld,     ld, ([IXH] [n]) },
        0x27 => regular!{z,        4,  no,    Daa,    daa, () },
        0x28 => e_inst! {z,    e,  0,  no,   Jrcc,   jrcc, ([Zcc] [e]) },
        0x29 => regular!{z,       11,  no,  Add16,  add16, ([IX] [IX]) },
        0x2A => nn_inst!{z,   nn, 16, mem,   Ld16,   ld16, ([IX] [Address(nn)]) },
        0x2B => regular!{z,        6,  no,  Dec16,  dec16, ([IX]) },
        0x2C => regular!{z,        4, mem,    Inc,    inc, ([IXL]) },
        0x2D => regular!{z,        4, mem,    Dec,    dec, ([IXL]) },
        0x2E => n_inst! {z,    n,  7, mem,     Ld,     ld, ([IXL] [n]) },
        0x2F => regular!{z,        4,  no,    Cpl,    cpl, () },
        0x30 => e_inst! {z,    e,  0,  no,   Jrcc,   jrcc, ([NCcc] [e]) },
        0x31 => nn_inst!{z,   nn, 10, mem,   Ld16,   ld16, ([SP] [nn]) },
        0x32 => nn_inst!{z,   nn, 13, mem,     Ld,     ld, ([Address(nn)] [A]) },
        0x33 => regular!{z,        6,  no,  Inc16,  inc16, ([SP]) },
        0x34 => e_inst! {z,    d, 19, mem,    Inc,    inc,   ([Shift(IX, d)]) },
        0x35 => e_inst! {z,    d, 19, mem,    Dec,    dec,   ([Shift(IX, d)]) },
        0x36 => en_inst!{z, d, n, 15, mem,     Ld,     ld,    ([Shift(IX, d)] [n]) },
        0x37 => regular!{z,        4,  no,    Scf,    scf, () },
        0x38 => e_inst! {z,    e,  0,  no,   Jrcc,   jrcc, ([Ccc] [e]) },
        0x39 => regular!{z,       11,  no,  Add16,  add16, ([IX] [SP]) },
        0x3A => nn_inst!{z,   nn, 13, mem,     Ld,     ld, ([A] [Address(nn)]) },
        0x3B => regular!{z,        6,  no,  Dec16,  dec16, ([SP]) },
        0x3C => regular!{z,        4, mem,    Inc,    inc, ([A]) },
        0x3D => regular!{z,        4, mem,    Dec,    dec, ([A]) },
        0x3E => n_inst! {z,    n,  7, mem,     Ld,     ld, ([A] [n]) },
        0x3F => regular!{z,        4,  no,    Ccf,    ccf, () },
        0x40 => regular!{z,        4, mem,     Ld,     ld, ([B] [B]) },
        0x41 => regular!{z,        4, mem,     Ld,     ld, ([B] [C]) },
        0x42 => regular!{z,        4, mem,     Ld,     ld, ([B] [D]) },
        0x43 => regular!{z,        4, mem,     Ld,     ld, ([B] [E]) },
        0x44 => regular!{z,        4, mem,     Ld,     ld, ([B] [IXH]) },
        0x45 => regular!{z,        4, mem,     Ld,     ld, ([B] [IXL]) },
        0x46 => e_inst! {z,    d, 15, mem,     Ld,     ld,    ([B] [Shift(IX, d)]) },
        0x47 => regular!{z,        4, mem,     Ld,     ld, ([B] [A]) },
        0x48 => regular!{z,        4, mem,     Ld,     ld, ([C] [B]) },
        0x49 => regular!{z,        4, mem,     Ld,     ld, ([C] [C]) },
        0x4A => regular!{z,        4, mem,     Ld,     ld, ([C] [D]) },
        0x4B => regular!{z,        4, mem,     Ld,     ld, ([C] [E]) },
        0x4C => regular!{z,        4, mem,     Ld,     ld, ([C] [IXH]) },
        0x4D => regular!{z,        4, mem,     Ld,     ld, ([C] [IXL]) },
        0x4E => e_inst! {z,    d, 15, mem,     Ld,     ld, ([C] [Shift(IX, d)]) },
        0x4F => regular!{z,        4, mem,     Ld,     ld, ([C] [A]) },
        0x50 => regular!{z,        4, mem,     Ld,     ld, ([D] [B]) },
        0x51 => regular!{z,        4, mem,     Ld,     ld, ([D] [C]) },
        0x52 => regular!{z,        4, mem,     Ld,     ld, ([D] [D]) },
        0x53 => regular!{z,        4, mem,     Ld,     ld, ([D] [E]) },
        0x54 => regular!{z,        4, mem,     Ld,     ld, ([D] [IXH]) },
        0x55 => regular!{z,        4, mem,     Ld,     ld, ([D] [IXL]) },
        0x56 => e_inst! {z,    d, 15, mem,     Ld,     ld, ([D] [Shift(IX, d)]) },
        0x57 => regular!{z,        4, mem,     Ld,     ld, ([D] [A]) },
        0x58 => regular!{z,        4, mem,     Ld,     ld, ([E] [B]) },
        0x59 => regular!{z,        4, mem,     Ld,     ld, ([E] [C]) },
        0x5A => regular!{z,        4, mem,     Ld,     ld, ([E] [D]) },
        0x5B => regular!{z,        4, mem,     Ld,     ld, ([E] [E]) },
        0x5C => regular!{z,        4, mem,     Ld,     ld, ([E] [IXH]) },
        0x5D => regular!{z,        4, mem,     Ld,     ld, ([E] [IXL]) },
        0x5E => e_inst! {z,    d, 15, mem,     Ld,     ld, ([E] [Shift(IX, d)]) },
        0x5F => regular!{z,        4, mem,     Ld,     ld, ([E] [A]) },
        0x60 => regular!{z,        4, mem,     Ld,     ld, ([IXH] [B]) },
        0x61 => regular!{z,        4, mem,     Ld,     ld, ([IXH] [C]) },
        0x62 => regular!{z,        4, mem,     Ld,     ld, ([IXH] [D]) },
        0x63 => regular!{z,        4, mem,     Ld,     ld, ([IXH] [E]) },
        0x64 => regular!{z,        4, mem,     Ld,     ld, ([IXH] [IXH]) },
        0x65 => regular!{z,        4, mem,     Ld,     ld, ([IXH] [IXL]) },
        0x66 => e_inst! {z,    d, 15, mem,     Ld,     ld, ([H] [Shift(IX, d)]) },
        0x67 => regular!{z,        4, mem,     Ld,     ld, ([IXH] [A]) },
        0x68 => regular!{z,        4, mem,     Ld,     ld, ([IXL] [B]) },
        0x69 => regular!{z,        4, mem,     Ld,     ld, ([IXL] [C]) },
        0x6A => regular!{z,        4, mem,     Ld,     ld, ([IXL] [D]) },
        0x6B => regular!{z,        4, mem,     Ld,     ld, ([IXL] [E]) },
        0x6C => regular!{z,        4, mem,     Ld,     ld, ([IXL] [IXH]) },
        0x6D => regular!{z,        4, mem,     Ld,     ld, ([IXL] [IXL]) },
        0x6E => e_inst! {z,    d, 15, mem,     Ld,     ld, ([L] [Shift(IX, d)]) },
        0x6F => regular!{z,        4, mem,     Ld,     ld, ([IXL] [A]) },
        0x70 => e_inst! {z,    d, 15, mem,     Ld,     ld, ([Shift(IX, d)] [B]) },
        0x71 => e_inst! {z,    d, 15, mem,     Ld,     ld, ([Shift(IX, d)] [C]) },
        0x72 => e_inst! {z,    d, 15, mem,     Ld,     ld, ([Shift(IX, d)] [D]) },
        0x73 => e_inst! {z,    d, 15, mem,     Ld,     ld, ([Shift(IX, d)] [E]) },
        0x74 => e_inst! {z,    d, 15, mem,     Ld,     ld, ([Shift(IX, d)] [H]) },
        0x75 => e_inst! {z,    d, 15, mem,     Ld,     ld, ([Shift(IX, d)] [L]) },
        0x76 => regular!{z,        4,  no,   Halt,   halt, () },
        0x77 => e_inst! {z,    d, 15, mem,     Ld,     ld, ([Shift(IX, d)] [A]) },
        0x78 => regular!{z,        4, mem,     Ld,     ld, ([A] [B]) },
        0x79 => regular!{z,        4, mem,     Ld,     ld, ([A] [C]) },
        0x7A => regular!{z,        4, mem,     Ld,     ld, ([A] [D]) },
        0x7B => regular!{z,        4, mem,     Ld,     ld, ([A] [E]) },
        0x7C => regular!{z,        4, mem,     Ld,     ld, ([A] [IXH]) },
        0x7D => regular!{z,        4, mem,     Ld,     ld, ([A] [IXL]) },
        0x7E => e_inst! {z,    d, 15, mem,     Ld,     ld, ([A] [Shift(IX, d)]) },
        0x7F => regular!{z,        4, mem,     Ld,     ld, ([A] [A]) },
        0x80 => regular!{z,        4, mem,    Add,    add, ([A] [B]) },
        0x81 => regular!{z,        4, mem,    Add,    add, ([A] [C]) },
        0x82 => regular!{z,        4, mem,    Add,    add, ([A] [D]) },
        0x83 => regular!{z,        4, mem,    Add,    add, ([A] [E]) },
        0x84 => regular!{z,        4, mem,    Add,    add, ([A] [IXH]) },
        0x85 => regular!{z,        4, mem,    Add,    add, ([A] [IXL]) },
        0x86 => e_inst! {z,    d, 15, mem,    Add,    add, ([A] [Shift(IX, d)]) },
        0x87 => regular!{z,        4, mem,    Add,    add, ([A] [A]) },
        0x88 => regular!{z,        4, mem,    Adc,    adc, ([A] [B]) },
        0x89 => regular!{z,        4, mem,    Adc,    adc, ([A] [C]) },
        0x8A => regular!{z,        4, mem,    Adc,    adc, ([A] [D]) },
        0x8B => regular!{z,        4, mem,    Adc,    adc, ([A] [E]) },
        0x8C => regular!{z,        4, mem,    Adc,    adc, ([A] [IXH]) },
        0x8D => regular!{z,        4, mem,    Adc,    adc, ([A] [IXL]) },
        0x8E => e_inst! {z,    d, 15, mem,    Adc,    adc, ([A] [Shift(IX, d)]) },
        0x8F => regular!{z,        4, mem,    Adc,    adc, ([A] [A]) },
        0x90 => regular!{z,        4, mem,    Sub,    sub, ([A] [B]) },
        0x91 => regular!{z,        4, mem,    Sub,    sub, ([A] [C]) },
        0x92 => regular!{z,        4, mem,    Sub,    sub, ([A] [D]) },
        0x93 => regular!{z,        4, mem,    Sub,    sub, ([A] [E]) },
        0x94 => regular!{z,        4, mem,    Sub,    sub, ([A] [IXH]) },
        0x95 => regular!{z,        4, mem,    Sub,    sub, ([A] [IXL]) },
        0x96 => e_inst! {z,    d, 15, mem,    Sub,    sub, ([A] [Shift(IX, d)]) },
        0x97 => regular!{z,        4, mem,    Sub,    sub, ([A] [A]) },
        0x98 => regular!{z,        4, mem,    Sbc,    sbc, ([A] [B]) },
        0x99 => regular!{z,        4, mem,    Sbc,    sbc, ([A] [C]) },
        0x9A => regular!{z,        4, mem,    Sbc,    sbc, ([A] [D]) },
        0x9B => regular!{z,        4, mem,    Sbc,    sbc, ([A] [E]) },
        0x9C => regular!{z,        4, mem,    Sbc,    sbc, ([A] [IXH]) },
        0x9D => regular!{z,        4, mem,    Sbc,    sbc, ([A] [IXL]) },
        0x9E => e_inst! {z,    d, 15, mem,    Sbc,    sbc, ([A] [Shift(IX, d)]) },
        0x9F => regular!{z,        4, mem,    Sbc,    sbc, ([A] [A]) },
        0xA0 => regular!{z,        4, mem,    And,    and, ([B]) },
        0xA1 => regular!{z,        4, mem,    And,    and, ([C]) },
        0xA2 => regular!{z,        4, mem,    And,    and, ([D]) },
        0xA3 => regular!{z,        4, mem,    And,    and, ([E]) },
        0xA4 => regular!{z,        4, mem,    And,    and, ([IXH]) },
        0xA5 => regular!{z,        4, mem,    And,    and, ([IXL]) },
        0xA6 => e_inst! {z,    d, 15, mem,    And,    and, ([Shift(IX, d)]) },
        0xA7 => regular!{z,        4, mem,    And,    and, ([A]) },
        0xA8 => regular!{z,        4, mem,    Xor,    xor, ([B]) },
        0xA9 => regular!{z,        4, mem,    Xor,    xor, ([C]) },
        0xAA => regular!{z,        4, mem,    Xor,    xor, ([D]) },
        0xAB => regular!{z,        4, mem,    Xor,    xor, ([E]) },
        0xAC => regular!{z,        4, mem,    Xor,    xor, ([IXH]) },
        0xAD => regular!{z,        4, mem,    Xor,    xor, ([IXL]) },
        0xAE => e_inst! {z,    d, 15, mem,    Xor,    xor, ([Shift(IX, d)]) },
        0xAF => regular!{z,        4, mem,    Xor,    xor, ([A]) },
        0xB0 => regular!{z,        4, mem,     Or,     or, ([B]) },
        0xB1 => regular!{z,        4, mem,     Or,     or, ([C]) },
        0xB2 => regular!{z,        4, mem,     Or,     or, ([D]) },
        0xB3 => regular!{z,        4, mem,     Or,     or, ([E]) },
        0xB4 => regular!{z,        4, mem,     Or,     or, ([IXH]) },
        0xB5 => regular!{z,        4, mem,     Or,     or, ([IXL]) },
        0xB6 => e_inst! {z,    d, 15, mem,     Or,     or, ([Shift(IX, d)]) },
        0xB7 => regular!{z,        4, mem,     Or,     or, ([A]) },
        0xB8 => regular!{z,        4, mem,     Cp,     cp, ([B]) },
        0xB9 => regular!{z,        4, mem,     Cp,     cp, ([C]) },
        0xBA => regular!{z,        4, mem,     Cp,     cp, ([D]) },
        0xBB => regular!{z,        4, mem,     Cp,     cp, ([E]) },
        0xBC => regular!{z,        4, mem,     Cp,     cp, ([IXH]) },
        0xBD => regular!{z,        4, mem,     Cp,     cp, ([IXL]) },
        0xBE => e_inst! {z,    d, 15, mem,     Cp,     cp, ([Shift(IX, d)]) },
        0xBF => regular!{z,        4, mem,     Cp,     cp, ([A]) },
        0xC0 => regular!{z,        5, mem,  Retcc,  retcc, ([NZcc]) },
        0xC1 => regular!{z,       10, mem,    Pop,    pop, ([BC]) },
        0xC2 => nn_inst!{z,   nn, 10,  no,   Jpcc,   jpcc, ([NZcc] [nn]) },
        0xC3 => nn_inst!{z,   nn, 10, mem,     Jp,     jp, ([nn]) },
        0xC4 => nn_inst!{z,   nn,  0, mem, Callcc, callcc, ([NZcc] [nn]) },
        0xC5 => regular!{z,       11, mem,   Push,   push, ([BC]) },
        0xC6 => n_inst!{z,     n,  7, mem,    Add,    add, ([A] [n]) },
        0xC7 => regular!{z,       11, mem,    Rst,    rst, ([0x00]) },
        0xC8 => regular!{z,        5, mem,  Retcc,  retcc, ([Zcc]) },
        0xC9 => regular!{z,       10, mem,    Ret,    ret, () },
        0xCA => nn_inst!{z,   nn, 10,  no,   Jpcc,   jpcc, ([Zcc] [nn]) },
        0xCB => regular!{z,        0,  no,   Ddcb,   ddcb, () },
        0xCC => nn_inst!{z,   nn,  0, mem, Callcc, callcc, ([Zcc] [nn]) },
        0xCD => nn_inst!{z,   nn, 17, mem,   Call,   call, ([nn]) },
        0xCE => n_inst! {z,    n,  7, mem,    Adc,    adc, ([A] [n]) },
        0xCF => regular!{z,       11, mem,    Rst,    rst, ([0x08]) },
        0xD0 => regular!{z,        5, mem,  Retcc,  retcc, ([NCcc]) },
        0xD1 => regular!{z,       10, mem,    Pop,    pop, ([DE]) },
        0xD2 => nn_inst!{z,   nn, 10,  no,   Jpcc,   jpcc, ([NCcc] [nn]) },
        0xD3 => n_inst! {z,    n, 11,  io,   OutN,  out_n, ([n ] [A]) },
        0xD4 => nn_inst!{z,   nn,  0, mem, Callcc, callcc, ([NCcc] [nn]) },
        0xD5 => regular!{z,       11, mem,   Push,   push, ([DE]) },
        0xD6 => n_inst! {z,    n,  7, mem,    Sub,    sub, ([A] [n]) },
        0xD7 => regular!{z,       11, mem,    Rst,    rst, ([0x10]) },
        0xD8 => regular!{z,        5, mem,  Retcc,  retcc, ([Ccc]) },
        0xD9 => regular!{z,        4,  no,    Exx,    exx, () },
        0xDA => nn_inst!{z,   nn, 10,  no,   Jpcc,   jpcc, ([Ccc] [nn]) },
        0xDB => n_inst! {z,    n, 11,  io,    InN,   in_n, ([A] [n]) },
        0xDC => nn_inst!{z,   nn,  0, mem, Callcc, callcc, ([Ccc] [nn]) },
        0xDD => regular!{z,        4,  no,     Dd,     dd, () },
        0xDE => n_inst! {z,    n,  7, mem,    Sbc,    sbc, ([A] [n]) },
        0xDF => regular!{z,       11, mem,    Rst,    rst, ([0x18]) },
        0xE0 => regular!{z,        5, mem,  Retcc,  retcc, ([POcc]) },
        0xE1 => regular!{z,       10, mem,    Pop,    pop, ([IX]) },
        0xE2 => nn_inst!{z,   nn, 10,  no,   Jpcc,   jpcc, ([POcc] [nn]) },
        0xE3 => regular!{z,       19, mem,     Ex,     ex, ([Address(SP)] [IX]) },
        0xE4 => nn_inst!{z,   nn,  0, mem, Callcc, callcc, ([POcc] [nn]) },
        0xE5 => regular!{z,       11, mem,   Push,   push, ([IX]) },
        0xE6 => n_inst! {z,    n,  7, mem,    And,    and, ([n]) },
        0xE7 => regular!{z,       11, mem,    Rst,    rst, ([0x20]) },
        0xE8 => regular!{z,        5, mem,  Retcc,  retcc, ([PEcc]) },
        0xE9 => regular!{z,        4, mem,     Jp,     jp, ([IX]) },
        0xEA => nn_inst!{z,   nn, 10,  no,   Jpcc,   jpcc, ([PEcc] [nn]) },
        0xEB => regular!{z,        4, mem,     Ex,     ex, ([DE] [HL]) },
        0xEC => nn_inst!{z,   nn,  0, mem, Callcc, callcc, ([PEcc] [nn]) },
        0xED => regular!{z,        4,  no,     Ed,     ed, () },
        0xEE => n_inst! {z,    n,  7, mem,    Xor,    xor, ([n]) },
        0xEF => regular!{z,       11, mem,    Rst,    rst, ([0x28]) },
        0xF0 => regular!{z,        5, mem,  Retcc,  retcc, ([Pcc]) },
        0xF1 => regular!{z,       10, mem,    Pop,    pop, ([AF]) },
        0xF2 => nn_inst!{z,   nn, 10,  no,   Jpcc,   jpcc, ([Pcc] [nn]) },
        0xF3 => regular!{z,        4,  no,     Di,     di, () },
        0xF4 => nn_inst!{z,   nn,  0, mem, Callcc, callcc, ([Pcc] [nn]) },
        0xF5 => regular!{z,       11, mem,   Push,   push, ([AF]) },
        0xF6 => n_inst! {z,    n,  7, mem,     Or,     or, ([n]) },
        0xF7 => regular!{z,       11, mem,    Rst,    rst, ([0x30]) },
        0xF8 => regular!{z,        5, mem,  Retcc,  retcc, ([Mcc]) },
        0xF9 => regular!{z,        6, mem,   Ld16,   ld16, ([SP] [IX]) },
        0xFA => nn_inst!{z,   nn, 10,  no,   Jpcc,   jpcc, ([Mcc] [nn]) },
        0xFB => regular!{z,        4,  no,     Ei,     ei, () },
        0xFC => nn_inst!{z,   nn,  0, mem, Callcc, callcc, ([Mcc] [nn]) },
        0xFD => regular!{z,        4,  no,     Fd,     fd, () },
        0xFE => n_inst! {z,    n,  7, mem,     Cp,     cp, ([n]) },
        0xFF => regular!{z,       11, mem,    Rst,    rst, ([0x38]) },
        _ => unimplemented!(),
    }
}

pub fn fd<Z>(z: &mut Z)
where
    Z: Z80Emulator,
{
    use self::instruction_traits::*;

    use self::ConditionCode::*;
    use self::Reg16::*;
    use self::Reg8::*;

    let pc = z.read_pc();
    z.inc_pc();
    match pc {
        0x00 => regular!{z,        4,  no,    Nop,    nop, () },
        0x01 => nn_inst!{z,   nn, 10, mem,   Ld16,   ld16, ([BC] [nn]) },
        0x02 => regular!{z,        7, mem,     Ld,     ld, ([Address(BC)] [A]) },
        0x03 => regular!{z,        6,  no,  Inc16,  inc16, ([BC]) },
        0x04 => regular!{z,        4, mem,    Inc,    inc, ([B]) },
        0x05 => regular!{z,        4, mem,    Dec,    dec, ([B]) },
        0x06 => n_inst! {z,    n,  7, mem,     Ld,     ld, ([B] [n]) },
        0x07 => regular!{z,        4, mem,   Rlca,   rlca, () },
        0x08 => regular!{z,        4, mem,     Ex,     ex, ([AF] [AF0]) },
        0x09 => regular!{z,       11,  no,  Add16,  add16, ([IY] [BC]) },
        0x0A => regular!{z,        7, mem,     Ld,     ld, ([A] [Address(BC)]) },
        0x0B => regular!{z,        6,  no,  Dec16,  dec16, ([BC]) },
        0x0C => regular!{z,        4, mem,    Inc,    inc, ([C]) },
        0x0D => regular!{z,        4, mem,    Dec,    dec, ([C]) },
        0x0E => n_inst! {z,    n,  7, mem,     Ld,     ld, ([C] [n]) },
        0x0F => regular!{z,        4, mem,   Rrca,   rrca, () },
        0x10 => e_inst! {z,    e,  8,  no,   Djnz,   djnz, ([e]) },
        0x11 => nn_inst!{z,   nn, 10, mem,   Ld16,   ld16, ([DE] [nn]) },
        0x12 => regular!{z,        7, mem,     Ld,     ld, ([Address(DE)] [A]) },
        0x13 => regular!{z,        6,  no,  Inc16,  inc16, ([DE]) },
        0x14 => regular!{z,        4, mem,    Inc,    inc, ([D]) },
        0x15 => regular!{z,        4, mem,    Dec,    dec, ([D]) },
        0x16 => n_inst! {z,    n,  7, mem,     Ld,     ld, ([D] [n]) },
        0x17 => regular!{z,        4, mem,    Rla,    rla, () },
        0x18 => e_inst! {z,    e, 12,  no,     Jr,     jr, ([e]) },
        0x19 => regular!{z,       11,  no,  Add16,  add16, ([IY] [DE]) },
        0x1A => regular!{z,        7, mem,     Ld,     ld, ([A] [Address(DE)]) },
        0x1B => regular!{z,        6,  no,  Dec16,  dec16, ([DE]) },
        0x1C => regular!{z,        4, mem,    Inc,    inc, ([E]) },
        0x1D => regular!{z,        4, mem,    Dec,    dec, ([E]) },
        0x1E => n_inst! {z,    n,  7, mem,     Ld,     ld, ([E] [n]) },
        0x1F => regular!{z,        4, mem,    Rra,    rra, () },
        0x20 => e_inst! {z,    e,  0,  no,   Jrcc,   jrcc, ([NZcc] [e]) },
        0x21 => nn_inst!{z,   nn, 10, mem,   Ld16,   ld16, ([IY] [nn]) },
        0x22 => nn_inst!{z,   nn, 16, mem,   Ld16,   ld16, ([Address(nn)] [IY]) },
        0x23 => regular!{z,        6,  no,  Inc16,  inc16, ([IY]) },
        0x24 => regular!{z,        4, mem,    Inc,    inc, ([IYH]) },
        0x25 => regular!{z,        4, mem,    Dec,    dec, ([IYH]) },
        0x26 => n_inst! {z,    n,  7, mem,     Ld,     ld, ([IYH] [n]) },
        0x27 => regular!{z,        4,  no,    Daa,    daa, () },
        0x28 => e_inst! {z,    e,  0,  no,   Jrcc,   jrcc, ([Zcc] [e]) },
        0x29 => regular!{z,       11,  no,  Add16,  add16, ([IY] [IY]) },
        0x2A => nn_inst!{z,   nn, 16, mem,   Ld16,   ld16, ([IY] [Address(nn)]) },
        0x2B => regular!{z,        6,  no,  Dec16,  dec16, ([IY]) },
        0x2C => regular!{z,        4, mem,    Inc,    inc, ([IYL]) },
        0x2D => regular!{z,        4, mem,    Dec,    dec, ([IYL]) },
        0x2E => n_inst! {z,    n,  7, mem,     Ld,     ld, ([IYL] [n]) },
        0x2F => regular!{z,        4,  no,    Cpl,    cpl, () },
        0x30 => e_inst! {z,    e,  0,  no,   Jrcc,   jrcc, ([NCcc] [e]) },
        0x31 => nn_inst!{z,   nn, 10, mem,   Ld16,   ld16, ([SP] [nn]) },
        0x32 => nn_inst!{z,   nn, 13, mem,     Ld,     ld, ([Address(nn)] [A]) },
        0x33 => regular!{z,        6,  no,  Inc16,  inc16, ([SP]) },
        0x34 => e_inst! {z,    d, 19, mem,    Inc,    inc,   ([Shift(IY, d)]) },
        0x35 => e_inst! {z,    d, 19, mem,    Dec,    dec,   ([Shift(IY, d)]) },
        0x36 => en_inst!{z, d, n, 15, mem,     Ld,     ld,    ([Shift(IY, d)] [n]) },
        0x37 => regular!{z,        4,  no,    Scf,    scf, () },
        0x38 => e_inst! {z,    e,  0,  no,   Jrcc,   jrcc, ([Ccc] [e]) },
        0x39 => regular!{z,       11,  no,  Add16,  add16, ([IY] [SP]) },
        0x3A => nn_inst!{z,   nn, 13, mem,     Ld,     ld, ([A] [Address(nn)]) },
        0x3B => regular!{z,        6,  no,  Dec16,  dec16, ([SP]) },
        0x3C => regular!{z,        4, mem,    Inc,    inc, ([A]) },
        0x3D => regular!{z,        4, mem,    Dec,    dec, ([A]) },
        0x3E => n_inst! {z,    n,  7, mem,     Ld,     ld, ([A] [n]) },
        0x3F => regular!{z,        4,  no,    Ccf,    ccf, () },
        0x40 => regular!{z,        4, mem,     Ld,     ld, ([B] [B]) },
        0x41 => regular!{z,        4, mem,     Ld,     ld, ([B] [C]) },
        0x42 => regular!{z,        4, mem,     Ld,     ld, ([B] [D]) },
        0x43 => regular!{z,        4, mem,     Ld,     ld, ([B] [E]) },
        0x44 => regular!{z,        4, mem,     Ld,     ld, ([B] [IYH]) },
        0x45 => regular!{z,        4, mem,     Ld,     ld, ([B] [IYL]) },
        0x46 => e_inst! {z,    d, 15, mem,     Ld,     ld,    ([B] [Shift(IY, d)]) },
        0x47 => regular!{z,        4, mem,     Ld,     ld, ([B] [A]) },
        0x48 => regular!{z,        4, mem,     Ld,     ld, ([C] [B]) },
        0x49 => regular!{z,        4, mem,     Ld,     ld, ([C] [C]) },
        0x4A => regular!{z,        4, mem,     Ld,     ld, ([C] [D]) },
        0x4B => regular!{z,        4, mem,     Ld,     ld, ([C] [E]) },
        0x4C => regular!{z,        4, mem,     Ld,     ld, ([C] [IYH]) },
        0x4D => regular!{z,        4, mem,     Ld,     ld, ([C] [IYL]) },
        0x4E => e_inst! {z,    d, 15, mem,     Ld,     ld, ([C] [Shift(IY, d)]) },
        0x4F => regular!{z,        4, mem,     Ld,     ld, ([C] [A]) },
        0x50 => regular!{z,        4, mem,     Ld,     ld, ([D] [B]) },
        0x51 => regular!{z,        4, mem,     Ld,     ld, ([D] [C]) },
        0x52 => regular!{z,        4, mem,     Ld,     ld, ([D] [D]) },
        0x53 => regular!{z,        4, mem,     Ld,     ld, ([D] [E]) },
        0x54 => regular!{z,        4, mem,     Ld,     ld, ([D] [IYH]) },
        0x55 => regular!{z,        4, mem,     Ld,     ld, ([D] [IYL]) },
        0x56 => e_inst! {z,    d, 15, mem,     Ld,     ld, ([D] [Shift(IY, d)]) },
        0x57 => regular!{z,        4, mem,     Ld,     ld, ([D] [A]) },
        0x58 => regular!{z,        4, mem,     Ld,     ld, ([E] [B]) },
        0x59 => regular!{z,        4, mem,     Ld,     ld, ([E] [C]) },
        0x5A => regular!{z,        4, mem,     Ld,     ld, ([E] [D]) },
        0x5B => regular!{z,        4, mem,     Ld,     ld, ([E] [E]) },
        0x5C => regular!{z,        4, mem,     Ld,     ld, ([E] [IYH]) },
        0x5D => regular!{z,        4, mem,     Ld,     ld, ([E] [IYL]) },
        0x5E => e_inst! {z,    d, 15, mem,     Ld,     ld, ([E] [Shift(IY, d)]) },
        0x5F => regular!{z,        4, mem,     Ld,     ld, ([E] [A]) },
        0x60 => regular!{z,        4, mem,     Ld,     ld, ([IYH] [B]) },
        0x61 => regular!{z,        4, mem,     Ld,     ld, ([IYH] [C]) },
        0x62 => regular!{z,        4, mem,     Ld,     ld, ([IYH] [D]) },
        0x63 => regular!{z,        4, mem,     Ld,     ld, ([IYH] [E]) },
        0x64 => regular!{z,        4, mem,     Ld,     ld, ([IYH] [IYH]) },
        0x65 => regular!{z,        4, mem,     Ld,     ld, ([IYH] [IYL]) },
        0x66 => e_inst! {z,    d, 15, mem,     Ld,     ld, ([H] [Shift(IY, d)]) },
        0x67 => regular!{z,        4, mem,     Ld,     ld, ([IYH] [A]) },
        0x68 => regular!{z,        4, mem,     Ld,     ld, ([IYL] [B]) },
        0x69 => regular!{z,        4, mem,     Ld,     ld, ([IYL] [C]) },
        0x6A => regular!{z,        4, mem,     Ld,     ld, ([IYL] [D]) },
        0x6B => regular!{z,        4, mem,     Ld,     ld, ([IYL] [E]) },
        0x6C => regular!{z,        4, mem,     Ld,     ld, ([IYL] [IYH]) },
        0x6D => regular!{z,        4, mem,     Ld,     ld, ([IYL] [IYL]) },
        0x6E => e_inst! {z,    d, 15, mem,     Ld,     ld, ([L] [Shift(IY, d)]) },
        0x6F => regular!{z,        4, mem,     Ld,     ld, ([IYL] [A]) },
        0x70 => e_inst! {z,    d, 15, mem,     Ld,     ld, ([Shift(IY, d)] [B]) },
        0x71 => e_inst! {z,    d, 15, mem,     Ld,     ld, ([Shift(IY, d)] [C]) },
        0x72 => e_inst! {z,    d, 15, mem,     Ld,     ld, ([Shift(IY, d)] [D]) },
        0x73 => e_inst! {z,    d, 15, mem,     Ld,     ld, ([Shift(IY, d)] [E]) },
        0x74 => e_inst! {z,    d, 15, mem,     Ld,     ld, ([Shift(IY, d)] [H]) },
        0x75 => e_inst! {z,    d, 15, mem,     Ld,     ld, ([Shift(IY, d)] [L]) },
        0x76 => regular!{z,        4,  no,   Halt,   halt, () },
        0x77 => e_inst! {z,    d, 15, mem,     Ld,     ld, ([Shift(IY, d)] [A]) },
        0x78 => regular!{z,        4, mem,     Ld,     ld, ([A] [B]) },
        0x79 => regular!{z,        4, mem,     Ld,     ld, ([A] [C]) },
        0x7A => regular!{z,        4, mem,     Ld,     ld, ([A] [D]) },
        0x7B => regular!{z,        4, mem,     Ld,     ld, ([A] [E]) },
        0x7C => regular!{z,        4, mem,     Ld,     ld, ([A] [IYH]) },
        0x7D => regular!{z,        4, mem,     Ld,     ld, ([A] [IYL]) },
        0x7E => e_inst! {z,    d, 15, mem,     Ld,     ld, ([A] [Shift(IY, d)]) },
        0x7F => regular!{z,        4, mem,     Ld,     ld, ([A] [A]) },
        0x80 => regular!{z,        4, mem,    Add,    add, ([A] [B]) },
        0x81 => regular!{z,        4, mem,    Add,    add, ([A] [C]) },
        0x82 => regular!{z,        4, mem,    Add,    add, ([A] [D]) },
        0x83 => regular!{z,        4, mem,    Add,    add, ([A] [E]) },
        0x84 => regular!{z,        4, mem,    Add,    add, ([A] [IYH]) },
        0x85 => regular!{z,        4, mem,    Add,    add, ([A] [IYL]) },
        0x86 => e_inst! {z,    d, 15, mem,    Add,    add, ([A] [Shift(IY, d)]) },
        0x87 => regular!{z,        4, mem,    Add,    add, ([A] [A]) },
        0x88 => regular!{z,        4, mem,    Adc,    adc, ([A] [B]) },
        0x89 => regular!{z,        4, mem,    Adc,    adc, ([A] [C]) },
        0x8A => regular!{z,        4, mem,    Adc,    adc, ([A] [D]) },
        0x8B => regular!{z,        4, mem,    Adc,    adc, ([A] [E]) },
        0x8C => regular!{z,        4, mem,    Adc,    adc, ([A] [IYH]) },
        0x8D => regular!{z,        4, mem,    Adc,    adc, ([A] [IYL]) },
        0x8E => e_inst! {z,    d, 15, mem,    Adc,    adc, ([A] [Shift(IY, d)]) },
        0x8F => regular!{z,        4, mem,    Adc,    adc, ([A] [A]) },
        0x90 => regular!{z,        4, mem,    Sub,    sub, ([A] [B]) },
        0x91 => regular!{z,        4, mem,    Sub,    sub, ([A] [C]) },
        0x92 => regular!{z,        4, mem,    Sub,    sub, ([A] [D]) },
        0x93 => regular!{z,        4, mem,    Sub,    sub, ([A] [E]) },
        0x94 => regular!{z,        4, mem,    Sub,    sub, ([A] [IYH]) },
        0x95 => regular!{z,        4, mem,    Sub,    sub, ([A] [IYL]) },
        0x96 => e_inst! {z,    d, 15, mem,    Sub,    sub, ([A] [Shift(IY, d)]) },
        0x97 => regular!{z,        4, mem,    Sub,    sub, ([A] [A]) },
        0x98 => regular!{z,        4, mem,    Sbc,    sbc, ([A] [B]) },
        0x99 => regular!{z,        4, mem,    Sbc,    sbc, ([A] [C]) },
        0x9A => regular!{z,        4, mem,    Sbc,    sbc, ([A] [D]) },
        0x9B => regular!{z,        4, mem,    Sbc,    sbc, ([A] [E]) },
        0x9C => regular!{z,        4, mem,    Sbc,    sbc, ([A] [IYH]) },
        0x9D => regular!{z,        4, mem,    Sbc,    sbc, ([A] [IYL]) },
        0x9E => e_inst! {z,    d, 15, mem,    Sbc,    sbc, ([A] [Shift(IY, d)]) },
        0x9F => regular!{z,        4, mem,    Sbc,    sbc, ([A] [A]) },
        0xA0 => regular!{z,        4, mem,    And,    and, ([B]) },
        0xA1 => regular!{z,        4, mem,    And,    and, ([C]) },
        0xA2 => regular!{z,        4, mem,    And,    and, ([D]) },
        0xA3 => regular!{z,        4, mem,    And,    and, ([E]) },
        0xA4 => regular!{z,        4, mem,    And,    and, ([IYH]) },
        0xA5 => regular!{z,        4, mem,    And,    and, ([IYL]) },
        0xA6 => e_inst! {z,    d, 15, mem,    And,    and, ([Shift(IY, d)]) },
        0xA7 => regular!{z,        4, mem,    And,    and, ([A]) },
        0xA8 => regular!{z,        4, mem,    Xor,    xor, ([B]) },
        0xA9 => regular!{z,        4, mem,    Xor,    xor, ([C]) },
        0xAA => regular!{z,        4, mem,    Xor,    xor, ([D]) },
        0xAB => regular!{z,        4, mem,    Xor,    xor, ([E]) },
        0xAC => regular!{z,        4, mem,    Xor,    xor, ([IYH]) },
        0xAD => regular!{z,        4, mem,    Xor,    xor, ([IYL]) },
        0xAE => e_inst! {z,    d, 15, mem,    Xor,    xor, ([Shift(IY, d)]) },
        0xAF => regular!{z,        4, mem,    Xor,    xor, ([A]) },
        0xB0 => regular!{z,        4, mem,     Or,     or, ([B]) },
        0xB1 => regular!{z,        4, mem,     Or,     or, ([C]) },
        0xB2 => regular!{z,        4, mem,     Or,     or, ([D]) },
        0xB3 => regular!{z,        4, mem,     Or,     or, ([E]) },
        0xB4 => regular!{z,        4, mem,     Or,     or, ([IYH]) },
        0xB5 => regular!{z,        4, mem,     Or,     or, ([IYL]) },
        0xB6 => e_inst! {z,    d, 15, mem,     Or,     or, ([Shift(IY, d)]) },
        0xB7 => regular!{z,        4, mem,     Or,     or, ([A]) },
        0xB8 => regular!{z,        4, mem,     Cp,     cp, ([B]) },
        0xB9 => regular!{z,        4, mem,     Cp,     cp, ([C]) },
        0xBA => regular!{z,        4, mem,     Cp,     cp, ([D]) },
        0xBB => regular!{z,        4, mem,     Cp,     cp, ([E]) },
        0xBC => regular!{z,        4, mem,     Cp,     cp, ([IYH]) },
        0xBD => regular!{z,        4, mem,     Cp,     cp, ([IYL]) },
        0xBE => e_inst! {z,    d, 15, mem,     Cp,     cp, ([Shift(IY, d)]) },
        0xBF => regular!{z,        4, mem,     Cp,     cp, ([A]) },
        0xC0 => regular!{z,        5, mem,  Retcc,  retcc, ([NZcc]) },
        0xC1 => regular!{z,       10, mem,    Pop,    pop, ([BC]) },
        0xC2 => nn_inst!{z,   nn, 10,  no,   Jpcc,   jpcc, ([NZcc] [nn]) },
        0xC3 => nn_inst!{z,   nn, 10, mem,     Jp,     jp, ([nn]) },
        0xC4 => nn_inst!{z,   nn,  0, mem, Callcc, callcc, ([NZcc] [nn]) },
        0xC5 => regular!{z,       11, mem,   Push,   push, ([BC]) },
        0xC6 => n_inst!{z,     n,  7, mem,    Add,    add, ([A] [n]) },
        0xC7 => regular!{z,       11, mem,    Rst,    rst, ([0x00]) },
        0xC8 => regular!{z,        5, mem,  Retcc,  retcc, ([Zcc]) },
        0xC9 => regular!{z,       10, mem,    Ret,    ret, () },
        0xCA => nn_inst!{z,   nn, 10,  no,   Jpcc,   jpcc, ([Zcc] [nn]) },
        0xCB => regular!{z,        0,  no,   Fdcb,   fdcb, () },
        0xCC => nn_inst!{z,   nn,  0, mem, Callcc, callcc, ([Zcc] [nn]) },
        0xCD => nn_inst!{z,   nn, 17, mem,   Call,   call, ([nn]) },
        0xCE => n_inst! {z,    n,  7, mem,    Adc,    adc, ([A] [n]) },
        0xCF => regular!{z,       11, mem,    Rst,    rst, ([0x08]) },
        0xD0 => regular!{z,        5, mem,  Retcc,  retcc, ([NCcc]) },
        0xD1 => regular!{z,       10, mem,    Pop,    pop, ([DE]) },
        0xD2 => nn_inst!{z,   nn, 10,  no,   Jpcc,   jpcc, ([NCcc] [nn]) },
        0xD3 => n_inst! {z,    n, 11,  io,   OutN,  out_n, ([n ] [A]) },
        0xD4 => nn_inst!{z,   nn,  0, mem, Callcc, callcc, ([NCcc] [nn]) },
        0xD5 => regular!{z,       11, mem,   Push,   push, ([DE]) },
        0xD6 => n_inst! {z,    n,  7, mem,    Sub,    sub, ([A] [n]) },
        0xD7 => regular!{z,       11, mem,    Rst,    rst, ([0x10]) },
        0xD8 => regular!{z,        5, mem,  Retcc,  retcc, ([Ccc]) },
        0xD9 => regular!{z,        4,  no,    Exx,    exx, () },
        0xDA => nn_inst!{z,   nn, 10,  no,   Jpcc,   jpcc, ([Ccc] [nn]) },
        0xDB => n_inst! {z,    n, 11,  io,    InN,   in_n, ([A] [n]) },
        0xDC => nn_inst!{z,   nn,  0, mem, Callcc, callcc, ([Ccc] [nn]) },
        0xDD => regular!{z,        4,  no,     Dd,     dd, () },
        0xDE => n_inst! {z,    n,  7, mem,    Sbc,    sbc, ([A] [n]) },
        0xDF => regular!{z,       11, mem,    Rst,    rst, ([0x18]) },
        0xE0 => regular!{z,        5, mem,  Retcc,  retcc, ([POcc]) },
        0xE1 => regular!{z,       10, mem,    Pop,    pop, ([IY]) },
        0xE2 => nn_inst!{z,   nn, 10,  no,   Jpcc,   jpcc, ([POcc] [nn]) },
        0xE3 => regular!{z,       19, mem,     Ex,     ex, ([Address(SP)] [IY]) },
        0xE4 => nn_inst!{z,   nn,  0, mem, Callcc, callcc, ([POcc] [nn]) },
        0xE5 => regular!{z,       11, mem,   Push,   push, ([IY]) },
        0xE6 => n_inst! {z,    n,  7, mem,    And,    and, ([n]) },
        0xE7 => regular!{z,       11, mem,    Rst,    rst, ([0x20]) },
        0xE8 => regular!{z,        5, mem,  Retcc,  retcc, ([PEcc]) },
        0xE9 => regular!{z,        4, mem,     Jp,     jp, ([IY]) },
        0xEA => nn_inst!{z,   nn, 10,  no,   Jpcc,   jpcc, ([PEcc] [nn]) },
        0xEB => regular!{z,        4, mem,     Ex,     ex, ([DE] [HL]) },
        0xEC => nn_inst!{z,   nn,  0, mem, Callcc, callcc, ([PEcc] [nn]) },
        0xED => regular!{z,        4,  no,     Ed,     ed, () },
        0xEE => n_inst! {z,    n,  7, mem,    Xor,    xor, ([n]) },
        0xEF => regular!{z,       11, mem,    Rst,    rst, ([0x28]) },
        0xF0 => regular!{z,        5, mem,  Retcc,  retcc, ([Pcc]) },
        0xF1 => regular!{z,       10, mem,    Pop,    pop, ([AF]) },
        0xF2 => nn_inst!{z,   nn, 10,  no,   Jpcc,   jpcc, ([Pcc] [nn]) },
        0xF3 => regular!{z,        4,  no,     Di,     di, () },
        0xF4 => nn_inst!{z,   nn,  0, mem, Callcc, callcc, ([Pcc] [nn]) },
        0xF5 => regular!{z,       11, mem,   Push,   push, ([AF]) },
        0xF6 => n_inst! {z,    n,  7, mem,     Or,     or, ([n]) },
        0xF7 => regular!{z,       11, mem,    Rst,    rst, ([0x30]) },
        0xF8 => regular!{z,        5, mem,  Retcc,  retcc, ([Mcc]) },
        0xF9 => regular!{z,        6, mem,   Ld16,   ld16, ([SP] [IY]) },
        0xFA => nn_inst!{z,   nn, 10,  no,   Jpcc,   jpcc, ([Mcc] [nn]) },
        0xFB => regular!{z,        4,  no,     Ei,     ei, () },
        0xFC => nn_inst!{z,   nn,  0, mem, Callcc, callcc, ([Mcc] [nn]) },
        0xFD => regular!{z,        4,  no,     Fd,     fd, () },
        0xFE => n_inst! {z,    n,  7, mem,     Cp,     cp, ([n]) },
        0xFF => regular!{z,       11, mem,    Rst,    rst, ([0x38]) },
        _ => unimplemented!(),
    }
}

pub fn ddcb<Z>(z: &mut Z)
where
    Z: Z80Emulator,
{
    use self::instruction_traits::*;

    use self::Reg16::*;
    use self::Reg8::*;

    let d = z.read_pc() as i8;
    z.inc_pc();
    let pc = z.read_pc();
    z.inc_pc();
    match pc {
        0x00 => regular!{z, 19, mem, RlcStore, rlc_store, ([Shift(IX, d)] [B]) },
        0x01 => regular!{z, 19, mem, RlcStore, rlc_store, ([Shift(IX, d)] [C]) },
        0x02 => regular!{z, 19, mem, RlcStore, rlc_store, ([Shift(IX, d)] [D]) },
        0x03 => regular!{z, 19, mem, RlcStore, rlc_store, ([Shift(IX, d)] [E]) },
        0x04 => regular!{z, 19, mem, RlcStore, rlc_store, ([Shift(IX, d)] [H]) },
        0x05 => regular!{z, 19, mem, RlcStore, rlc_store, ([Shift(IX, d)] [L]) },
        0x06 => regular!{z, 19, mem, Rlc, rlc, ([Shift(IX, d)]) },
        0x07 => regular!{z, 19, mem, RlcStore, rlc_store, ([Shift(IX, d)] [A]) },
        0x08 => regular!{z, 19, mem, RrcStore, rrc_store, ([Shift(IX, d)] [B]) },
        0x09 => regular!{z, 19, mem, RrcStore, rrc_store, ([Shift(IX, d)] [C]) },
        0x0A => regular!{z, 19, mem, RrcStore, rrc_store, ([Shift(IX, d)] [D]) },
        0x0B => regular!{z, 19, mem, RrcStore, rrc_store, ([Shift(IX, d)] [E]) },
        0x0C => regular!{z, 19, mem, RrcStore, rrc_store, ([Shift(IX, d)] [H]) },
        0x0D => regular!{z, 19, mem, RrcStore, rrc_store, ([Shift(IX, d)] [L]) },
        0x0E => regular!{z, 19, mem, Rrc, rrc, ([Shift(IX, d)]) },
        0x0F => regular!{z, 19, mem, RrcStore, rrc_store, ([Shift(IX, d)] [A]) },
        0x10 => regular!{z, 19, mem, RlStore, rl_store, ([Shift(IX, d)] [B]) },
        0x11 => regular!{z, 19, mem, RlStore, rl_store, ([Shift(IX, d)] [C]) },
        0x12 => regular!{z, 19, mem, RlStore, rl_store, ([Shift(IX, d)] [D]) },
        0x13 => regular!{z, 19, mem, RlStore, rl_store, ([Shift(IX, d)] [E]) },
        0x14 => regular!{z, 19, mem, RlStore, rl_store, ([Shift(IX, d)] [H]) },
        0x15 => regular!{z, 19, mem, RlStore, rl_store, ([Shift(IX, d)] [L]) },
        0x16 => regular!{z, 19, mem, Rl, rl, ([Shift(IX, d)]) },
        0x17 => regular!{z, 19, mem, RlStore, rl_store, ([Shift(IX, d)] [A]) },
        0x18 => regular!{z, 19, mem, RrStore, rr_store, ([Shift(IX, d)] [B]) },
        0x19 => regular!{z, 19, mem, RrStore, rr_store, ([Shift(IX, d)] [C]) },
        0x1A => regular!{z, 19, mem, RrStore, rr_store, ([Shift(IX, d)] [D]) },
        0x1B => regular!{z, 19, mem, RrStore, rr_store, ([Shift(IX, d)] [E]) },
        0x1C => regular!{z, 19, mem, RrStore, rr_store, ([Shift(IX, d)] [H]) },
        0x1D => regular!{z, 19, mem, RrStore, rr_store, ([Shift(IX, d)] [L]) },
        0x1E => regular!{z, 19, mem, Rr, rr, ([Shift(IX, d)]) },
        0x1F => regular!{z, 19, mem, RrStore, rr_store, ([Shift(IX, d)] [A]) },
        0x20 => regular!{z, 19, mem, SlaStore, sla_store, ([Shift(IX, d)] [B]) },
        0x21 => regular!{z, 19, mem, SlaStore, sla_store, ([Shift(IX, d)] [C]) },
        0x22 => regular!{z, 19, mem, SlaStore, sla_store, ([Shift(IX, d)] [D]) },
        0x23 => regular!{z, 19, mem, SlaStore, sla_store, ([Shift(IX, d)] [E]) },
        0x24 => regular!{z, 19, mem, SlaStore, sla_store, ([Shift(IX, d)] [H]) },
        0x25 => regular!{z, 19, mem, SlaStore, sla_store, ([Shift(IX, d)] [L]) },
        0x26 => regular!{z, 19, mem, Sla, sla, ([Shift(IX, d)]) },
        0x27 => regular!{z, 19, mem, SlaStore, sla_store, ([Shift(IX, d)] [A]) },
        0x28 => regular!{z, 19, mem, SraStore, sra_store, ([Shift(IX, d)] [B]) },
        0x29 => regular!{z, 19, mem, SraStore, sra_store, ([Shift(IX, d)] [C]) },
        0x2A => regular!{z, 19, mem, SraStore, sra_store, ([Shift(IX, d)] [D]) },
        0x2B => regular!{z, 19, mem, SraStore, sra_store, ([Shift(IX, d)] [E]) },
        0x2C => regular!{z, 19, mem, SraStore, sra_store, ([Shift(IX, d)] [H]) },
        0x2D => regular!{z, 19, mem, SraStore, sra_store, ([Shift(IX, d)] [L]) },
        0x2E => regular!{z, 19, mem, Sra, sra, ([Shift(IX, d)]) },
        0x2F => regular!{z, 19, mem, SraStore, sra_store, ([Shift(IX, d)] [A]) },
        0x30 => regular!{z, 19, mem, SllStore, sll_store, ([Shift(IX, d)] [B]) },
        0x31 => regular!{z, 19, mem, SllStore, sll_store, ([Shift(IX, d)] [C]) },
        0x32 => regular!{z, 19, mem, SllStore, sll_store, ([Shift(IX, d)] [D]) },
        0x33 => regular!{z, 19, mem, SllStore, sll_store, ([Shift(IX, d)] [E]) },
        0x34 => regular!{z, 19, mem, SllStore, sll_store, ([Shift(IX, d)] [H]) },
        0x35 => regular!{z, 19, mem, SllStore, sll_store, ([Shift(IX, d)] [L]) },
        0x36 => regular!{z, 19, mem, Sll, sll, ([Shift(IX, d)]) },
        0x37 => regular!{z, 19, mem, SllStore, sll_store, ([Shift(IX, d)] [A]) },
        0x38 => regular!{z, 19, mem, SrlStore, srl_store, ([Shift(IX, d)] [B]) },
        0x39 => regular!{z, 19, mem, SrlStore, srl_store, ([Shift(IX, d)] [C]) },
        0x3A => regular!{z, 19, mem, SrlStore, srl_store, ([Shift(IX, d)] [D]) },
        0x3B => regular!{z, 19, mem, SrlStore, srl_store, ([Shift(IX, d)] [E]) },
        0x3C => regular!{z, 19, mem, SrlStore, srl_store, ([Shift(IX, d)] [H]) },
        0x3D => regular!{z, 19, mem, SrlStore, srl_store, ([Shift(IX, d)] [L]) },
        0x3E => regular!{z, 19, mem, Srl, srl, ([Shift(IX, d)]) },
        0x3F => regular!{z, 19, mem, SrlStore, srl_store, ([Shift(IX, d)] [A]) },
        0x40 => regular!{z, 16, mem, Bit, bit, ([0] [Shift(IX, d)]) },
        0x41 => regular!{z, 16, mem, Bit, bit, ([0] [Shift(IX, d)]) },
        0x42 => regular!{z, 16, mem, Bit, bit, ([0] [Shift(IX, d)]) },
        0x43 => regular!{z, 16, mem, Bit, bit, ([0] [Shift(IX, d)]) },
        0x44 => regular!{z, 16, mem, Bit, bit, ([0] [Shift(IX, d)]) },
        0x45 => regular!{z, 16, mem, Bit, bit, ([0] [Shift(IX, d)]) },
        0x46 => regular!{z, 16, mem, Bit, bit, ([0] [Shift(IX, d)]) },
        0x47 => regular!{z, 16, mem, Bit, bit, ([0] [Shift(IX, d)]) },
        0x48 => regular!{z, 16, mem, Bit, bit, ([1] [Shift(IX, d)]) },
        0x49 => regular!{z, 16, mem, Bit, bit, ([1] [Shift(IX, d)]) },
        0x4A => regular!{z, 16, mem, Bit, bit, ([1] [Shift(IX, d)]) },
        0x4B => regular!{z, 16, mem, Bit, bit, ([1] [Shift(IX, d)]) },
        0x4C => regular!{z, 16, mem, Bit, bit, ([1] [Shift(IX, d)]) },
        0x4D => regular!{z, 16, mem, Bit, bit, ([1] [Shift(IX, d)]) },
        0x4E => regular!{z, 16, mem, Bit, bit, ([1] [Shift(IX, d)]) },
        0x4F => regular!{z, 16, mem, Bit, bit, ([1] [Shift(IX, d)]) },
        0x50 => regular!{z, 16, mem, Bit, bit, ([2] [Shift(IX, d)]) },
        0x51 => regular!{z, 16, mem, Bit, bit, ([2] [Shift(IX, d)]) },
        0x52 => regular!{z, 16, mem, Bit, bit, ([2] [Shift(IX, d)]) },
        0x53 => regular!{z, 16, mem, Bit, bit, ([2] [Shift(IX, d)]) },
        0x54 => regular!{z, 16, mem, Bit, bit, ([2] [Shift(IX, d)]) },
        0x55 => regular!{z, 16, mem, Bit, bit, ([2] [Shift(IX, d)]) },
        0x56 => regular!{z, 16, mem, Bit, bit, ([2] [Shift(IX, d)]) },
        0x57 => regular!{z, 16, mem, Bit, bit, ([2] [Shift(IX, d)]) },
        0x58 => regular!{z, 16, mem, Bit, bit, ([3] [Shift(IX, d)]) },
        0x59 => regular!{z, 16, mem, Bit, bit, ([3] [Shift(IX, d)]) },
        0x5A => regular!{z, 16, mem, Bit, bit, ([3] [Shift(IX, d)]) },
        0x5B => regular!{z, 16, mem, Bit, bit, ([3] [Shift(IX, d)]) },
        0x5C => regular!{z, 16, mem, Bit, bit, ([3] [Shift(IX, d)]) },
        0x5D => regular!{z, 16, mem, Bit, bit, ([3] [Shift(IX, d)]) },
        0x5E => regular!{z, 16, mem, Bit, bit, ([3] [Shift(IX, d)]) },
        0x5F => regular!{z, 16, mem, Bit, bit, ([3] [Shift(IX, d)]) },
        0x60 => regular!{z, 16, mem, Bit, bit, ([4] [Shift(IX, d)]) },
        0x61 => regular!{z, 16, mem, Bit, bit, ([4] [Shift(IX, d)]) },
        0x62 => regular!{z, 16, mem, Bit, bit, ([4] [Shift(IX, d)]) },
        0x63 => regular!{z, 16, mem, Bit, bit, ([4] [Shift(IX, d)]) },
        0x64 => regular!{z, 16, mem, Bit, bit, ([4] [Shift(IX, d)]) },
        0x65 => regular!{z, 16, mem, Bit, bit, ([4] [Shift(IX, d)]) },
        0x66 => regular!{z, 16, mem, Bit, bit, ([4] [Shift(IX, d)]) },
        0x67 => regular!{z, 16, mem, Bit, bit, ([4] [Shift(IX, d)]) },
        0x68 => regular!{z, 16, mem, Bit, bit, ([5] [Shift(IX, d)]) },
        0x69 => regular!{z, 16, mem, Bit, bit, ([5] [Shift(IX, d)]) },
        0x6A => regular!{z, 16, mem, Bit, bit, ([5] [Shift(IX, d)]) },
        0x6B => regular!{z, 16, mem, Bit, bit, ([5] [Shift(IX, d)]) },
        0x6C => regular!{z, 16, mem, Bit, bit, ([5] [Shift(IX, d)]) },
        0x6D => regular!{z, 16, mem, Bit, bit, ([5] [Shift(IX, d)]) },
        0x6E => regular!{z, 16, mem, Bit, bit, ([5] [Shift(IX, d)]) },
        0x6F => regular!{z, 16, mem, Bit, bit, ([5] [Shift(IX, d)]) },
        0x70 => regular!{z, 16, mem, Bit, bit, ([6] [Shift(IX, d)]) },
        0x71 => regular!{z, 16, mem, Bit, bit, ([6] [Shift(IX, d)]) },
        0x72 => regular!{z, 16, mem, Bit, bit, ([6] [Shift(IX, d)]) },
        0x73 => regular!{z, 16, mem, Bit, bit, ([6] [Shift(IX, d)]) },
        0x74 => regular!{z, 16, mem, Bit, bit, ([6] [Shift(IX, d)]) },
        0x75 => regular!{z, 16, mem, Bit, bit, ([6] [Shift(IX, d)]) },
        0x76 => regular!{z, 16, mem, Bit, bit, ([6] [Shift(IX, d)]) },
        0x77 => regular!{z, 16, mem, Bit, bit, ([6] [Shift(IX, d)]) },
        0x78 => regular!{z, 16, mem, Bit, bit, ([7] [Shift(IX, d)]) },
        0x79 => regular!{z, 16, mem, Bit, bit, ([7] [Shift(IX, d)]) },
        0x7A => regular!{z, 16, mem, Bit, bit, ([7] [Shift(IX, d)]) },
        0x7B => regular!{z, 16, mem, Bit, bit, ([7] [Shift(IX, d)]) },
        0x7C => regular!{z, 16, mem, Bit, bit, ([7] [Shift(IX, d)]) },
        0x7D => regular!{z, 16, mem, Bit, bit, ([7] [Shift(IX, d)]) },
        0x7E => regular!{z, 16, mem, Bit, bit, ([7] [Shift(IX, d)]) },
        0x7F => regular!{z, 16, mem, Bit, bit, ([7] [Shift(IX, d)]) },
        0x80 => regular!{z, 19, mem, ResStore, res_store, ([0] [Shift(IX, d)] [B]) },
        0x81 => regular!{z, 19, mem, ResStore, res_store, ([0] [Shift(IX, d)] [C]) },
        0x82 => regular!{z, 19, mem, ResStore, res_store, ([0] [Shift(IX, d)] [D]) },
        0x83 => regular!{z, 19, mem, ResStore, res_store, ([0] [Shift(IX, d)] [E]) },
        0x84 => regular!{z, 19, mem, ResStore, res_store, ([0] [Shift(IX, d)] [H]) },
        0x85 => regular!{z, 19, mem, ResStore, res_store, ([0] [Shift(IX, d)] [L]) },
        0x86 => regular!{z, 19, mem, Res, res, ([0] [Shift(IX, d)]) },
        0x87 => regular!{z, 19, mem, ResStore, res_store, ([0] [Shift(IX, d)] [A]) },
        0x88 => regular!{z, 19, mem, ResStore, res_store, ([1] [Shift(IX, d)] [B]) },
        0x89 => regular!{z, 19, mem, ResStore, res_store, ([1] [Shift(IX, d)] [C]) },
        0x8A => regular!{z, 19, mem, ResStore, res_store, ([1] [Shift(IX, d)] [D]) },
        0x8B => regular!{z, 19, mem, ResStore, res_store, ([1] [Shift(IX, d)] [E]) },
        0x8C => regular!{z, 19, mem, ResStore, res_store, ([1] [Shift(IX, d)] [H]) },
        0x8D => regular!{z, 19, mem, ResStore, res_store, ([1] [Shift(IX, d)] [L]) },
        0x8E => regular!{z, 19, mem, Res, res, ([1] [Shift(IX, d)]) },
        0x8F => regular!{z, 19, mem, ResStore, res_store, ([1] [Shift(IX, d)] [A]) },
        0x90 => regular!{z, 19, mem, ResStore, res_store, ([2] [Shift(IX, d)] [B]) },
        0x91 => regular!{z, 19, mem, ResStore, res_store, ([2] [Shift(IX, d)] [C]) },
        0x92 => regular!{z, 19, mem, ResStore, res_store, ([2] [Shift(IX, d)] [D]) },
        0x93 => regular!{z, 19, mem, ResStore, res_store, ([2] [Shift(IX, d)] [E]) },
        0x94 => regular!{z, 19, mem, ResStore, res_store, ([2] [Shift(IX, d)] [H]) },
        0x95 => regular!{z, 19, mem, ResStore, res_store, ([2] [Shift(IX, d)] [L]) },
        0x96 => regular!{z, 19, mem, Res, res, ([2] [Shift(IX, d)]) },
        0x97 => regular!{z, 19, mem, ResStore, res_store, ([2] [Shift(IX, d)] [A]) },
        0x98 => regular!{z, 19, mem, ResStore, res_store, ([3] [Shift(IX, d)] [B]) },
        0x99 => regular!{z, 19, mem, ResStore, res_store, ([3] [Shift(IX, d)] [C]) },
        0x9A => regular!{z, 19, mem, ResStore, res_store, ([3] [Shift(IX, d)] [D]) },
        0x9B => regular!{z, 19, mem, ResStore, res_store, ([3] [Shift(IX, d)] [E]) },
        0x9C => regular!{z, 19, mem, ResStore, res_store, ([3] [Shift(IX, d)] [H]) },
        0x9D => regular!{z, 19, mem, ResStore, res_store, ([3] [Shift(IX, d)] [L]) },
        0x9E => regular!{z, 19, mem, Res, res, ([3] [Shift(IX, d)]) },
        0x9F => regular!{z, 19, mem, ResStore, res_store, ([3] [Shift(IX, d)] [A]) },
        0xA0 => regular!{z, 19, mem, ResStore, res_store, ([4] [Shift(IX, d)] [B]) },
        0xA1 => regular!{z, 19, mem, ResStore, res_store, ([4] [Shift(IX, d)] [C]) },
        0xA2 => regular!{z, 19, mem, ResStore, res_store, ([4] [Shift(IX, d)] [D]) },
        0xA3 => regular!{z, 19, mem, ResStore, res_store, ([4] [Shift(IX, d)] [E]) },
        0xA4 => regular!{z, 19, mem, ResStore, res_store, ([4] [Shift(IX, d)] [H]) },
        0xA5 => regular!{z, 19, mem, ResStore, res_store, ([4] [Shift(IX, d)] [L]) },
        0xA6 => regular!{z, 19, mem, Res, res, ([4] [Shift(IX, d)]) },
        0xA7 => regular!{z, 19, mem, ResStore, res_store, ([4] [Shift(IX, d)] [A]) },
        0xA8 => regular!{z, 19, mem, ResStore, res_store, ([5] [Shift(IX, d)] [B]) },
        0xA9 => regular!{z, 19, mem, ResStore, res_store, ([5] [Shift(IX, d)] [C]) },
        0xAA => regular!{z, 19, mem, ResStore, res_store, ([5] [Shift(IX, d)] [D]) },
        0xAB => regular!{z, 19, mem, ResStore, res_store, ([5] [Shift(IX, d)] [E]) },
        0xAC => regular!{z, 19, mem, ResStore, res_store, ([5] [Shift(IX, d)] [H]) },
        0xAD => regular!{z, 19, mem, ResStore, res_store, ([5] [Shift(IX, d)] [L]) },
        0xAE => regular!{z, 19, mem, Res, res, ([5] [Shift(IX, d)]) },
        0xAF => regular!{z, 19, mem, ResStore, res_store, ([5] [Shift(IX, d)] [A]) },
        0xB0 => regular!{z, 19, mem, ResStore, res_store, ([6] [Shift(IX, d)] [B]) },
        0xB1 => regular!{z, 19, mem, ResStore, res_store, ([6] [Shift(IX, d)] [C]) },
        0xB2 => regular!{z, 19, mem, ResStore, res_store, ([6] [Shift(IX, d)] [D]) },
        0xB3 => regular!{z, 19, mem, ResStore, res_store, ([6] [Shift(IX, d)] [E]) },
        0xB4 => regular!{z, 19, mem, ResStore, res_store, ([6] [Shift(IX, d)] [H]) },
        0xB5 => regular!{z, 19, mem, ResStore, res_store, ([6] [Shift(IX, d)] [L]) },
        0xB6 => regular!{z, 19, mem, Res, res, ([6] [Shift(IX, d)]) },
        0xB7 => regular!{z, 19, mem, ResStore, res_store, ([6] [Shift(IX, d)] [A]) },
        0xB8 => regular!{z, 19, mem, ResStore, res_store, ([7] [Shift(IX, d)] [B]) },
        0xB9 => regular!{z, 19, mem, ResStore, res_store, ([7] [Shift(IX, d)] [C]) },
        0xBA => regular!{z, 19, mem, ResStore, res_store, ([7] [Shift(IX, d)] [D]) },
        0xBB => regular!{z, 19, mem, ResStore, res_store, ([7] [Shift(IX, d)] [E]) },
        0xBC => regular!{z, 19, mem, ResStore, res_store, ([7] [Shift(IX, d)] [H]) },
        0xBD => regular!{z, 19, mem, ResStore, res_store, ([7] [Shift(IX, d)] [L]) },
        0xBE => regular!{z, 19, mem, Res, res, ([7] [Shift(IX, d)]) },
        0xBF => regular!{z, 19, mem, ResStore, res_store, ([7] [Shift(IX, d)] [A]) },
        0xC0 => regular!{z, 19, mem, SetStore, set_store, ([0] [Shift(IX, d)] [B]) },
        0xC1 => regular!{z, 19, mem, SetStore, set_store, ([0] [Shift(IX, d)] [C]) },
        0xC2 => regular!{z, 19, mem, SetStore, set_store, ([0] [Shift(IX, d)] [D]) },
        0xC3 => regular!{z, 19, mem, SetStore, set_store, ([0] [Shift(IX, d)] [E]) },
        0xC4 => regular!{z, 19, mem, SetStore, set_store, ([0] [Shift(IX, d)] [H]) },
        0xC5 => regular!{z, 19, mem, SetStore, set_store, ([0] [Shift(IX, d)] [L]) },
        0xC6 => regular!{z, 19, mem, Set, set, ([0] [Shift(IX, d)]) },
        0xC7 => regular!{z, 19, mem, SetStore, set_store, ([0] [Shift(IX, d)] [A]) },
        0xC8 => regular!{z, 19, mem, SetStore, set_store, ([1] [Shift(IX, d)] [B]) },
        0xC9 => regular!{z, 19, mem, SetStore, set_store, ([1] [Shift(IX, d)] [C]) },
        0xCA => regular!{z, 19, mem, SetStore, set_store, ([1] [Shift(IX, d)] [D]) },
        0xCB => regular!{z, 19, mem, SetStore, set_store, ([1] [Shift(IX, d)] [E]) },
        0xCC => regular!{z, 19, mem, SetStore, set_store, ([1] [Shift(IX, d)] [H]) },
        0xCD => regular!{z, 19, mem, SetStore, set_store, ([1] [Shift(IX, d)] [L]) },
        0xCE => regular!{z, 19, mem, Set, set, ([1] [Shift(IX, d)]) },
        0xCF => regular!{z, 19, mem, SetStore, set_store, ([1] [Shift(IX, d)] [A]) },
        0xD0 => regular!{z, 19, mem, SetStore, set_store, ([2] [Shift(IX, d)] [B]) },
        0xD1 => regular!{z, 19, mem, SetStore, set_store, ([2] [Shift(IX, d)] [C]) },
        0xD2 => regular!{z, 19, mem, SetStore, set_store, ([2] [Shift(IX, d)] [D]) },
        0xD3 => regular!{z, 19, mem, SetStore, set_store, ([2] [Shift(IX, d)] [E]) },
        0xD4 => regular!{z, 19, mem, SetStore, set_store, ([2] [Shift(IX, d)] [H]) },
        0xD5 => regular!{z, 19, mem, SetStore, set_store, ([2] [Shift(IX, d)] [L]) },
        0xD6 => regular!{z, 19, mem, Set, set, ([2] [Shift(IX, d)]) },
        0xD7 => regular!{z, 19, mem, SetStore, set_store, ([2] [Shift(IX, d)] [A]) },
        0xD8 => regular!{z, 19, mem, SetStore, set_store, ([3] [Shift(IX, d)] [B]) },
        0xD9 => regular!{z, 19, mem, SetStore, set_store, ([3] [Shift(IX, d)] [C]) },
        0xDA => regular!{z, 19, mem, SetStore, set_store, ([3] [Shift(IX, d)] [D]) },
        0xDB => regular!{z, 19, mem, SetStore, set_store, ([3] [Shift(IX, d)] [E]) },
        0xDC => regular!{z, 19, mem, SetStore, set_store, ([3] [Shift(IX, d)] [H]) },
        0xDD => regular!{z, 19, mem, SetStore, set_store, ([3] [Shift(IX, d)] [L]) },
        0xDE => regular!{z, 19, mem, Set, set, ([3] [Shift(IX, d)]) },
        0xDF => regular!{z, 19, mem, SetStore, set_store, ([3] [Shift(IX, d)] [A]) },
        0xE0 => regular!{z, 19, mem, SetStore, set_store, ([4] [Shift(IX, d)] [B]) },
        0xE1 => regular!{z, 19, mem, SetStore, set_store, ([4] [Shift(IX, d)] [C]) },
        0xE2 => regular!{z, 19, mem, SetStore, set_store, ([4] [Shift(IX, d)] [D]) },
        0xE3 => regular!{z, 19, mem, SetStore, set_store, ([4] [Shift(IX, d)] [E]) },
        0xE4 => regular!{z, 19, mem, SetStore, set_store, ([4] [Shift(IX, d)] [H]) },
        0xE5 => regular!{z, 19, mem, SetStore, set_store, ([4] [Shift(IX, d)] [L]) },
        0xE6 => regular!{z, 19, mem, Set, set, ([4] [Shift(IX, d)]) },
        0xE7 => regular!{z, 19, mem, SetStore, set_store, ([4] [Shift(IX, d)] [A]) },
        0xE8 => regular!{z, 19, mem, SetStore, set_store, ([5] [Shift(IX, d)] [B]) },
        0xE9 => regular!{z, 19, mem, SetStore, set_store, ([5] [Shift(IX, d)] [C]) },
        0xEA => regular!{z, 19, mem, SetStore, set_store, ([5] [Shift(IX, d)] [D]) },
        0xEB => regular!{z, 19, mem, SetStore, set_store, ([5] [Shift(IX, d)] [E]) },
        0xEC => regular!{z, 19, mem, SetStore, set_store, ([5] [Shift(IX, d)] [H]) },
        0xED => regular!{z, 19, mem, SetStore, set_store, ([5] [Shift(IX, d)] [L]) },
        0xEE => regular!{z, 19, mem, Set, set, ([5] [Shift(IX, d)]) },
        0xEF => regular!{z, 19, mem, SetStore, set_store, ([5] [Shift(IX, d)] [A]) },
        0xF0 => regular!{z, 19, mem, SetStore, set_store, ([6] [Shift(IX, d)] [B]) },
        0xF1 => regular!{z, 19, mem, SetStore, set_store, ([6] [Shift(IX, d)] [C]) },
        0xF2 => regular!{z, 19, mem, SetStore, set_store, ([6] [Shift(IX, d)] [D]) },
        0xF3 => regular!{z, 19, mem, SetStore, set_store, ([6] [Shift(IX, d)] [E]) },
        0xF4 => regular!{z, 19, mem, SetStore, set_store, ([6] [Shift(IX, d)] [H]) },
        0xF5 => regular!{z, 19, mem, SetStore, set_store, ([6] [Shift(IX, d)] [L]) },
        0xF6 => regular!{z, 19, mem, Set, set, ([6] [Shift(IX, d)]) },
        0xF7 => regular!{z, 19, mem, SetStore, set_store, ([6] [Shift(IX, d)] [A]) },
        0xF8 => regular!{z, 19, mem, SetStore, set_store, ([7] [Shift(IX, d)] [B]) },
        0xF9 => regular!{z, 19, mem, SetStore, set_store, ([7] [Shift(IX, d)] [C]) },
        0xFA => regular!{z, 19, mem, SetStore, set_store, ([7] [Shift(IX, d)] [D]) },
        0xFB => regular!{z, 19, mem, SetStore, set_store, ([7] [Shift(IX, d)] [E]) },
        0xFC => regular!{z, 19, mem, SetStore, set_store, ([7] [Shift(IX, d)] [H]) },
        0xFD => regular!{z, 19, mem, SetStore, set_store, ([7] [Shift(IX, d)] [L]) },
        0xFE => regular!{z, 19, mem, Set, set, ([7] [Shift(IX, d)]) },
        0xFF => regular!{z, 19, mem, SetStore, set_store, ([7] [Shift(IX, d)] [A]) },
        _ => unimplemented!(),
    }
}

pub fn fdcb<Z>(z: &mut Z)
where
    Z: Z80Emulator,
{
    use self::instruction_traits::*;

    use self::Reg16::*;
    use self::Reg8::*;

    let d = z.read_pc() as i8;
    z.inc_pc();
    let pc = z.read_pc();
    z.inc_pc();
    match pc {
        0x00 => regular!{z, 19, mem, RlcStore, rlc_store, ([Shift(IY, d)] [B]) },
        0x01 => regular!{z, 19, mem, RlcStore, rlc_store, ([Shift(IY, d)] [C]) },
        0x02 => regular!{z, 19, mem, RlcStore, rlc_store, ([Shift(IY, d)] [D]) },
        0x03 => regular!{z, 19, mem, RlcStore, rlc_store, ([Shift(IY, d)] [E]) },
        0x04 => regular!{z, 19, mem, RlcStore, rlc_store, ([Shift(IY, d)] [H]) },
        0x05 => regular!{z, 19, mem, RlcStore, rlc_store, ([Shift(IY, d)] [L]) },
        0x06 => regular!{z, 19, mem, Rlc, rlc, ([Shift(IY, d)]) },
        0x07 => regular!{z, 19, mem, RlcStore, rlc_store, ([Shift(IY, d)] [A]) },
        0x08 => regular!{z, 19, mem, RrcStore, rrc_store, ([Shift(IY, d)] [B]) },
        0x09 => regular!{z, 19, mem, RrcStore, rrc_store, ([Shift(IY, d)] [C]) },
        0x0A => regular!{z, 19, mem, RrcStore, rrc_store, ([Shift(IY, d)] [D]) },
        0x0B => regular!{z, 19, mem, RrcStore, rrc_store, ([Shift(IY, d)] [E]) },
        0x0C => regular!{z, 19, mem, RrcStore, rrc_store, ([Shift(IY, d)] [H]) },
        0x0D => regular!{z, 19, mem, RrcStore, rrc_store, ([Shift(IY, d)] [L]) },
        0x0E => regular!{z, 19, mem, Rrc, rrc, ([Shift(IY, d)]) },
        0x0F => regular!{z, 19, mem, RrcStore, rrc_store, ([Shift(IY, d)] [A]) },
        0x10 => regular!{z, 19, mem, RlStore, rl_store, ([Shift(IY, d)] [B]) },
        0x11 => regular!{z, 19, mem, RlStore, rl_store, ([Shift(IY, d)] [C]) },
        0x12 => regular!{z, 19, mem, RlStore, rl_store, ([Shift(IY, d)] [D]) },
        0x13 => regular!{z, 19, mem, RlStore, rl_store, ([Shift(IY, d)] [E]) },
        0x14 => regular!{z, 19, mem, RlStore, rl_store, ([Shift(IY, d)] [H]) },
        0x15 => regular!{z, 19, mem, RlStore, rl_store, ([Shift(IY, d)] [L]) },
        0x16 => regular!{z, 19, mem, Rl, rl, ([Shift(IY, d)]) },
        0x17 => regular!{z, 19, mem, RlStore, rl_store, ([Shift(IY, d)] [A]) },
        0x18 => regular!{z, 19, mem, RrStore, rr_store, ([Shift(IY, d)] [B]) },
        0x19 => regular!{z, 19, mem, RrStore, rr_store, ([Shift(IY, d)] [C]) },
        0x1A => regular!{z, 19, mem, RrStore, rr_store, ([Shift(IY, d)] [D]) },
        0x1B => regular!{z, 19, mem, RrStore, rr_store, ([Shift(IY, d)] [E]) },
        0x1C => regular!{z, 19, mem, RrStore, rr_store, ([Shift(IY, d)] [H]) },
        0x1D => regular!{z, 19, mem, RrStore, rr_store, ([Shift(IY, d)] [L]) },
        0x1E => regular!{z, 19, mem, Rr, rr, ([Shift(IY, d)]) },
        0x1F => regular!{z, 19, mem, RrStore, rr_store, ([Shift(IY, d)] [A]) },
        0x20 => regular!{z, 19, mem, SlaStore, sla_store, ([Shift(IY, d)] [B]) },
        0x21 => regular!{z, 19, mem, SlaStore, sla_store, ([Shift(IY, d)] [C]) },
        0x22 => regular!{z, 19, mem, SlaStore, sla_store, ([Shift(IY, d)] [D]) },
        0x23 => regular!{z, 19, mem, SlaStore, sla_store, ([Shift(IY, d)] [E]) },
        0x24 => regular!{z, 19, mem, SlaStore, sla_store, ([Shift(IY, d)] [H]) },
        0x25 => regular!{z, 19, mem, SlaStore, sla_store, ([Shift(IY, d)] [L]) },
        0x26 => regular!{z, 19, mem, Sla, sla, ([Shift(IY, d)]) },
        0x27 => regular!{z, 19, mem, SlaStore, sla_store, ([Shift(IY, d)] [A]) },
        0x28 => regular!{z, 19, mem, SraStore, sra_store, ([Shift(IY, d)] [B]) },
        0x29 => regular!{z, 19, mem, SraStore, sra_store, ([Shift(IY, d)] [C]) },
        0x2A => regular!{z, 19, mem, SraStore, sra_store, ([Shift(IY, d)] [D]) },
        0x2B => regular!{z, 19, mem, SraStore, sra_store, ([Shift(IY, d)] [E]) },
        0x2C => regular!{z, 19, mem, SraStore, sra_store, ([Shift(IY, d)] [H]) },
        0x2D => regular!{z, 19, mem, SraStore, sra_store, ([Shift(IY, d)] [L]) },
        0x2E => regular!{z, 19, mem, Sra, sra, ([Shift(IY, d)]) },
        0x2F => regular!{z, 19, mem, SraStore, sra_store, ([Shift(IY, d)] [A]) },
        0x30 => regular!{z, 19, mem, SllStore, sll_store, ([Shift(IY, d)] [B]) },
        0x31 => regular!{z, 19, mem, SllStore, sll_store, ([Shift(IY, d)] [C]) },
        0x32 => regular!{z, 19, mem, SllStore, sll_store, ([Shift(IY, d)] [D]) },
        0x33 => regular!{z, 19, mem, SllStore, sll_store, ([Shift(IY, d)] [E]) },
        0x34 => regular!{z, 19, mem, SllStore, sll_store, ([Shift(IY, d)] [H]) },
        0x35 => regular!{z, 19, mem, SllStore, sll_store, ([Shift(IY, d)] [L]) },
        0x36 => regular!{z, 19, mem, Sll, sll, ([Shift(IY, d)]) },
        0x37 => regular!{z, 19, mem, SllStore, sll_store, ([Shift(IY, d)] [A]) },
        0x38 => regular!{z, 19, mem, SrlStore, srl_store, ([Shift(IY, d)] [B]) },
        0x39 => regular!{z, 19, mem, SrlStore, srl_store, ([Shift(IY, d)] [C]) },
        0x3A => regular!{z, 19, mem, SrlStore, srl_store, ([Shift(IY, d)] [D]) },
        0x3B => regular!{z, 19, mem, SrlStore, srl_store, ([Shift(IY, d)] [E]) },
        0x3C => regular!{z, 19, mem, SrlStore, srl_store, ([Shift(IY, d)] [H]) },
        0x3D => regular!{z, 19, mem, SrlStore, srl_store, ([Shift(IY, d)] [L]) },
        0x3E => regular!{z, 19, mem, Srl, srl, ([Shift(IY, d)]) },
        0x3F => regular!{z, 19, mem, SrlStore, srl_store, ([Shift(IY, d)] [A]) },
        0x40 => regular!{z, 16, mem, Bit, bit, ([0] [Shift(IY, d)]) },
        0x41 => regular!{z, 16, mem, Bit, bit, ([0] [Shift(IY, d)]) },
        0x42 => regular!{z, 16, mem, Bit, bit, ([0] [Shift(IY, d)]) },
        0x43 => regular!{z, 16, mem, Bit, bit, ([0] [Shift(IY, d)]) },
        0x44 => regular!{z, 16, mem, Bit, bit, ([0] [Shift(IY, d)]) },
        0x45 => regular!{z, 16, mem, Bit, bit, ([0] [Shift(IY, d)]) },
        0x46 => regular!{z, 16, mem, Bit, bit, ([0] [Shift(IY, d)]) },
        0x47 => regular!{z, 16, mem, Bit, bit, ([0] [Shift(IY, d)]) },
        0x48 => regular!{z, 16, mem, Bit, bit, ([1] [Shift(IY, d)]) },
        0x49 => regular!{z, 16, mem, Bit, bit, ([1] [Shift(IY, d)]) },
        0x4A => regular!{z, 16, mem, Bit, bit, ([1] [Shift(IY, d)]) },
        0x4B => regular!{z, 16, mem, Bit, bit, ([1] [Shift(IY, d)]) },
        0x4C => regular!{z, 16, mem, Bit, bit, ([1] [Shift(IY, d)]) },
        0x4D => regular!{z, 16, mem, Bit, bit, ([1] [Shift(IY, d)]) },
        0x4E => regular!{z, 16, mem, Bit, bit, ([1] [Shift(IY, d)]) },
        0x4F => regular!{z, 16, mem, Bit, bit, ([1] [Shift(IY, d)]) },
        0x50 => regular!{z, 16, mem, Bit, bit, ([2] [Shift(IY, d)]) },
        0x51 => regular!{z, 16, mem, Bit, bit, ([2] [Shift(IY, d)]) },
        0x52 => regular!{z, 16, mem, Bit, bit, ([2] [Shift(IY, d)]) },
        0x53 => regular!{z, 16, mem, Bit, bit, ([2] [Shift(IY, d)]) },
        0x54 => regular!{z, 16, mem, Bit, bit, ([2] [Shift(IY, d)]) },
        0x55 => regular!{z, 16, mem, Bit, bit, ([2] [Shift(IY, d)]) },
        0x56 => regular!{z, 16, mem, Bit, bit, ([2] [Shift(IY, d)]) },
        0x57 => regular!{z, 16, mem, Bit, bit, ([2] [Shift(IY, d)]) },
        0x58 => regular!{z, 16, mem, Bit, bit, ([3] [Shift(IY, d)]) },
        0x59 => regular!{z, 16, mem, Bit, bit, ([3] [Shift(IY, d)]) },
        0x5A => regular!{z, 16, mem, Bit, bit, ([3] [Shift(IY, d)]) },
        0x5B => regular!{z, 16, mem, Bit, bit, ([3] [Shift(IY, d)]) },
        0x5C => regular!{z, 16, mem, Bit, bit, ([3] [Shift(IY, d)]) },
        0x5D => regular!{z, 16, mem, Bit, bit, ([3] [Shift(IY, d)]) },
        0x5E => regular!{z, 16, mem, Bit, bit, ([3] [Shift(IY, d)]) },
        0x5F => regular!{z, 16, mem, Bit, bit, ([3] [Shift(IY, d)]) },
        0x60 => regular!{z, 16, mem, Bit, bit, ([4] [Shift(IY, d)]) },
        0x61 => regular!{z, 16, mem, Bit, bit, ([4] [Shift(IY, d)]) },
        0x62 => regular!{z, 16, mem, Bit, bit, ([4] [Shift(IY, d)]) },
        0x63 => regular!{z, 16, mem, Bit, bit, ([4] [Shift(IY, d)]) },
        0x64 => regular!{z, 16, mem, Bit, bit, ([4] [Shift(IY, d)]) },
        0x65 => regular!{z, 16, mem, Bit, bit, ([4] [Shift(IY, d)]) },
        0x66 => regular!{z, 16, mem, Bit, bit, ([4] [Shift(IY, d)]) },
        0x67 => regular!{z, 16, mem, Bit, bit, ([4] [Shift(IY, d)]) },
        0x68 => regular!{z, 16, mem, Bit, bit, ([5] [Shift(IY, d)]) },
        0x69 => regular!{z, 16, mem, Bit, bit, ([5] [Shift(IY, d)]) },
        0x6A => regular!{z, 16, mem, Bit, bit, ([5] [Shift(IY, d)]) },
        0x6B => regular!{z, 16, mem, Bit, bit, ([5] [Shift(IY, d)]) },
        0x6C => regular!{z, 16, mem, Bit, bit, ([5] [Shift(IY, d)]) },
        0x6D => regular!{z, 16, mem, Bit, bit, ([5] [Shift(IY, d)]) },
        0x6E => regular!{z, 16, mem, Bit, bit, ([5] [Shift(IY, d)]) },
        0x6F => regular!{z, 16, mem, Bit, bit, ([5] [Shift(IY, d)]) },
        0x70 => regular!{z, 16, mem, Bit, bit, ([6] [Shift(IY, d)]) },
        0x71 => regular!{z, 16, mem, Bit, bit, ([6] [Shift(IY, d)]) },
        0x72 => regular!{z, 16, mem, Bit, bit, ([6] [Shift(IY, d)]) },
        0x73 => regular!{z, 16, mem, Bit, bit, ([6] [Shift(IY, d)]) },
        0x74 => regular!{z, 16, mem, Bit, bit, ([6] [Shift(IY, d)]) },
        0x75 => regular!{z, 16, mem, Bit, bit, ([6] [Shift(IY, d)]) },
        0x76 => regular!{z, 16, mem, Bit, bit, ([6] [Shift(IY, d)]) },
        0x77 => regular!{z, 16, mem, Bit, bit, ([6] [Shift(IY, d)]) },
        0x78 => regular!{z, 16, mem, Bit, bit, ([7] [Shift(IY, d)]) },
        0x79 => regular!{z, 16, mem, Bit, bit, ([7] [Shift(IY, d)]) },
        0x7A => regular!{z, 16, mem, Bit, bit, ([7] [Shift(IY, d)]) },
        0x7B => regular!{z, 16, mem, Bit, bit, ([7] [Shift(IY, d)]) },
        0x7C => regular!{z, 16, mem, Bit, bit, ([7] [Shift(IY, d)]) },
        0x7D => regular!{z, 16, mem, Bit, bit, ([7] [Shift(IY, d)]) },
        0x7E => regular!{z, 16, mem, Bit, bit, ([7] [Shift(IY, d)]) },
        0x7F => regular!{z, 16, mem, Bit, bit, ([7] [Shift(IY, d)]) },
        0x80 => regular!{z, 19, mem, ResStore, res_store, ([0] [Shift(IY, d)] [B]) },
        0x81 => regular!{z, 19, mem, ResStore, res_store, ([0] [Shift(IY, d)] [C]) },
        0x82 => regular!{z, 19, mem, ResStore, res_store, ([0] [Shift(IY, d)] [D]) },
        0x83 => regular!{z, 19, mem, ResStore, res_store, ([0] [Shift(IY, d)] [E]) },
        0x84 => regular!{z, 19, mem, ResStore, res_store, ([0] [Shift(IY, d)] [H]) },
        0x85 => regular!{z, 19, mem, ResStore, res_store, ([0] [Shift(IY, d)] [L]) },
        0x86 => regular!{z, 19, mem, Res, res, ([0] [Shift(IY, d)]) },
        0x87 => regular!{z, 19, mem, ResStore, res_store, ([0] [Shift(IY, d)] [A]) },
        0x88 => regular!{z, 19, mem, ResStore, res_store, ([1] [Shift(IY, d)] [B]) },
        0x89 => regular!{z, 19, mem, ResStore, res_store, ([1] [Shift(IY, d)] [C]) },
        0x8A => regular!{z, 19, mem, ResStore, res_store, ([1] [Shift(IY, d)] [D]) },
        0x8B => regular!{z, 19, mem, ResStore, res_store, ([1] [Shift(IY, d)] [E]) },
        0x8C => regular!{z, 19, mem, ResStore, res_store, ([1] [Shift(IY, d)] [H]) },
        0x8D => regular!{z, 19, mem, ResStore, res_store, ([1] [Shift(IY, d)] [L]) },
        0x8E => regular!{z, 19, mem, Res, res, ([1] [Shift(IY, d)]) },
        0x8F => regular!{z, 19, mem, ResStore, res_store, ([1] [Shift(IY, d)] [A]) },
        0x90 => regular!{z, 19, mem, ResStore, res_store, ([2] [Shift(IY, d)] [B]) },
        0x91 => regular!{z, 19, mem, ResStore, res_store, ([2] [Shift(IY, d)] [C]) },
        0x92 => regular!{z, 19, mem, ResStore, res_store, ([2] [Shift(IY, d)] [D]) },
        0x93 => regular!{z, 19, mem, ResStore, res_store, ([2] [Shift(IY, d)] [E]) },
        0x94 => regular!{z, 19, mem, ResStore, res_store, ([2] [Shift(IY, d)] [H]) },
        0x95 => regular!{z, 19, mem, ResStore, res_store, ([2] [Shift(IY, d)] [L]) },
        0x96 => regular!{z, 19, mem, Res, res, ([2] [Shift(IY, d)]) },
        0x97 => regular!{z, 19, mem, ResStore, res_store, ([2] [Shift(IY, d)] [A]) },
        0x98 => regular!{z, 19, mem, ResStore, res_store, ([3] [Shift(IY, d)] [B]) },
        0x99 => regular!{z, 19, mem, ResStore, res_store, ([3] [Shift(IY, d)] [C]) },
        0x9A => regular!{z, 19, mem, ResStore, res_store, ([3] [Shift(IY, d)] [D]) },
        0x9B => regular!{z, 19, mem, ResStore, res_store, ([3] [Shift(IY, d)] [E]) },
        0x9C => regular!{z, 19, mem, ResStore, res_store, ([3] [Shift(IY, d)] [H]) },
        0x9D => regular!{z, 19, mem, ResStore, res_store, ([3] [Shift(IY, d)] [L]) },
        0x9E => regular!{z, 19, mem, Res, res, ([3] [Shift(IY, d)]) },
        0x9F => regular!{z, 19, mem, ResStore, res_store, ([3] [Shift(IY, d)] [A]) },
        0xA0 => regular!{z, 19, mem, ResStore, res_store, ([4] [Shift(IY, d)] [B]) },
        0xA1 => regular!{z, 19, mem, ResStore, res_store, ([4] [Shift(IY, d)] [C]) },
        0xA2 => regular!{z, 19, mem, ResStore, res_store, ([4] [Shift(IY, d)] [D]) },
        0xA3 => regular!{z, 19, mem, ResStore, res_store, ([4] [Shift(IY, d)] [E]) },
        0xA4 => regular!{z, 19, mem, ResStore, res_store, ([4] [Shift(IY, d)] [H]) },
        0xA5 => regular!{z, 19, mem, ResStore, res_store, ([4] [Shift(IY, d)] [L]) },
        0xA6 => regular!{z, 19, mem, Res, res, ([4] [Shift(IY, d)]) },
        0xA7 => regular!{z, 19, mem, ResStore, res_store, ([4] [Shift(IY, d)] [A]) },
        0xA8 => regular!{z, 19, mem, ResStore, res_store, ([5] [Shift(IY, d)] [B]) },
        0xA9 => regular!{z, 19, mem, ResStore, res_store, ([5] [Shift(IY, d)] [C]) },
        0xAA => regular!{z, 19, mem, ResStore, res_store, ([5] [Shift(IY, d)] [D]) },
        0xAB => regular!{z, 19, mem, ResStore, res_store, ([5] [Shift(IY, d)] [E]) },
        0xAC => regular!{z, 19, mem, ResStore, res_store, ([5] [Shift(IY, d)] [H]) },
        0xAD => regular!{z, 19, mem, ResStore, res_store, ([5] [Shift(IY, d)] [L]) },
        0xAE => regular!{z, 19, mem, Res, res, ([5] [Shift(IY, d)]) },
        0xAF => regular!{z, 19, mem, ResStore, res_store, ([5] [Shift(IY, d)] [A]) },
        0xB0 => regular!{z, 19, mem, ResStore, res_store, ([6] [Shift(IY, d)] [B]) },
        0xB1 => regular!{z, 19, mem, ResStore, res_store, ([6] [Shift(IY, d)] [C]) },
        0xB2 => regular!{z, 19, mem, ResStore, res_store, ([6] [Shift(IY, d)] [D]) },
        0xB3 => regular!{z, 19, mem, ResStore, res_store, ([6] [Shift(IY, d)] [E]) },
        0xB4 => regular!{z, 19, mem, ResStore, res_store, ([6] [Shift(IY, d)] [H]) },
        0xB5 => regular!{z, 19, mem, ResStore, res_store, ([6] [Shift(IY, d)] [L]) },
        0xB6 => regular!{z, 19, mem, Res, res, ([6] [Shift(IY, d)]) },
        0xB7 => regular!{z, 19, mem, ResStore, res_store, ([6] [Shift(IY, d)] [A]) },
        0xB8 => regular!{z, 19, mem, ResStore, res_store, ([7] [Shift(IY, d)] [B]) },
        0xB9 => regular!{z, 19, mem, ResStore, res_store, ([7] [Shift(IY, d)] [C]) },
        0xBA => regular!{z, 19, mem, ResStore, res_store, ([7] [Shift(IY, d)] [D]) },
        0xBB => regular!{z, 19, mem, ResStore, res_store, ([7] [Shift(IY, d)] [E]) },
        0xBC => regular!{z, 19, mem, ResStore, res_store, ([7] [Shift(IY, d)] [H]) },
        0xBD => regular!{z, 19, mem, ResStore, res_store, ([7] [Shift(IY, d)] [L]) },
        0xBE => regular!{z, 19, mem, Res, res, ([7] [Shift(IY, d)]) },
        0xBF => regular!{z, 19, mem, ResStore, res_store, ([7] [Shift(IY, d)] [A]) },
        0xC0 => regular!{z, 19, mem, SetStore, set_store, ([0] [Shift(IY, d)] [B]) },
        0xC1 => regular!{z, 19, mem, SetStore, set_store, ([0] [Shift(IY, d)] [C]) },
        0xC2 => regular!{z, 19, mem, SetStore, set_store, ([0] [Shift(IY, d)] [D]) },
        0xC3 => regular!{z, 19, mem, SetStore, set_store, ([0] [Shift(IY, d)] [E]) },
        0xC4 => regular!{z, 19, mem, SetStore, set_store, ([0] [Shift(IY, d)] [H]) },
        0xC5 => regular!{z, 19, mem, SetStore, set_store, ([0] [Shift(IY, d)] [L]) },
        0xC6 => regular!{z, 19, mem, Set, set, ([0] [Shift(IY, d)]) },
        0xC7 => regular!{z, 19, mem, SetStore, set_store, ([0] [Shift(IY, d)] [A]) },
        0xC8 => regular!{z, 19, mem, SetStore, set_store, ([1] [Shift(IY, d)] [B]) },
        0xC9 => regular!{z, 19, mem, SetStore, set_store, ([1] [Shift(IY, d)] [C]) },
        0xCA => regular!{z, 19, mem, SetStore, set_store, ([1] [Shift(IY, d)] [D]) },
        0xCB => regular!{z, 19, mem, SetStore, set_store, ([1] [Shift(IY, d)] [E]) },
        0xCC => regular!{z, 19, mem, SetStore, set_store, ([1] [Shift(IY, d)] [H]) },
        0xCD => regular!{z, 19, mem, SetStore, set_store, ([1] [Shift(IY, d)] [L]) },
        0xCE => regular!{z, 19, mem, Set, set, ([1] [Shift(IY, d)]) },
        0xCF => regular!{z, 19, mem, SetStore, set_store, ([1] [Shift(IY, d)] [A]) },
        0xD0 => regular!{z, 19, mem, SetStore, set_store, ([2] [Shift(IY, d)] [B]) },
        0xD1 => regular!{z, 19, mem, SetStore, set_store, ([2] [Shift(IY, d)] [C]) },
        0xD2 => regular!{z, 19, mem, SetStore, set_store, ([2] [Shift(IY, d)] [D]) },
        0xD3 => regular!{z, 19, mem, SetStore, set_store, ([2] [Shift(IY, d)] [E]) },
        0xD4 => regular!{z, 19, mem, SetStore, set_store, ([2] [Shift(IY, d)] [H]) },
        0xD5 => regular!{z, 19, mem, SetStore, set_store, ([2] [Shift(IY, d)] [L]) },
        0xD6 => regular!{z, 19, mem, Set, set, ([2] [Shift(IY, d)]) },
        0xD7 => regular!{z, 19, mem, SetStore, set_store, ([2] [Shift(IY, d)] [A]) },
        0xD8 => regular!{z, 19, mem, SetStore, set_store, ([3] [Shift(IY, d)] [B]) },
        0xD9 => regular!{z, 19, mem, SetStore, set_store, ([3] [Shift(IY, d)] [C]) },
        0xDA => regular!{z, 19, mem, SetStore, set_store, ([3] [Shift(IY, d)] [D]) },
        0xDB => regular!{z, 19, mem, SetStore, set_store, ([3] [Shift(IY, d)] [E]) },
        0xDC => regular!{z, 19, mem, SetStore, set_store, ([3] [Shift(IY, d)] [H]) },
        0xDD => regular!{z, 19, mem, SetStore, set_store, ([3] [Shift(IY, d)] [L]) },
        0xDE => regular!{z, 19, mem, Set, set, ([3] [Shift(IY, d)]) },
        0xDF => regular!{z, 19, mem, SetStore, set_store, ([3] [Shift(IY, d)] [A]) },
        0xE0 => regular!{z, 19, mem, SetStore, set_store, ([4] [Shift(IY, d)] [B]) },
        0xE1 => regular!{z, 19, mem, SetStore, set_store, ([4] [Shift(IY, d)] [C]) },
        0xE2 => regular!{z, 19, mem, SetStore, set_store, ([4] [Shift(IY, d)] [D]) },
        0xE3 => regular!{z, 19, mem, SetStore, set_store, ([4] [Shift(IY, d)] [E]) },
        0xE4 => regular!{z, 19, mem, SetStore, set_store, ([4] [Shift(IY, d)] [H]) },
        0xE5 => regular!{z, 19, mem, SetStore, set_store, ([4] [Shift(IY, d)] [L]) },
        0xE6 => regular!{z, 19, mem, Set, set, ([4] [Shift(IY, d)]) },
        0xE7 => regular!{z, 19, mem, SetStore, set_store, ([4] [Shift(IY, d)] [A]) },
        0xE8 => regular!{z, 19, mem, SetStore, set_store, ([5] [Shift(IY, d)] [B]) },
        0xE9 => regular!{z, 19, mem, SetStore, set_store, ([5] [Shift(IY, d)] [C]) },
        0xEA => regular!{z, 19, mem, SetStore, set_store, ([5] [Shift(IY, d)] [D]) },
        0xEB => regular!{z, 19, mem, SetStore, set_store, ([5] [Shift(IY, d)] [E]) },
        0xEC => regular!{z, 19, mem, SetStore, set_store, ([5] [Shift(IY, d)] [H]) },
        0xED => regular!{z, 19, mem, SetStore, set_store, ([5] [Shift(IY, d)] [L]) },
        0xEE => regular!{z, 19, mem, Set, set, ([5] [Shift(IY, d)]) },
        0xEF => regular!{z, 19, mem, SetStore, set_store, ([5] [Shift(IY, d)] [A]) },
        0xF0 => regular!{z, 19, mem, SetStore, set_store, ([6] [Shift(IY, d)] [B]) },
        0xF1 => regular!{z, 19, mem, SetStore, set_store, ([6] [Shift(IY, d)] [C]) },
        0xF2 => regular!{z, 19, mem, SetStore, set_store, ([6] [Shift(IY, d)] [D]) },
        0xF3 => regular!{z, 19, mem, SetStore, set_store, ([6] [Shift(IY, d)] [E]) },
        0xF4 => regular!{z, 19, mem, SetStore, set_store, ([6] [Shift(IY, d)] [H]) },
        0xF5 => regular!{z, 19, mem, SetStore, set_store, ([6] [Shift(IY, d)] [L]) },
        0xF6 => regular!{z, 19, mem, Set, set, ([6] [Shift(IY, d)]) },
        0xF7 => regular!{z, 19, mem, SetStore, set_store, ([6] [Shift(IY, d)] [A]) },
        0xF8 => regular!{z, 19, mem, SetStore, set_store, ([7] [Shift(IY, d)] [B]) },
        0xF9 => regular!{z, 19, mem, SetStore, set_store, ([7] [Shift(IY, d)] [C]) },
        0xFA => regular!{z, 19, mem, SetStore, set_store, ([7] [Shift(IY, d)] [D]) },
        0xFB => regular!{z, 19, mem, SetStore, set_store, ([7] [Shift(IY, d)] [E]) },
        0xFC => regular!{z, 19, mem, SetStore, set_store, ([7] [Shift(IY, d)] [H]) },
        0xFD => regular!{z, 19, mem, SetStore, set_store, ([7] [Shift(IY, d)] [L]) },
        0xFE => regular!{z, 19, mem, Set, set, ([7] [Shift(IY, d)]) },
        0xFF => regular!{z, 19, mem, SetStore, set_store, ([7] [Shift(IY, d)] [A]) },
        _ => unimplemented!(),
    }
}
