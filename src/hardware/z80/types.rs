// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use std;
use std::fmt;

use ::utilities;
use ::memo::Outbox;

/// The Z80 processor.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Component {
    pub halted: bool,
    pub cycles: u64,
    pub address: u16,
    pub data: u8,

    /// Represents the iff1 flag, determining whether maskable interrupts are
    /// accepted.
    ///
    /// The Z80 `ei` instruction is supposed to set iff1, but then interrupts
    /// aren't supposed to actually be accepted until after the following
    /// instruction. To emulate this, my `ei` implementation sets the `iff1`
    /// field to the current value of `cycles`. Then when an interrupt is
    /// desired, the function `maskable_interrupt` first checks to see if
    /// `cycles` is larger than `iff1`. `di` sets `iff1` to 0xFFFFFFFFFFFFFFFF.
    pub iff1: u64,
    pub iff2: bool,
    pub interrupt_mode: InterruptMode,
    pub registers: [u16; 13],
    id: u32,
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "SZYHXPNC  A   BC   DE   HL   IX   IY   SP   PC\n\
             {:0>8b} {:0>2X} {:0>4X} {:0>4X} {:0>4X} {:0>4X} {:0>4X} {:0>4X} {:0>4X}\n",
            self.get_reg8(F),
            self.get_reg8(A),
            self.get_reg16(BC),
            self.get_reg16(DE),
            self.get_reg16(HL),
            self.get_reg16(IX),
            self.get_reg16(IY),
            self.get_reg16(SP),
            self.get_reg16(PC),
        )
    }
}

#[cfg(target_endian = "little")]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Matchable)]
pub enum Reg8 {
    C, B, E, D, F, A, L, H,
    C0, B0, E0, D0, F0, A0, L0, H0,
    IXL, IXH, IYL, IYH,
    SPL, SPH, PCL, PCH, I, R
}

#[cfg(target_endian = "big")]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Matchable)]
pub enum Reg8 {
    B, C, D, E, A, F, H, L,
    B0, C0, D0, E0, A0, F0, H0, L0,
    IXH, IXL, IYH, IYL,
    SPH, SPL, PCH, PCL, I, R
}

impl fmt::Display for Reg8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match *self {
            B => "b",
            C => "c",
            D => "d",
            E => "e",
            A => "a",
            F => "f",
            H => "h",
            L => "l",
            B0 => "b'",
            C0 => "c'",
            D0 => "d'",
            E0 => "e'",
            A0 => "a'",
            F0 => "f'",
            H0 => "h'",
            L0 => "l'",
            IXL => "ixl",
            IXH => "ixh",
            IYL => "iyl",
            IYH => "iyh",
            SPL => "spl",
            SPH => "sph",
            PCL => "pcl",
            PCH => "pch",
            I => "i",
            R => "r",
        };
        f.pad(s)
    }
}

pub use self::Reg8::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Matchable)]
pub enum Reg16 {
    BC, DE, AF, HL,
    BC0, DE0, AF0, HL0,
    IX, IY,
    SP, PC
}

impl fmt::Display for Reg16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match *self {
            BC => "bc",
            DE => "de",
            AF => "af",
            HL => "hl",
            BC0 => "bc'",
            DE0 => "de'",
            AF0 => "af'",
            HL0 => "hl'",
            IX => "ix",
            IY => "iy",
            SP => "sp",
            PC => "pc",
        };
        f.pad(s)
    }
}

pub use self::Reg16::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Matchable)]
pub enum ConditionCode {
    NZcc, Zcc, NCcc, Ccc, POcc, PEcc, Pcc, Mcc
}

pub use self::ConditionCode::*;

impl fmt::Display for ConditionCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match *self {
            NZcc => "nz",
            Zcc => "z",
            NCcc => "nc",
            Ccc => "c",
            POcc => "po",
            PEcc => "pe",
            Pcc => "p",
            Mcc => "m",
        };
        f.pad(&s)
    }
}

bitflags! {
    pub struct Flags: u8 {
        const CF = 1 << 0;
        const NF = 1 << 1;
        const PF = 1 << 2;
        const XF = 1 << 3;
        const HF = 1 << 4;
        const YF = 1 << 5;
        const ZF = 1 << 6;
        const SF = 1 << 7;
    }
}

