// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use super::higher;

pub trait T: higher::T {
    /// execute instructions until the total number of cycles run is `cycles`
    fn run(&mut self, cycles: u64);
}

pub trait Impler<S: ?Sized> {
    fn run(&mut S, cycles: u64);
}

pub trait Impl {
    type Impler: Impler<Self>;
}

impl<S> T for S
where
    S: Impl + higher::T,
{
    #[inline]
    fn run(&mut self, cycles: u64) {
        <S::Impler as Impler<Self>>::run(self, cycles);
    }
}
