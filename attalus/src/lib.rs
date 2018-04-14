#![recursion_limit="1024"]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate bitflags;
extern crate sdl2;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bincode;

pub mod memo;
#[macro_use]
mod macros;
pub mod errors;
mod runtime_pattern;
#[macro_use]
mod utilities;
pub mod sdl_wrap;
#[macro_use]
pub mod hardware;
pub mod systems;
pub mod host_multimedia;
pub mod save;

pub use utilities::{FrameInfo, TimeInfo, time_govern};
