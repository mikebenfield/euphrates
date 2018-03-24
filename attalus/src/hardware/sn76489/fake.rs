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
