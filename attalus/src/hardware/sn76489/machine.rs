use std;

use failure::Error;

use super::*;

type Result<T> = std::result::Result<T, Error>;

pub trait Sn76489: Sn76489Hardware {
    fn queue(&mut self, target_cycles: u64) -> Result<()>;
}

pub trait Sn76489Impler<S>
where
    S: ?Sized,
{
    fn queue(&mut S, target_cycles: u64) -> Result<()>;
}

pub trait Sn76489Impl {
    type Impler: Sn76489Impler<Self>;
}


impl<S> Sn76489 for S
where
    S: Sn76489Impl + Sn76489Hardware + ?Sized
{
    #[inline]
    fn queue(&mut self, target_cycles: u64) -> Result<()> {
        <<S as Sn76489Impl>::Impler as Sn76489Impler<Self>>::queue(self, target_cycles)
    }
}
