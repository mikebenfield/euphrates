// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use std;

use failure;

use ::errors;

pub use ::errors::CommonKind as Kind;

pub type Error = errors::Error<Kind>;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SimpleColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

pub trait SimpleGraphics {
    fn set_resolution(&mut self, width: u32, height: u32) -> Result<()>;

    fn resolution(&self) -> (u32, u32);

    /// Will panic if x and y are outside the bounds determined by the
    /// resolution, but is memory safe. Any errors in the implementation that occur
    /// will be returned in the next call to `render`.
    fn paint(&mut self, x: u32, y: u32, color: SimpleColor);

    /// Will panic if x and y are outside the bounds determined by the
    /// resolution, but is memory safe. Any errors in the implementation that occur
    /// will be returned in the next call to `render`.
    fn get(&self, x: u32, y: u32) -> SimpleColor;

    /// Display the pixels that have been `paint`ed. Any pixel position that has
    /// not been `paint`ed since the last call to `render` may show arbitrary
    /// results.
    fn render(&mut self) -> Result<()>;
}

pub trait SimpleAudio {
    fn configure(&mut self, frequency: u32, buffer_size: u16) -> std::result::Result<(), failure::Error>;

    fn play(&mut self) -> std::result::Result<(), failure::Error>;

    fn pause(&mut self) -> std::result::Result<(), failure::Error>;

    fn buffer(&mut self) -> &mut [i16];

    fn queue_buffer(&mut self) -> std::result::Result<(), failure::Error>;

    fn clear(&mut self) -> std::result::Result<(), failure::Error>;
}

#[derive(Clone, Default, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct FakeAudio(Box<[i16]>);


impl SimpleAudio for FakeAudio {
    #[inline]
    fn configure(&mut self, _frequency: u32, buffer_size: u16) -> std::result::Result<(), failure::Error> {
        self.0 = vec![0i16; buffer_size as usize].into_boxed_slice();
        Ok(())
    }

    #[inline]
    fn play(&mut self) -> std::result::Result<(), failure::Error> {
        Ok(())
    }

    #[inline]
    fn pause(&mut self) -> std::result::Result<(), failure::Error> {
        Ok(())
    }

    #[inline]
    fn buffer(&mut self) -> &mut [i16] {
        &mut self.0
    }

    #[inline]
    fn queue_buffer(&mut self) -> std::result::Result<(), failure::Error> {
        Ok(())
    }

    #[inline]
    fn clear(&mut self) -> std::result::Result<(), failure::Error> {
        Ok(())
    }
}
