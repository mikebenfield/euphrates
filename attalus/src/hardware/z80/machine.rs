// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use std::convert::{AsMut, AsRef};
use std::fmt;

use hardware::io_16_8;
use hardware::irq::Irq;
use hardware::memory_16_8;
use memo::{Inbox, Outbox};
use super::*;
use utilities;

fn receive<Z>(z: &mut Z, memo: Memo)
where
    Z: Inbox<Memo> + AsMut<Component> + AsRef<Component> + ?Sized,
{
    let id = z.as_ref().id();
    z.receive(id, memo);
}

pub trait Machine
    : io_16_8::T
    + memory_16_8::Machine
    + AsMut<Component>
    + AsRef<Component>
    + Inbox<Memo>
    + Irq
    + MachineImpl {
}

pub trait MachineImpl {}

impl<T> Machine for T
where
    T: io_16_8::T
        + memory_16_8::Machine
        + AsMut<Component>
        + AsRef<Component>
        + Inbox<Memo>
        + Irq
        + MachineImpl,
{
}

pub trait Viewable<Output>: fmt::Debug + Copy {
    fn view<Z>(self, z: &mut Z) -> Output
    where
        Z: Machine + ?Sized;
}

pub trait Changeable<Output>: Viewable<Output> {
    fn change<Z>(self, z: &mut Z, x: Output)
    where
        Z: Machine + ?Sized;
}

impl Viewable<u8> for u8 {
    fn view<Z>(self, _z: &mut Z) -> u8
    where
        Z: Machine + ?Sized,
    {
        self
    }
}

impl Viewable<u16> for u16 {
    fn view<Z>(self, _z: &mut Z) -> u16
    where
        Z: Machine + ?Sized,
    {
        self
    }
}

impl Viewable<u8> for Reg8 {
    fn view<Z>(self, z: &mut Z) -> u8
    where
        Z: Machine + ?Sized,
    {
        z.as_ref().get_reg8(self)
    }
}

impl Changeable<u8> for Reg8 {
    fn change<Z>(self, z: &mut Z, x: u8)
    where
        Z: Machine + ?Sized,
    {
        let old_value = z.as_ref().get_reg8(self);
        receive(
            z,
            Memo::Reg8Changed {
                register: self,
                old_value,
                new_value: x,
            },
        );
        z.as_mut().set_reg8(self, x);
    }
}

impl Viewable<u16> for Reg16 {
    fn view<Z>(self, z: &mut Z) -> u16
    where
        Z: Machine + ?Sized,
    {
        z.as_ref().get_reg16(self)
    }
}

impl Changeable<u16> for Reg16 {
    fn change<Z>(self, z: &mut Z, x: u16)
    where
        Z: Machine + ?Sized,
    {
        let old_value = z.as_ref().get_reg16(self);
        receive(
            z,
            Memo::Reg16Changed {
                register: self,
                old_value,
                new_value: x,
            },
        );
        z.as_mut().set_reg16(self, x);
    }
}

impl Viewable<u16> for Address<Reg16> {
    fn view<Z>(self, z: &mut Z) -> u16
    where
        Z: Machine + ?Sized,
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
        Z: Machine + ?Sized,
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
        Z: Machine + ?Sized,
    {
        let addr = self.0.view(z);
        z.read(addr)
    }
}

impl Changeable<u8> for Address<Reg16> {
    fn change<Z>(self, z: &mut Z, x: u8)
    where
        Z: Machine + ?Sized,
    {
        let addr = self.0.view(z);
        z.write(addr, x);
    }
}

impl Viewable<u16> for Address<u16> {
    fn view<Z>(self, z: &mut Z) -> u16
    where
        Z: Machine + ?Sized,
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
        Z: Machine + ?Sized,
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
        Z: Machine + ?Sized,
    {
        z.read(self.0)
    }
}

impl Changeable<u8> for Address<u16> {
    fn change<Z>(self, z: &mut Z, x: u8)
    where
        Z: Machine + ?Sized,
    {
        z.write(self.0, x);
    }
}

impl Viewable<u8> for Shift {
    fn view<Z>(self, z: &mut Z) -> u8
    where
        Z: Machine + ?Sized,
    {
        let addr = self.0.view(z).wrapping_add(self.1 as i16 as u16);
        Address(addr).view(z)
    }
}

impl Changeable<u8> for Shift {
    fn change<Z>(self, z: &mut Z, x: u8)
    where
        Z: Machine + ?Sized,
    {
        let addr = self.0.view(z).wrapping_add(self.1 as i16 as u16);
        Address(addr).change(z, x);
    }
}

impl Viewable<bool> for ConditionCode {
    fn view<Z>(self, z: &mut Z) -> bool
    where
        Z: Machine + ?Sized,
    {
        let f = z.as_ref().flags();
        match self {
            NZcc => !f.contains(Flags::ZF),
            Zcc => f.contains(Flags::ZF),
            NCcc => !f.contains(Flags::CF),
            Ccc => f.contains(Flags::CF),
            POcc => !f.contains(Flags::PF),
            PEcc => f.contains(Flags::PF),
            Pcc => !f.contains(Flags::SF),
            Mcc => f.contains(Flags::SF),
        }
    }
}

pub trait Emulator<Z>
where
    Z: Machine + ?Sized,
{
    fn run(&mut self, z: &mut Z, cycles: u64);
}
