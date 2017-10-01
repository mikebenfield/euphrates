// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

#[macro_use]
extern crate bitflags;
extern crate rand;
extern crate tempdir;
extern crate sdl2;
extern crate serde;
#[macro_use]
extern crate serde_derive;

pub mod message;
mod bits;
pub mod sdl_wrap;
#[macro_use]
pub mod hardware;
pub mod emulation_manager;
