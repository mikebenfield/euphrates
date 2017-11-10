// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use sdl2;

use errors::*;
use super::*;
use ::hardware::vdp;

const DEFAULT_SIZE: usize = 256;

pub struct Window {
    // Fields are dropped in the same order they are declared, so the order of
    // the first three fields here shouldn't change.
    // Also, I have been unable to figure out how the hell rust_sdl2 expects
    // users to use the `Texture` class due to lifetime issues. I just
    // circumvent the problem by transmuting to get a 'static lifetime, which
    // I'm fairly sure is the only reasonable solution.
    texture: sdl2::render::Texture<'static>,
    texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    canvas: sdl2::render::WindowCanvas,
    pixels: Box<[u8]>,
    width: usize,
    height: usize,
    texture_width: usize,
    texture_height: usize,
}

impl Window {
    pub fn new(sdl: &sdl2::Sdl) -> Result<Window> {
        let vid = sdl.video()?;
        let win = vid.window(&"", DEFAULT_SIZE as u32, DEFAULT_SIZE as u32)
            .build().chain_err(||
                ErrorKind::HostMultimedia(
                    format!("Error building window {} by {}", DEFAULT_SIZE, DEFAULT_SIZE)
                )
            )?;
        let canvas = win.into_canvas()
            .accelerated()
            .build().chain_err(||
                ErrorKind::HostMultimedia("Error creating canvas".to_owned())
            )?;
        let texture_creator = canvas.texture_creator();
        let texture = {
            let texture_tmp = texture_creator.create_texture_static(
                Some(sdl2::pixels::PixelFormatEnum::ARGB8888),
                DEFAULT_SIZE as u32,
                DEFAULT_SIZE as u32,
            ).chain_err(||
                ErrorKind::HostMultimedia(
                    format!("Error creating texture")
                )
            )?;
            unsafe {
                std::mem::transmute(texture_tmp)
            }
        };
        let pixels = vec![0; 4 * DEFAULT_SIZE * DEFAULT_SIZE].into_boxed_slice();
        Ok(
            Window {
                canvas: canvas,
                texture_creator: texture_creator,
                texture: texture,
                pixels: pixels,
                width: DEFAULT_SIZE,
                height: DEFAULT_SIZE,
                texture_width: DEFAULT_SIZE,
                texture_height: DEFAULT_SIZE,
            }
        )
    }

    pub fn set_title(&mut self, title: &str) {
        // rust_sdl2's set_title gives an error if the string has a null
        // character in it. Rather than propagate that error, let's just
        // circumvent it.
        let len = title.find('\0').unwrap_or(title.len());
        let result = self.canvas.window_mut().set_title(&title[0..len]);
        debug_assert!(result.is_ok());
    }

    pub fn title(&mut self) -> &str {
        self.canvas.window().title()
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn set_size(&mut self, width: usize, height: usize) {
        if width == self.width && height == self.height {
            return;
        }

        // rust_sdl2 gives an error, or sometimes even a segfault, if the width
        // or height are too big. How about, instead, we just clamp them.
        let max_size = 20000;
        let use_width = if width > max_size { max_size } else { width };
        let use_height = if height > max_size { max_size } else { height };
        let result = self.canvas.window_mut().set_size(use_width as u32, use_height as u32);
        debug_assert!(result.is_ok());

        self.width = use_width;
        self.height = use_height;
    }

    pub fn texture_size(&self) -> (usize, usize) {
        (self.texture_width, self.texture_height)
    }

    pub fn set_texture_size(&mut self, texture_width: usize, texture_height: usize) {
        if self.texture_size() == (texture_width, texture_height) {
            return;
        }
        let texture = {
            let texture_tmp = self.texture_creator.create_texture_static(
                Some(sdl2::pixels::PixelFormatEnum::ARGB8888),
                texture_width as u32,
                texture_height as u32,
            ).expect("Unable to create a texture");
            unsafe {
                std::mem::transmute(texture_tmp)
            }
        };

        self.texture = texture;

        let pixels = vec![0; 4 * texture_width * texture_height].into_boxed_slice();
        self.pixels = pixels;

        self.texture_width = texture_width;
        self.texture_height = texture_height;
    }
}

impl vdp::Screen for Window {
    fn paint(&mut self, x: usize, y: usize, color: u8) {
        let blue = (0x30 & color) << 2;
        let green = (0x0C & color) << 4;
        let red = (0x03 & color) << 6;
        let idx = 4 * (y * self.texture_width + x);
        self.pixels[idx] = blue;
        self.pixels[idx + 1] = green;
        self.pixels[idx + 2] = red;
        self.pixels[idx + 3] = 0;
    }

    fn render(&mut self) -> Result<()> {
        self.canvas.clear();
        self.texture.update(
            None,
            &self.pixels,
            self.texture_width * 4,
        ).chain_err(||
            ErrorKind::HostMultimedia("Error updating texture".to_owned())
        )?;
        match self.canvas.copy(&self.texture, None, None) {
            // why the hell does rust_sdl2 use String for some errors?
            Err(s) => bail!(
                ErrorKind::Screen(s)
            ),
            _ => {}
        }
        self.canvas.present();
        Ok(())
    }

    fn set_resolution(&mut self, width: usize, height: usize) -> Result<()> {
        self.set_texture_size(width, height);
        Ok(())
    }
}