impl Flags {
    pub fn set_sign(&mut self, x: u8)
    {
        self.set(SF, x & 0x80 != 0);
    }

    pub fn set_zero(&mut self, x: u8)
    {
        self.set(ZF, x == 0);
    }

    pub fn set_parity(&mut self, x: u8)
    {
        let parity = x.count_ones() % 2 == 0;
        self.set(PF, parity);
    }

}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Matchable)]
pub enum InterruptMode {
    Im0, Im1, Im2
}

pub use self::InterruptMode::*;

impl Default for InterruptMode {
    fn default() -> InterruptMode { Im0 }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Matchable)]
pub struct Address<T>(pub T);

impl fmt::Display for Address<Reg16> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = format!("({})", self.0);
        f.pad(&s)
    }
}

impl fmt::Display for Address<u16> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = format!("({:<#0X})", self.0);
        f.pad(&s)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Matchable)]
pub struct Shift(pub Reg16, pub i8);

impl fmt::Display for Shift {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = format!("({}{:<+#0X})", self.0, self.1);
        f.pad(&s)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize, Matchable)]
pub enum Opcode {
    OneByte([u8; 1]),
    TwoBytes([u8; 2]),
    ThreeBytes([u8; 3]),
    FourBytes([u8; 4]),
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        use std::fmt::Write;
        let slice: &[u8] = match self {
            &Opcode::OneByte(ref x) => x,
            &Opcode::TwoBytes(ref x) => x,
            &Opcode::ThreeBytes(ref x) => x,
            &Opcode::FourBytes(ref x) => x,
        };
        let mut s = "".to_owned();
        write!(s, "{:0<2X}", slice[0])?;
        for x in slice[1 ..].iter() {
            write!(s, " {:0<2X}", x)?;
        };
        f.pad(&s)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Matchable)]
pub enum Parameter {
    Reg8(Reg8), Reg16(Reg16), Shift(Shift), AddressReg16(Address<Reg16>),
    AddressU16(Address<u16>), U8(u8), I8(i8), U16(u16), Cc(ConditionCode),
}

impl fmt::Display for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        match *self {
            Parameter::Reg8(x) => x.fmt(f),
            Parameter::Reg16(x) => x.fmt(f),
            Parameter::Shift(x) => x.fmt(f),
            Parameter::AddressReg16(x) => x.fmt(f),
            Parameter::AddressU16(x) => x.fmt(f),
            Parameter::U8(x) => x.fmt(f),
            Parameter::I8(x) => x.fmt(f),
            Parameter::U16(x) => x.fmt(f),
            Parameter::Cc(x) => x.fmt(f),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Matchable)]
pub enum Mnemonic {
    Ld, Push, Pop, Ex, Exx, Ldi, Ldir, Ldd, Lddr, Cpi, Cpir, Cpd, Cpdr,
    Add, Adc, Sub, Sbc, And, Or, Xor, Cp, Inc, Dec, Daa, Cpl, Neg, Ccf,
    Scf, Nop, Halt, Di, Ei, Im, Rlca, Rla, Rrca, Rra, Rlc, Rl, Rrc, Rr, Sla,
    Sra, Sll, Srl, Rld, Rrd, Bit, Set, Res, Jp, Jr, Djnz, Call, Ret, Reti, Retn, Rst,
    In, Ini, Inir, Ind, Indr, Out, Outi, Otir, Outd, Otdr,
}

impl fmt::Display for Mnemonic {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        let s = format!("{:?}", self);
        let lower = s.to_lowercase();
        f.pad(&lower)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Matchable)]
pub enum FullMnemonic {
    ZeroParameters(Mnemonic),
    OneParameter(Mnemonic, Parameter),
    TwoParameters(Mnemonic, Parameter, Parameter),
    ThreeParameters(Mnemonic, Parameter, Parameter, Parameter),
}

impl fmt::Display for FullMnemonic {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        let s = match self {
            &FullMnemonic::ZeroParameters(f) => format!("{}", f),
            &FullMnemonic::OneParameter(f, p1) => format!("{} {}", f, p1),
            &FullMnemonic::TwoParameters(f, p1, p2) => format!("{} {}, {}", f, p1, p2),
            &FullMnemonic::ThreeParameters(f, p1, p2, p3) => format!("{} {}, {}, {}", f, p1, p2, p3),
        };
        f.pad(&s)
    }
}

