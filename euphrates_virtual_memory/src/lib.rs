#![deny(bare_trait_objects, anonymous_parameters)]

extern crate euphrates;
extern crate libc;
extern crate rand;

mod external;
mod posix;
mod traits;

use posix::*;

pub use external::*;
