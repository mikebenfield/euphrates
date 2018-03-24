//! Very simple wrapper around some SDL functions I need

use std;
use std::fmt;
use std::error::Error as StdError;
use std::string::FromUtf8Error;

use sdl2;

pub mod master_system_user_interface;
pub mod simple_graphics;
pub mod simple_audio;

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