macro_rules! function_to_mnemonic {
    (ld) => { Mnemonic::Ld };
    (ld_ir) => { Mnemonic::Ld };
    (ld16) => { Mnemonic::Ld };
    (push) => { Mnemonic::Push };
    (pop) => { Mnemonic::Pop };
    (ex) => { Mnemonic::Ex };
    (exx) => { Mnemonic::Exx };
    (ldi) => { Mnemonic::Ldi };
    (ldir) => { Mnemonic::Ldir };
    (ldd) => { Mnemonic::Ldd };
    (lddr) => { Mnemonic::Lddr };
    (cpi) => { Mnemonic::Cpi };
    (cpir) => { Mnemonic::Cpir };
    (cpd) => { Mnemonic::Cpd };
    (cpdr) => { Mnemonic::Cpdr };
    (add) => { Mnemonic::Add };
    (add16) => { Mnemonic::Add };
    (adc) => { Mnemonic::Adc };
    (adc16) => { Mnemonic::Adc };
    (sub) => { Mnemonic::Sub };
    (sbc) => { Mnemonic::Sbc };
    (sbc16) => { Mnemonic::Sbc };
    (and) => { Mnemonic::And };
    (or) => { Mnemonic::Or };
    (xor) => { Mnemonic::Xor };
    (cp) => { Mnemonic::Cp };
    (inc) => { Mnemonic::Inc };
    (inc16) => { Mnemonic::Inc };
    (dec) => { Mnemonic::Dec };
    (dec16) => { Mnemonic::Dec };
    (daa) => { Mnemonic::Daa };
    (cpl) => { Mnemonic::Cpl };
    (neg) => { Mnemonic::Neg };
    (ccf) => { Mnemonic::Ccf };
    (scf) => { Mnemonic::Scf };
    (nop) => { Mnemonic::Nop };
    (halt) => { Mnemonic::Halt };
    (di) => { Mnemonic::Di };
    (ei) => { Mnemonic::Ei };
    (im) => { Mnemonic::Im };
    (rlca) => { Mnemonic::Rlca };
    (rla) => { Mnemonic::Rla };
    (rrca) => { Mnemonic::Rrca };
    (rra) => { Mnemonic::Rra };
    (rlc) => { Mnemonic::Rlc };
    (rlc_store) => { Mnemonic::Rlc };
    (rl) => { Mnemonic::Rl };
    (rl_store) => { Mnemonic::Rl };
    (rrc) => { Mnemonic::Rrc };
    (rrc_store) => { Mnemonic::Rrc };
    (rr) => { Mnemonic::Rr };
    (rr_store) => { Mnemonic::Rr };
    (sla) => { Mnemonic::Sla };
    (sla_store) => { Mnemonic::Sla };
    (sra) => { Mnemonic::Sra };
    (sra_store) => { Mnemonic::Sra };
    (sll) => { Mnemonic::Sll };
    (sll_store) => { Mnemonic::Sll };
    (srl) => { Mnemonic::Srl };
    (srl_store) => { Mnemonic::Srl };
    (rld) => { Mnemonic::Rld };
    (rrd) => { Mnemonic::Rrd };
    (bit) => { Mnemonic::Bit };
    (set) => { Mnemonic::Set };
    (set_store) => { Mnemonic::Set };
    (res) => { Mnemonic::Res };
    (res_store) => { Mnemonic::Res };
    (jp) => { Mnemonic::Jp };
    (jpcc) => { Mnemonic::Jp };
    (jr) => { Mnemonic::Jr };
    (jrcc) => { Mnemonic::Jr };
    (djnz) => { Mnemonic::Djnz };
    (call) => { Mnemonic::Call };
    (callcc) => { Mnemonic::Call };
    (ret) => { Mnemonic::Ret };
    (retcc) => { Mnemonic::Ret };
    (reti) => { Mnemonic::Reti };
    (retn) => { Mnemonic::Retn };
    (rst) => { Mnemonic::Rst };
    (in_c) => { Mnemonic::In };
    (in_f) => { Mnemonic::In };
    (in_n) => { Mnemonic::In };
    (ini) => { Mnemonic::Ini };
    (inir) => { Mnemonic::Inir };
    (ind) => { Mnemonic::Ind };
    (indr) => { Mnemonic::Indr };
    (out_c) => { Mnemonic::Out };
    (out_n) => { Mnemonic::Out };
    (outi) => { Mnemonic::Outi };
    (otir) => { Mnemonic::Otir };
    (outd) => { Mnemonic::Outd };
    (otdr) => { Mnemonic::Otdr };
}

