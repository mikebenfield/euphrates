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
use std::os::raw::{c_char, c_int};

use sdl2::sys as sdls;

#[derive(Clone, Debug)]
pub struct Error(String);

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{:?}", self)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(x: FromUtf8Error) -> Error {
        let msg = format!("While trying to convert an SDL message: {}", x.description());
        Error(msg)
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        &self.0
    }
}

fn cstring_to_string(s: *const c_char) -> Result<String, FromUtf8Error> {
    let mut result: Vec<u8> = Vec::new();
    for i in 0.. {
        let value = unsafe {
            *s.offset(i)
        };
        if value == 0 {
            break;
        }
        result.push(value as u8);
    }
    String::from_utf8(result)
}

macro_rules! sdl_call {
    ($($args: tt)*) => {
        {
            let result = unsafe {
                $($args)*
            };
            if 0 != (result as isize) {
                let cs = unsafe {
                    sdls::sdl::SDL_GetError()
                };
                let s = cstring_to_string(cs)?;
                Err(Error(s))?;
            }
        }
    }
}

macro_rules! sdl_call_ptr {
    ($($args: tt)*) => {
        {
            let result = unsafe {
                $($args)*
            };
            if result.is_null() {
                let cs = unsafe {
                    sdls::sdl::SDL_GetError()
                };
                let s = cstring_to_string(cs)?;
                Err(Error(s))?;
            }
            result
        }
    }
}

// Put these down here so they will have access to macros
pub mod video;
pub mod event;
