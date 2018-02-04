// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

pub trait T {
    fn write(&mut self, data: u8);
}

pub trait Impler<S>
where
    S: ?Sized,
{
    fn write(&mut S, data: u8);
}

pub trait Impl {
    type Impler: Impler<Self>;
}

impl<S> T for S
where
    S: Impl
{
    #[inline]
    fn write(&mut self, data: u8) {
        <<S as Impl>::Impler as Impler<Self>>::write(self, data)
    }
}