impl Opcode {
    pub fn mnemonic(&self) -> Option<FullMnemonic> {
        // rustc insists these do not need to be mutable. Somehow it isn't
        // seeing the assignments behind the macros?
        let n: u8;
        let d: i8;
        let e: i8;
        let nn: u16;

        macro_rules! translate_parameter {
            (A) => { Parameter::Reg8(A) };
            (F) => { Parameter::Reg8(F) };
            (B) => { Parameter::Reg8(B) };
            (C) => { Parameter::Reg8(C) };
            (D) => { Parameter::Reg8(D) };
            (E) => { Parameter::Reg8(E) };
            (H) => { Parameter::Reg8(H) };
            (L) => { Parameter::Reg8(L) };
            (A0) => { Parameter::Reg8(A0) };
            (F0) => { Parameter::Reg8(F0) };
            (B0) => { Parameter::Reg8(B0) };
            (C0) => { Parameter::Reg8(C0) };
            (D0) => { Parameter::Reg8(D0) };
            (E0) => { Parameter::Reg8(E0) };
            (H0) => { Parameter::Reg8(H0) };
            (L0) => { Parameter::Reg8(L0) };
            (IXL) => { Parameter::Reg8(IXL) };
            (IXH) => { Parameter::Reg8(IXH) };
            (IYL) => { Parameter::Reg8(IYL) };
            (IYH) => { Parameter::Reg8(IYH) };
            (SPL) => { Parameter::Reg8(SPL) };
            (SPH) => { Parameter::Reg8(SPH) };
            (PCL) => { Parameter::Reg8(PCL) };
            (PCH) => { Parameter::Reg8(PCH) };
            (I) => { Parameter::Reg8(I) };
            (R) => { Parameter::Reg8(R) };
            (BC) => { Parameter::Reg16(BC) };
            (DE) => { Parameter::Reg16(DE) };
            (AF) => { Parameter::Reg16(AF) };
            (HL) => { Parameter::Reg16(HL) };
            (BC0) => { Parameter::Reg16(BC0) };
            (DE0) => { Parameter::Reg16(DE0) };
            (AF0) => { Parameter::Reg16(AF0) };
            (HL0) => { Parameter::Reg16(HL0) };
            (IX) => { Parameter::Reg16(IX) };
            (IY) => { Parameter::Reg16(IY) };
            (SP) => { Parameter::Reg16(SP) };
            (PC) => { Parameter::Reg16(PC) };
            ((IX+d)) => { Parameter::Shift(Shift(IX, d)) };
            ((IY+d)) => { Parameter::Shift(Shift(IY, d)) };
            ((BC)) => { Parameter::AddressReg16(Address(BC)) };
            ((DE)) => { Parameter::AddressReg16(Address(DE)) };
            ((HL)) => { Parameter::AddressReg16(Address(HL)) };
            ((SP)) => { Parameter::AddressReg16(Address(SP)) };
            ((nn)) => { Parameter::AddressU16(Address(nn)) };
            (n) => { Parameter::U8(n) };
            (d) => { Parameter::I8(d) };
            (e) => { Parameter::I8(e) };
            (nn) => { Parameter::U16(nn) };
            (0) => { Parameter::U8(0) };
            (1) => { Parameter::U8(1) };
            (2) => { Parameter::U8(2) };
            (3) => { Parameter::U8(3) };
            (4) => { Parameter::U8(4) };
            (5) => { Parameter::U8(5) };
            (6) => { Parameter::U8(6) };
            (7) => { Parameter::U8(7) };
            (NZcc) => { Parameter::Cc(NZcc) };
            (Zcc) => { Parameter::Cc(Zcc) };
            (NCcc) => { Parameter::Cc(NCcc) };
            (Ccc) => { Parameter::Cc(Ccc) };
            (POcc) => { Parameter::Cc(POcc) };
            (PEcc) => { Parameter::Cc(PEcc) };
            (Pcc) => { Parameter::Cc(Pcc) };
            (Mcc) => { Parameter::Cc(Mcc) };
        }

        macro_rules! make_full_mnemonic {
            ($function_name: ident, []) => {
                FullMnemonic::ZeroParameters(function_to_mnemonic!($function_name))
            };
            ($function_name: ident, [$a: tt]) => {
                FullMnemonic::OneParameter(
                    function_to_mnemonic!($function_name),
                    translate_parameter!($a),
                )
            };
            ($function_name: ident, [$a: tt, $b: tt]) => {
                FullMnemonic::TwoParameters(
                    function_to_mnemonic!($function_name),
                    translate_parameter!($a),
                    translate_parameter!($b),
                )
            };
            ($function_name: ident, [$a: tt, $b: tt, $c: tt]) => {
                FullMnemonic::ThreeParameters(
                    function_to_mnemonic!($function_name),
                    translate_parameter!($a),
                    translate_parameter!($b),
                    translate_parameter!($c),
                )
            };
        }

        macro_rules! find_code {
            // rst needs to be handled separately, as it's the only one with a u16 literal and
            // this is an easy way to distinguish it from a u8 literal
            ([$code: expr] ; rst ; [ $arg: expr ] ; $t_states: expr; $is_undoc: expr) => {
                if let &Opcode::OneByte(x) = self {
                    if $code == x[0] {
                        return Some(FullMnemonic::OneParameter(Mnemonic::Rst, Parameter::U16($arg)));
                    }
                }
            };
            ([$code: expr, n, n] ; $mnemonic: ident ; $arg_list: tt ; $t_states: expr ; $is_undoc: expr ) => {
                if let &Opcode::ThreeBytes(x) = self {
                    if $code == x[0] {
                        nn = utilities::to16(x[1], x[2]);
                        return Some(make_full_mnemonic!($mnemonic, $arg_list));
                    }
                }
            };
            ([$code: expr, e] ; $mnemonic: ident ; $arg_list: tt ; $t_states: expr ; $is_undoc: expr ) => {
                if let &Opcode::TwoBytes(x) = self {
                    if $code == x[0] {
                        e = x[1] as i8;
                        return Some(make_full_mnemonic!($mnemonic, $arg_list));
                    }
                }
            };
            ([$code: expr, d] ; $mnemonic: ident ; $arg_list: tt ; $t_states: expr ; $is_undoc: expr ) => {
                if let &Opcode::TwoBytes(x) = self {
                    if $code == x[0] {
                        d = x[1] as i8;
                        return Some(make_full_mnemonic!($mnemonic, $arg_list));
                    }
                }
            };
            ([$code: expr, n] ; $mnemonic: ident ; $arg_list: tt ; $t_states: expr ; $is_undoc: expr ) => {
                if let &Opcode::TwoBytes(x) = self {
                    if $code == x[0] {
                        n = x[1];
                        return Some(make_full_mnemonic!($mnemonic, $arg_list));
                    }
                }
            };
            ([$code1: expr, $code2: expr, n] ; $mnemonic: ident ; $arg_list: tt ; $t_states: expr ; $is_undoc: expr ) => {
                if let &Opcode::ThreeBytes(x) = self {
                    if $code1 == x[0] && $code2 == x[1] {
                        n = x[2];
                        return Some(make_full_mnemonic!($mnemonic, $arg_list));
                    }
                }
            };
            ([$code1: expr, $code2: expr, d] ; $mnemonic: ident ; $arg_list: tt ; $t_states: expr ; $is_undoc: expr ) => {
                if let &Opcode::ThreeBytes(x) = self {
                    if $code1 == x[0] && $code2 == x[1] {
                        d = x[2] as i8;
                        return Some(make_full_mnemonic!($mnemonic, $arg_list));
                    }
                }
            };
            ([$code1: expr, $code2: expr, d, n] ; $mnemonic: ident ; $arg_list: tt ; $t_states: expr ; $is_undoc: expr ) => {
                if let &Opcode::FourBytes(x) = self {
                    if $code1 == x[0] && $code2 == x[1] {
                        d = x[2] as i8;
                        n = x[3] as u8;
                        return Some(make_full_mnemonic!($mnemonic, $arg_list));
                    }
                }
            };
            ([$code1: expr, $code2: expr, n, n] ; $mnemonic: ident ; $arg_list: tt ; $t_states: expr ; $is_undoc: expr ) => {
                if let &Opcode::FourBytes(x) = self {
                    if $code1 == x[0] && $code2 == x[1] {
                        nn = utilities::to16(x[2], x[3]);
                        return Some(make_full_mnemonic!($mnemonic, $arg_list));
                    }
                }
            };
            ([$code1: expr, $code2: expr, d, $code3: expr] ; $mnemonic: ident ; $arg_list: tt ; $t_states: expr ; $is_undoc: expr ) => {
                if let &Opcode::FourBytes(x) = self {
                    if $code1 == x[0] && $code2 == x[1]  && $code3 == x[3] {
                        d = x[3] as i8;
                        return Some(make_full_mnemonic!($mnemonic, $arg_list));
                    }
                }
            };
            ([$code: expr] ; $mnemonic: ident ; $arg_list: tt ; $t_states: expr ; $is_undoc: expr ) => {
                if let &Opcode::OneByte(x) = self {
                    if $code == x[0] {
                        return Some(make_full_mnemonic!($mnemonic, $arg_list));
                    }
                }
            };
            ([$code1: expr, $code2: expr] ; $mnemonic: ident ; $arg_list: tt ; $t_states: expr ; $is_undoc: expr ) => {
                if let &Opcode::TwoBytes(x) = self {
                    if $code1 == x[0] && $code2 == x[1] {
                        return Some(make_full_mnemonic!($mnemonic, $arg_list));
                    }
                }
            };
        }

        process_instructions!(find_code, d, e, n, nn);

        return None;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Matchable)]
