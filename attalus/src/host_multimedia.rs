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

pub trait SimpleGraphicsImpl {
    type Impler: SimpleGraphics + ?Sized;

    fn close<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Self::Impler) -> T;

    fn close_mut<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self::Impler) -> T;
}

impl<T> SimpleGraphics for T
where
    T: SimpleGraphicsImpl + ?Sized,
{
    #[inline]
    fn set_resolution(&mut self, width: u32, height: u32) -> Result<()> {
        self.close_mut(|z| z.set_resolution(width, height))
    }

    #[inline]
    fn resolution(&self) -> (u32, u32) {
        self.close(|z| z.resolution())
    }

    #[inline]
    fn paint(&mut self, x: u32, y: u32, color: SimpleColor) {
        self.close_mut(|z| z.paint(x, y, color))
    }

    fn get(&self, x: u32, y: u32) -> SimpleColor {
        self.close(|z| z.get(x, y))
    }

    #[inline]
    fn render(&mut self) -> Result<()> {
        self.close_mut(|z| z.render())
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

pub trait SimpleAudioImpl {
    type Impler: SimpleAudio;

    fn close<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Self::Impler) -> T;

    fn close_mut<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self::Impler) -> T;
}

impl<T> SimpleAudio for T
where
    T: SimpleAudioImpl,
{
    fn configure(&mut self, frequency: u32, buffer_size: u16) -> Result<()> {
        self.close_mut(|z| z.configure(frequency, buffer_size))
    }

    fn play(&mut self) -> Result<()> {
        self.close_mut(|z| z.play())
    }

    fn pause(&mut self) -> Result<()> {
        self.close_mut(|z| z.pause())
    }

    fn buffer_len(&self) -> usize {
        self.close(|z| z.buffer_len())
    }

    fn buffer_set(&mut self, i: usize, value: i16) {
        self.close_mut(|z| z.buffer_set(i, value))
    }

    fn queue_buffer(&mut self) -> Result<()> {
        self.close_mut(|z| z.queue_buffer())
    }

    fn clear(&mut self) -> Result<()> {
        self.close_mut(|z| z.clear())
    }
}

#[derive(Clone, Default, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct FakeAudio(Box<[i16]>);

impl SimpleAudio for FakeAudio {
    #[inline]
    fn configure(&mut self, _frequency: u32, buffer_size: u16) -> Result<()> {
        self.0 = vec![0i16; buffer_size as usize].into_boxed_slice();
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
        self.0.len()
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
