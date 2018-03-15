// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

#[macro_use]
pub mod instruction_list;
pub mod instructions;
pub mod internal;
pub mod higher;
pub mod part;
pub mod state;
pub mod simple;
pub mod machine;
pub mod interpreter;
pub mod memo;

pub use self::memo::Opcode;

use std::fmt;

/// The Z80's `iff1` flag determines whether maskable interrupts are accepted.
///
/// The instruction immediately after the `ei` (`enable interrupts`) instruction
/// is still immune from interrupts, so there are 3 states.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Iff1State {
    On,
    Off,
    Ei,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum InterruptMode {
    Im0,
    Im1,
    Im2,
}

impl Default for InterruptMode {
    fn default() -> Self {
        InterruptMode::Im0
    }
}

#[cfg(target_endian = "little")]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Matchable)]
pub enum Reg8 {
    C,
    B,
    E,
    D,
    F,
    A,
    L,
    H,
    C0,
    B0,
    E0,
    D0,
    F0,
    A0,
    L0,
    H0,
    IXL,
    IXH,
    IYL,
    IYH,
    SPL,
    SPH,
    PCL,
    PCH,
    I,
    R,
}

#[cfg(target_endian = "big")]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Matchable)]
pub enum Reg8 {
    B,
    C,
    D,
    E,
    A,
    F,
    H,
    L,
    B0,
    C0,
    D0,
    E0,
    A0,
    F0,
    H0,
    L0,
    IXH,
    IXL,
    IYH,
    IYL,
    SPH,
    SPL,
    PCH,
    PCL,
    I,
    R,
}

impl fmt::Display for Reg8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match *self {
            Reg8::B => "b",
            Reg8::C => "c",
            Reg8::D => "d",
            Reg8::E => "e",
            Reg8::A => "a",
            Reg8::F => "f",
            Reg8::H => "h",
            Reg8::L => "l",
            Reg8::B0 => "b'",
            Reg8::C0 => "c'",
            Reg8::D0 => "d'",
            Reg8::E0 => "e'",
            Reg8::A0 => "a'",
            Reg8::F0 => "f'",
            Reg8::H0 => "h'",
            Reg8::L0 => "l'",
            Reg8::IXL => "ixl",
            Reg8::IXH => "ixh",
            Reg8::IYL => "iyl",
            Reg8::IYH => "iyh",
            Reg8::SPL => "spl",
            Reg8::SPH => "sph",
            Reg8::PCL => "pcl",
            Reg8::PCH => "pch",
            Reg8::I => "i",
            Reg8::R => "r",
        };
        f.pad(s)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Matchable)]
pub enum Reg16 {
    BC,
    DE,
    AF,
    HL,
    BC0,
    DE0,
    AF0,
    HL0,
    IX,
    IY,
    SP,
    PC,
}

impl fmt::Display for Reg16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match *self {
            Reg16::BC => "bc",
            Reg16::DE => "de",
            Reg16::AF => "af",
            Reg16::HL => "hl",
            Reg16::BC0 => "bc'",
            Reg16::DE0 => "de'",
            Reg16::AF0 => "af'",
            Reg16::HL0 => "hl'",
            Reg16::IX => "ix",
            Reg16::IY => "iy",
            Reg16::SP => "sp",
            Reg16::PC => "pc",
        };
        f.pad(s)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Matchable)]
#[repr(u8)]
pub enum ConditionCode {
    /// Zero flag not set
    NZcc,

    /// Zero flag set
    Zcc,

    /// Carry flag not set
    NCcc,

    /// Carry flag set
    Ccc,

    /// Parity odd (parity flag not set)
    POcc,

    /// Parity even (parity flag set)
    PEcc,

    /// Positive (sign flag not set)
    Pcc,

    /// Negative (sign flag set)
    Mcc,
}

impl fmt::Display for ConditionCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match *self {
            ConditionCode::NZcc => "nz",
            ConditionCode::Zcc => "z",
            ConditionCode::NCcc => "nc",
            ConditionCode::Ccc => "c",
            ConditionCode::POcc => "po",
            ConditionCode::PEcc => "pe",
            ConditionCode::Pcc => "p",
            ConditionCode::Mcc => "m",
        };
        f.pad(&s)
    }
}
