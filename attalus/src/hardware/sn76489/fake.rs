use failure::Error;

use super::*;

/// A `Sn76489Impler` that doesn't actually do anything.
///
/// If you don't need sound and want to save a bit of time and memory, use this.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct FakeSn76489;

impl<S> Sn76489InternalImpler<S> for FakeSn76489 {
    #[inline]
    fn write(_s: &mut S, _data: u8) {}
}

impl<S> Sn76489Impler<S> for FakeSn76489 {
    #[inline]
    fn queue(_s: &mut S, _target_cycles: u64) -> Result<(), Error> {
        Ok(())
    }
}
