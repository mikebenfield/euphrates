// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use errors::{CommonKind, Error};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Component;

impl<T> super::ComponentOf<T> for Component
where
    T: ?Sized,
{
    #[inline(always)]
    fn write_sound(_t: &mut T, _data: u8) {}
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Emulator;

impl<HostAudio, Comp> super::Emulator<HostAudio, Comp> for Emulator {
    #[inline(always)]
    fn queue(
        &mut self,
        _component: &mut Comp,
        _target_cycles: u64,
        _audio: &mut HostAudio,
    ) -> Result<(), Error<CommonKind>> {
        Ok(())
    }
}
