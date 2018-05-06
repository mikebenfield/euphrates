#[macro_use]
extern crate failure;
extern crate chrono;
extern crate sdl2;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bincode;

pub mod memo;
#[macro_use]
pub mod errors;
#[macro_use]
mod utilities;
#[macro_use]
pub mod hardware;
pub mod host_multimedia;
pub mod impler;
pub mod save;
pub mod systems;

pub use utilities::{time_govern, TimeInfo};
