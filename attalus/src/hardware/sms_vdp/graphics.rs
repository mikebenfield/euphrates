use failure::Error;

use host_multimedia::{SimpleColor, SimpleGraphics};
use impler::{ConstOrMut, Impler, ImplerImpl};
use utilities;

use super::*;

#[derive(Debug, Fail)]
pub enum SmsVdpGraphicsError {
    #[fail(display = "graphics error {}", _0)]
    Graphics(Error),
}

pub trait SmsVdpGraphics {
    fn draw_line(&mut self) -> Result<(), SmsVdpGraphicsError>;
}

pub trait SmsVdpGraphicsImpl {
    type Impler: SmsVdpGraphics + ?Sized;

    fn close<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Self::Impler) -> T;

    fn close_mut<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self::Impler) -> T;
}

impl<T: ?Sized> SmsVdpGraphics for T
where
    T: SmsVdpGraphicsImpl,
{
    #[inline]
    fn draw_line(&mut self) -> Result<(), SmsVdpGraphicsError> {
        self.close_mut(|z| z.draw_line())
    }
}

pub struct SimpleSmsVdpGraphicsImpler<T: ?Sized>(ConstOrMut<T>);

unsafe impl<T: ?Sized> ImplerImpl for SimpleSmsVdpGraphicsImpler<T> {
    type T = T;

    #[inline]
    unsafe fn new(c: ConstOrMut<Self::T>) -> Self {
        SimpleSmsVdpGraphicsImpler(c)
    }

    #[inline]
    fn get(&self) -> &ConstOrMut<Self::T> {
        &self.0
    }

    #[inline]
    fn get_mut(&mut self) -> &mut ConstOrMut<Self::T> {
        &mut self.0
    }
}

#[inline]
pub fn vdp_color_to_simple_color(color: u8) -> SimpleColor {
    let blue = (0x30 & color) << 2;
    let green = (0x0C & color) << 4;
    let red = (0x03 & color) << 6;
    SimpleColor { red, green, blue }
}

impl<T> SmsVdpGraphics for SimpleSmsVdpGraphicsImpler<T>
where
    T: SimpleGraphics + SmsVdpInternal + ?Sized,
{
    fn draw_line(&mut self) -> Result<(), SmsVdpGraphicsError> {
        use self::Resolution::*;

        let s = self.mut_0();

        let v = s.v();

        if v >= s.active_lines() {
            if v + 1 == s.total_lines() {
                s.render().map_err(|e| SmsVdpGraphicsError::Graphics(e))?;
            }
            return Ok(());
        }

        let active_lines = s.active_lines() as u32;
        s.set_resolution(256, active_lines)
            .map_err(|e| SmsVdpGraphicsError::Graphics(e))?;

        if !s.display_visible() {
            for x in 0..256 {
                s.paint(
                    x,
                    v as u32,
                    SimpleColor {
                        red: 0,
                        green: 0,
                        blue: 0,
                    },
                );
            }
            return Ok(());
        }

        let mut line_buffer = [0x80u8; 256];

        let v = s.v();

        // draw sprites
        let sprite_height = if s.tall_sprites() { 16 } else { 8 };
        let sprites_rendered = 0u8;
        for i in 0..64 {
            let sprite_y = unsafe { s.sprite_y(i) } as u16;
            if sprite_y == 0xD1 && SmsVdpInternal::resolution(s) == Low {
                break;
            }
            let sprite_line = v.wrapping_sub(sprite_y);
            if sprite_line >= sprite_height {
                continue;
            }
            if sprites_rendered == 8 {
                s.trigger_sprite_overflow();
                break;
            }

            let pattern_addr = unsafe { s.sprite_pattern_address(i) };

            let palette_indices: [u8; 8] =
                unsafe { s.pattern_address_to_palette_indices(pattern_addr, sprite_line) };
            let sprite_x = unsafe { s.sprite_x(i) } as usize;
            let shift_x = if s.shift_sprites() { 8 } else { 0 };
            for i in 0..8 {
                let render_x = sprite_x.wrapping_add(i).wrapping_sub(shift_x);
                if render_x > 255 {
                    break;
                }
                if line_buffer[render_x] != 0x80 {
                    s.trigger_sprite_collision();
                    continue;
                }
                if palette_indices[i] != 0 {
                    line_buffer[render_x] = s.cram(palette_indices[i] as u16 + 16) as u8;
                }
            }
        }

        // draw tiles
        let scrolled_v = (v + s.y_scroll() as u16) % if SmsVdpInternal::resolution(s) == Low {
            28 * 8
        } else {
            32 * 8
        };
        let tile_index_base = (scrolled_v / 8) * 32;
        let tile_line = scrolled_v % 8;
        for tile in 0..32u16 {
            let current_tile_address = s.name_table_address() + 2 * (tile + tile_index_base);
            let low_byte = s.vram(current_tile_address);
            let high_byte = s.vram(current_tile_address.wrapping_add(1));
            let vert_flip = 4 & high_byte != 0;
            let horiz_flip = 2 & high_byte != 0;
            let priority = 0x10 & high_byte != 0;
            let palette = ((high_byte & 8) << 1) as u16;
            let pattern_index = utilities::to16(low_byte, high_byte & 1);
            let tile_line_really = if vert_flip { 7 - tile_line } else { tile_line };
            let palette_indices: [u8; 8] = unsafe {
                s.pattern_address_to_palette_indices(pattern_index * 32, tile_line_really)
            };
            for j in 0..8 {
                let x = if horiz_flip {
                    tile as usize * 8 + (7 - j)
                } else {
                    tile as usize * 8 + j
                } as u8;

                let scrolled_x = x.wrapping_add(s.x_scroll()) as usize;
                if line_buffer[scrolled_x] == 0x80 || (priority && palette_indices[j] as usize > 0)
                {
                    line_buffer[scrolled_x] = s.cram(palette_indices[j] as u16 + palette) as u8;
                }
            }
        }

        if s.left_column_blank() {
            let color = s.cram(16 + s.backdrop_color_index() as u16);
            for i in 0..8 {
                line_buffer[i] = color as u8;
            }
        }

        for x in 0..256 {
            let color = vdp_color_to_simple_color(line_buffer[x as usize]);
            s.paint(x, v as u32, color);
        }

        Ok(())
    }
}

pub struct FakeSmsVdpGraphicsImpler<T: ?Sized>(ConstOrMut<T>);

unsafe impl<T: ?Sized> ImplerImpl for FakeSmsVdpGraphicsImpler<T> {
    type T = T;

    #[inline]
    unsafe fn new(c: ConstOrMut<Self::T>) -> Self {
        FakeSmsVdpGraphicsImpler(c)
    }

    #[inline]
    fn get(&self) -> &ConstOrMut<Self::T> {
        &self.0
    }

    #[inline]
    fn get_mut(&mut self) -> &mut ConstOrMut<Self::T> {
        &mut self.0
    }
}

impl<T> SmsVdpGraphics for FakeSmsVdpGraphicsImpler<T> {
    #[inline]
    fn draw_line(&mut self) -> Result<(), SmsVdpGraphicsError> {
        Ok(())
    }
}