pub enum Memo {
    Reg8Changed {
        register: Reg8,
        old_value: u8,
        new_value: u8,
    },

    Reg16Changed {
        register: Reg16,
        old_value: u16,
        new_value: u16,
    },

    ReadingPcToExecute(u16),
    InstructionAtPc(u16),
    InstructionOpcode(Opcode),

    MaskableInterruptDenied,
    MaskableInterruptAllowed,
    NonmaskableInterrupt,
}

impl Outbox for Component
{
    type Memo = Memo;

    fn id(&self) -> u32 { self.id }

    fn set_id(&mut self, id: u32) { self.id = id; }
}

impl Default for Component {
    fn default() -> Self {
        Component::new()
    }
}

impl Component {
    pub fn new() -> Component {
        let mut registers = [0u16; 13];
        // according to Young 2.4 these are the power on defaults
        registers[AF as usize] = 0xFFFF;
        registers[SP as usize] = 0xFFFF;
        Component {
            halted: false,
            cycles: 0,
            address: 0,
            data: 0,
            iff1: 0xFFFFFFFFFFFFFFFF,
            iff2: false,
            interrupt_mode: Im0,
            registers: registers,
            id: 0,
        }
    }

    pub fn toggle_flags(&mut self, flags: Flags) {
        let mut f = Flags::from_bits_truncate(self.get_reg8(F));
        f.toggle(flags);
        self.set_reg8(F, f.bits());
    }

    pub fn get_reg8(&self, reg8: Reg8) -> u8 {
        let byte_array: &[u8; 26] =
            unsafe {
                std::mem::transmute(&self.registers)
            };
        byte_array[reg8 as usize]
    }

    pub fn set_reg8(&mut self, reg8: Reg8, x: u8) {
        let byte_array: &mut [u8; 26] =
            unsafe {
                std::mem::transmute(&mut self.registers)
            };
        byte_array[reg8 as usize] = x
    }

    pub fn get_reg16(&self, reg16: Reg16) -> u16 {
        self.registers[reg16 as usize]
    }

    pub fn set_reg16(&mut self, reg16: Reg16, x: u16) {
        self.registers[reg16 as usize] = x;
    }

    pub fn flags(&self) -> Flags {
        Flags::from_bits_truncate(self.get_reg8(F))
    }

    pub fn set_flags(&mut self, f: Flags) {
        self.set_reg8(F, f.bits());
    }

    pub fn inc_r(&mut self) {
        let r = self.get_reg8(R);
        let ir = r.wrapping_add(1) & 0x7F;
        self.set_reg8(R, ir | (r & 0x80));
    }
}
