
use std::os::raw::{c_int, c_void};

use super::*;

use hardware::vdp;

struct Window(*mut sdls::video::SDL_Window);

impl Window {
    fn new() -> Result<Window, Error> {
        let v: Vec<i8> = vec![0; 1];
        Ok(
            Window(
                sdl_call_ptr!(
                    // XXX - use SDL_WINDOW_ALLOW_HIGHDPI?
                    sdls::video::SDL_CreateWindow(
                        v.as_ptr(),
                        5,
                        5,
                        5,
                        5,
                        0,
                    )
                )
            )
        )
    }
}

impl std::ops::Drop for Window {
    fn drop(&mut self) {
        unsafe {
            sdls::video::SDL_DestroyWindow(self.0)
        }
    }
}

struct Renderer(*mut sdls::render::SDL_Renderer);

impl Renderer {
    fn new(win: &Window) -> Result<Renderer, Error> {
        Ok(
            Renderer(
                sdl_call_ptr!(
                    sdls::render::SDL_CreateRenderer(
                        win.0,
                        0,
                        sdls::render::SDL_RENDERER_TARGETTEXTURE |
                        sdls::render::SDL_RENDERER_ACCELERATED
                    )
                )
            )
        )
    }
}

impl std::ops::Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            sdls::render::SDL_DestroyRenderer(self.0)
        }
    }
}

struct Texture(*mut sdls::render::SDL_Texture);

impl Texture {
    fn new(renderer: &Renderer, width: usize, height: usize) -> Result<Texture, Error> {
        Ok(
            Texture(
                sdl_call_ptr!(
                    sdls::render::SDL_CreateTexture(
                        renderer.0,
                        sdls::pixels::SDL_PIXELFORMAT_RGB332,
                        sdls::render::SDL_TEXTUREACCESS_STREAMING as c_int,
                        width as c_int,
                        height as c_int,
                    )
                )
            )
        )
    }
}

impl std::ops::Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            sdls::render::SDL_DestroyTexture(self.0)
        }
    }
}

pub struct WindowCanvas {
    window: Window,
    renderer: Renderer,
    texture: Texture,
    pixels: Box<[u8]>,
    logical_width: usize,
}

impl WindowCanvas {
    pub fn new() -> Result<WindowCanvas, Error> {
        sdl_call!(
            sdls::sdl::SDL_Init(sdls::sdl::SDL_INIT_VIDEO)
        );

        let mut window = Window::new()?;
        let mut renderer = Renderer::new(&window)?;
        sdl_call!(
            sdls::render::SDL_RenderSetLogicalSize(renderer.0, 1, 1)
        );
        let mut texture = Texture::new(&renderer, 1, 1)?;

        Ok(
            WindowCanvas {
                window: window,
                renderer: renderer,
                texture: texture,
                pixels: vec![0; 1].into_boxed_slice(),
                logical_width: 1,
            }
        )
    }

    pub fn set_window_size(&mut self, w: usize, h: usize) {
        unsafe {
            sdls::video::SDL_SetWindowSize(self.window.0, w as c_int, h as c_int);
        }
    }

    pub fn set_logical_size(&mut self, w: usize, h: usize) -> Result<(), Error> {
        self.pixels = vec![0; w*h].into_boxed_slice();
        self.logical_width = w;
        sdl_call!(
            sdls::render::SDL_RenderSetLogicalSize(self.renderer.0, w as c_int, h as c_int)
        );
        self.texture = Texture::new(&self.renderer, w, h)?;
        Ok(())
    }

    pub fn get_logical_size(&mut self) -> (usize, usize) {
        (self.logical_width, self.pixels.len() / self.logical_width)
    }

    pub fn set_title(&mut self, s: &str) {
        let mut v = s.as_bytes().to_vec();
        v.push(0);
        unsafe {
            sdls::video::SDL_SetWindowTitle(self.window.0, v.as_ptr() as *const c_char);
        }
    }
}

impl vdp::Canvas for WindowCanvas {
    fn paint(&mut self, x: usize, y: usize, color: u8) {
        let blue = (0x30 & color) >> 4;
        let green = (0x0C) << 1;
        let red = (0x03 & color) << 6;
        self.pixels[y*self.logical_width + x] = blue | green | red;
    }

    fn render(&mut self) -> Result<(), vdp::CanvasError> {
        sdl_call!(
            sdls::render::SDL_UpdateTexture(
                self.texture.0,
                std::ptr::null(),
                std::mem::transmute(self.pixels.as_ptr()), // XXX
                self.logical_width as c_int,
            )
        );
        sdl_call!(
            sdls::render::SDL_RenderClear(self.renderer.0)
        );
        sdl_call!(
            sdls::render::SDL_RenderCopy(
                self.renderer.0,
                self.texture.0,
                std::ptr::null(),
                std::ptr::null(),
            )
        );
        unsafe {
            sdls::render::SDL_RenderPresent(self.renderer.0);
        }
        Ok(())
    }
}
