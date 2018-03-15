// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use std::fmt;

use hardware::io_16_8;
use hardware::memory_16_8;
use super::higher;
use super::{Reg16, Reg8, ConditionCode};
use utilities;

pub trait T: higher::T + io_16_8::T + memory_16_8::T {
    fn requesting_mi(&self) -> Option<u8>;
    fn requesting_nmi(&self) -> bool;

    /// The Z80 responds to nonmaskable interrupts due to the change in voltage
    /// in the NMI pin from high to low, so it will not continually execute
    /// interrupts when the voltage is held low. In software, that means we need
    /// to tell the device the interrupt is being executed and to stop
    /// requesting it.
    fn clear_nmi(&mut self);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Shift(pub Reg16, pub i8);

impl fmt::Display for Shift {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = format!("({}{:<+#0X})", self.0, self.1);
        f.pad(&s)
    }
}

/// An aspect of the Z80 that we can view, like a register or a memory address.
///
/// This trait (and `Changeable`) exists so that we may implement an instruction
/// like `ld x, y` with a single generic function, although `x` and `y` may be
/// memory addresses or registers.
pub trait Viewable<Output>: Copy {
    fn view<Z>(self, z: &mut Z) -> Output
    where
        Z: T + ?Sized;
}

/// An aspect of the Z80 that we can change, like a register or a memory address.
///
/// This trait (and `Viewable`) exists so that we may implement an instruction
/// like `ld x, y` with a single generic function, although `x` and `y` may be
/// memory addresses or registers.
pub trait Changeable<Output>: Viewable<Output> {
    fn change<Z>(self, z: &mut Z, x: Output)
    where
        Z: T + ?Sized;
}

impl Viewable<u8> for u8 {
    fn view<Z>(self, _z: &mut Z) -> u8
    where
        Z: T + ?Sized,
    {
        self
    }
}

impl Viewable<u16> for u16 {
    fn view<Z>(self, _z: &mut Z) -> u16
    where
        Z: T + ?Sized,
    {
        self
    }
}

impl Viewable<u8> for Reg8 {
    fn view<Z>(self, z: &mut Z) -> u8
    where
        Z: T + ?Sized,
    {
        z.reg8(self)
    }
}

impl Changeable<u8> for Reg8 {
    fn change<Z>(self, z: &mut Z, x: u8)
    where
        Z: T + ?Sized,
    {
        z.set_reg8(self, x);
    }
}

impl Viewable<u16> for Reg16 {
    fn view<Z>(self, z: &mut Z) -> u16
    where
        Z: T + ?Sized,
    {
        z.reg16(self)
    }
}

impl Changeable<u16> for Reg16 {
    fn change<Z>(self, z: &mut Z, x: u16)
    where
        Z: T + ?Sized,
    {
        z.set_reg16(self, x);
    }
}

impl Viewable<u16> for Address<Reg16> {
    fn view<Z>(self, z: &mut Z) -> u16
    where
        Z: T + ?Sized,
    {
        let addr = self.0.view(z);
        let lo = z.read(addr);
        let hi = z.read(addr.wrapping_add(1));
        utilities::to16(lo, hi)
    }
}

impl Changeable<u16> for Address<Reg16> {
    fn change<Z>(self, z: &mut Z, x: u16)
    where
        Z: T + ?Sized,
    {
        let addr = self.0.view(z);
        let (lo, hi) = utilities::to8(x);
        z.write(addr, lo);
        z.write(addr.wrapping_add(1), hi);
    }
}

impl Viewable<u8> for Address<Reg16> {
    fn view<Z>(self, z: &mut Z) -> u8
    where
        Z: T + ?Sized,
    {
        let addr = self.0.view(z);
        z.read(addr)
    }
}

impl Changeable<u8> for Address<Reg16> {
    fn change<Z>(self, z: &mut Z, x: u8)
    where
        Z: T + ?Sized,
    {
        let addr = self.0.view(z);
        z.write(addr, x);
    }
}

impl Viewable<u16> for Address<u16> {
    fn view<Z>(self, z: &mut Z) -> u16
    where
        Z: T + ?Sized,
    {
        let addr = self.0;
        let lo = z.read(addr);
        let hi = z.read(addr.wrapping_add(1));
        utilities::to16(lo, hi)
    }
}

impl Changeable<u16> for Address<u16> {
    fn change<Z>(self, z: &mut Z, x: u16)
    where
        Z: T + ?Sized,
    {
        let addr = self.0;
        let (lo, hi) = utilities::to8(x);
        z.write(addr, lo);
        z.write(addr.wrapping_add(1), hi);
    }
}

impl Viewable<u8> for Address<u16> {
    fn view<Z>(self, z: &mut Z) -> u8
    where
        Z: T + ?Sized,
    {
        z.read(self.0)
    }
}

impl Changeable<u8> for Address<u16> {
    fn change<Z>(self, z: &mut Z, x: u8)
    where
        Z: T + ?Sized,
    {
        z.write(self.0, x);
    }
}

impl Viewable<u8> for Shift {
    fn view<Z>(self, z: &mut Z) -> u8
    where
        Z: T + ?Sized,
    {
        let addr = self.0.view(z).wrapping_add(self.1 as i16 as u16);
        Address(addr).view(z)
    }
}

impl Changeable<u8> for Shift {
    fn change<Z>(self, z: &mut Z, x: u8)
    where
        Z: T + ?Sized,
    {
        let addr = self.0.view(z).wrapping_add(self.1 as i16 as u16);
        Address(addr).change(z, x);
    }
}

impl Viewable<bool> for ConditionCode {
    fn view<Z>(self, z: &mut Z) -> bool
    where
        Z: T + ?Sized,
    {
        let f = z.reg8(Reg8::F);
        match self {
            ConditionCode::NZcc => f & higher::ZF == 0,
            ConditionCode::Zcc => f & higher::ZF != 0,
            ConditionCode::NCcc => f & higher::CF == 0,
            ConditionCode::Ccc => f & higher::CF != 0,
            ConditionCode::POcc => f & higher::PF == 0,
            ConditionCode::PEcc => f & higher::PF != 0,
            ConditionCode::Pcc => f & higher::SF == 0,
            ConditionCode::Mcc => f & higher::SF != 0,
        }
    }
}
