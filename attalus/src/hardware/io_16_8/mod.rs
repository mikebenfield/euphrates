// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

pub mod sms2;

/// A machine that has an IO system with 16 bit addresses and 8 bit data.
pub trait Machine {
    fn input(&mut self, address: u16) -> u8;
    fn output(&mut self, address: u16, value: u8);
}

/// A component providing IO services to `T`.
pub trait ComponentOf<T>
where
    T: ?Sized,
{
    fn input(t: &mut T, address: u16) -> u8;
    fn output(t: &mut T, address: u16, value: u8);
}

pub trait MachineImpl {
    type C: ComponentOf<Self>;
}

impl<T> Machine for T
where
    T: MachineImpl,
{
    #[inline]
    fn input(&mut self, address: u16) -> u8 {
        <T::C as ComponentOf<Self>>::input(self, address)
    }

    #[inline]
    fn output(&mut self, address: u16, value: u8) {
        <T::C as ComponentOf<Self>>::output(self, address, value)
    }
}


#[derive(Clone, Copy, Default, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SimpleIo;

impl SimpleIo {
    pub fn new() -> SimpleIo {
        SimpleIo
    }
}

impl<T> ComponentOf<T> for SimpleIo
where
    T: ?Sized,
{
    fn input(_t: &mut T, _address: u16) -> u8 {
        0
    }

    fn output(_t: &mut T, _address: u16, _value: u8) {}
}
