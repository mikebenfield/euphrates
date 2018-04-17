use super::*;

pub trait Z80: Z80Internal {
    /// execute instructions until the total number of cycles run is `cycles`
    fn run(&mut self, cycles: u64);
}

pub trait Z80Impler<S: ?Sized> {
    fn run(&mut S, cycles: u64);
}

pub trait Z80Impl {
    type Impler: Z80Impler<Self>;
}

impl<S> Z80 for S
where
    S: Z80Impl + Z80Internal,
{
    #[inline]
    fn run(&mut self, cycles: u64) {
        S::Impler::run(self, cycles);
    }
}
