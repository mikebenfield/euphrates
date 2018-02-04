
use std;

use failure::Error;

use super::hardware;

type Result<T> = std::result::Result<T, Error>;

pub trait T: hardware::T {
    fn queue(&mut self, target_cycles: u64) -> Result<()>;
}

pub trait Impler<S>
where
    S: ?Sized,
{
    fn queue(&mut S, target_cycles: u64) -> Result<()>;
}

pub trait Impl {
    type Impler: Impler<Self>;
}


impl<S> T for S
where
    S: Impl + hardware::T + ?Sized
{
    #[inline]
    fn queue(&mut self, target_cycles: u64) -> Result<()> {
        <<S as Impl>::Impler as Impler<Self>>::queue(self, target_cycles)
    }
}
