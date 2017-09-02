// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

//! Very simple wrapper around some SDL functions I need

use std;
use std::fmt;
use std::error::Error as StdError;
use std::string::FromUtf8Error;

use sdl2;

pub mod video;
pub mod event;

#[derive(Clone, Debug)]
pub struct Error(String);

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{:?}", self)
    }
}

macro_rules! impl_from {
    ($my_type: ident, $their_type: path) => {
        impl From<$their_type> for $my_type {
            fn from(x: $their_type) -> $my_type {
                Error(x.description().to_string())
            }
        }
    }
}

impl From<String> for Error {
    fn from(x: String) -> Error {
        Error(x)
    }
}

impl_from!{Error, sdl2::IntegerOrSdlError}
impl_from!{Error, sdl2::video::WindowBuildError}
impl_from!{Error, sdl2::render::TextureValueError}
impl_from!{Error, sdl2::render::UpdateTextureError}
impl_from!{Error, FromUtf8Error}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        &self.0
    }
}
