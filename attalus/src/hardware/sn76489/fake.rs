// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use failure::Error;

use super::{hardware, machine};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct T;

impl<S> hardware::Impler<S> for T {
    #[inline]
    fn write(_s: &mut S, _data: u8) {}
}

impl<S> machine::Impler<S> for T {
    #[inline]
    fn queue(_s: &mut S, _target_cycles: u64) -> Result<(), Error> {
        Ok(())
    }
}
