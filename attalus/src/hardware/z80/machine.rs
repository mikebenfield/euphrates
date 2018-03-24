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
