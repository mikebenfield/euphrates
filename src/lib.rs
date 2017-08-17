// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify this file
// under the terms of the GNU General Public License, version 3, as published by
// the Free Software Foundation. You should have received a copy of the GNU
// General Public License along with Attalus. If not, see
// <http://www.gnu.org/licenses/>.

extern crate rand;
extern crate tempdir;
extern crate sdl2;

#[macro_use]
pub mod log;
mod bits;
// pub mod sdl_wrap;
#[macro_use]
pub mod hardware;
pub mod emulation_manager;
