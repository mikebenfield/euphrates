use std;

use failure::Error;

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

pub struct SimpleGraphicsImpl;

#[derive(Clone, Default, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct FakeGraphics(u32, u32);

impl SimpleGraphics for FakeGraphics {
    #[inline]
    fn set_resolution(&mut self, width: u32, height: u32) -> Result<()> {
        self.0 = width;
        self.1 = height;
        Ok(())
    }

    #[inline]
    fn resolution(&self) -> (u32, u32) {
        (self.0, self.1)
    }

    #[inline]
    fn paint(&mut self, _x: u32, _y: u32, _color: SimpleColor) {}

    #[inline]
    fn get(&self, _x: u32, _y: u32) -> SimpleColor {
        SimpleColor {
            red: 0,
            green: 0,
            blue: 0,
        }
    }

    #[inline]
    fn render(&mut self) -> Result<()> {
        Ok(())
    }
}

pub trait SimpleAudio {
    fn configure(&mut self, frequency: u32, buffer_size: u16) -> Result<()>;

    fn play(&mut self) -> Result<()>;

    fn pause(&mut self) -> Result<()>;

    fn buffer_len(&self) -> usize;

    fn buffer_set(&mut self, i: usize, value: i16);

    fn queue_buffer(&mut self) -> Result<()>;

    fn clear(&mut self) -> Result<()>;
}

#[derive(Clone, Default, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct FakeAudio;

impl SimpleAudio for FakeAudio {
    #[inline]
    fn configure(&mut self, _frequency: u32, _buffer_size: u16) -> Result<()> {
        Ok(())
    }

    #[inline]
    fn play(&mut self) -> Result<()> {
        Ok(())
    }

    #[inline]
    fn pause(&mut self) -> Result<()> {
        Ok(())
    }

    #[inline]
    fn buffer_set(&mut self, _i: usize, _value: i16) {}

    #[inline]
    fn buffer_len(&self) -> usize {
        1024 // lie
    }

    #[inline]
    fn queue_buffer(&mut self) -> Result<()> {
        Ok(())
    }

    #[inline]
    fn clear(&mut self) -> Result<()> {
        Ok(())
    }
}
