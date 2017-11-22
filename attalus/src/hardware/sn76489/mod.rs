// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

pub mod fake;
pub mod real;

use ::errors::*;

pub trait Machine {
    /// The default implementation does nothing, so you can easily implement
    /// `Machine` if you don't want sound.
    fn write_sound(&mut self, data: u8);
}

pub trait ComponentOf<T>
where
    T: ?Sized
{
    fn write_sound(&mut T, data: u8);
}

pub trait MachineImpl {
    type C: ComponentOf<Self>;
}

impl<T> Machine for T
where
    T: MachineImpl
{
    #[inline(always)]
    fn write_sound(&mut self, data: u8) {
        <<T as MachineImpl>::C as ComponentOf<Self>>::write_sound(self, data)
    }
}

pub trait Emulator<HostAudio, Component>
where
    HostAudio: ?Sized,
    Component: ?Sized,
{
    fn queue(
        &mut self,
        component: &mut Component,
        target_cycles: u64,
        audio: &mut HostAudio,
    ) -> Result<()>;
}
