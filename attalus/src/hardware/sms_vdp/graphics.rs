use failure::Error;

use host_multimedia::{SimpleColor, SimpleGraphics};
use impler::{Cref, Impl, Mref, Ref};
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

pub struct SmsVdpGraphicsImpl;

impl<T: ?Sized> SmsVdpGraphics for T
where
    T: Impl<SmsVdpGraphicsImpl>,
    T::Impler: SmsVdpGraphics,
{
    #[inline]
    fn draw_line(&mut self) -> Result<(), SmsVdpGraphicsError> {
        self.make_mut().draw_line()
    }
}

pub struct SimpleSmsVdpGraphicsImpler<T: ?Sized>(Ref<T>);

impl<T: ?Sized> SimpleSmsVdpGraphicsImpler<T> {
    #[inline(always)]
    pub fn new<'a>(t: &'a T) -> Cref<'a, Self> {
        Cref::Own(SimpleSmsVdpGraphicsImpler(unsafe { Ref::new(t) }))
    }

    #[inline(always)]
    pub fn new_mut<'a>(t: &'a mut T) -> Mref<'a, Self> {
        Mref::Own(SimpleSmsVdpGraphicsImpler(unsafe { Ref::new_mut(t) }))
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

        let s = self.0.mut_0();

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

        // allow 7 extra pixels so we don't have to check writes
        // when horizontally scrolled
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

            // which line of the sprite are we rendering?
            let sprite_line = v.wrapping_sub(sprite_y) / if s.zoomed_sprites() { 2 } else { 1 };
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
            for j in 0..8 {
                let index = if s.zoomed_sprites() { 2 * j } else { j };
                let render_x = sprite_x.wrapping_add(index).wrapping_sub(shift_x);
                if render_x > 255 {
                    break;
                }
                if line_buffer[render_x] != 0x80 {
                    s.trigger_sprite_collision();
                    continue;
                }
                if palette_indices[j] != 0 {
                    line_buffer[render_x] = s.cram(palette_indices[j] as u16 + 16) as u8;
                }
                if s.zoomed_sprites() {
                    let render_x2 = render_x + 1;
                    if render_x2 > 255 {
                        break;
                    }
                    if line_buffer[render_x2] != 0x80 {
                        s.trigger_sprite_collision();
                        continue;
                    }
                    if palette_indices[j] != 0 {
                        line_buffer[render_x2] = s.cram(palette_indices[j] as u16 + 16) as u8;
                    }
                }
            }
        }

        // draw tiles
        let vert_scroll_locked = s.vert_scroll_locked();

        let scroll_x = if s.horiz_scroll_locked() && v < 16 {
            0
        } else {
            s.x_scroll()
        };
        let pixel_offset_x = scroll_x & 7;
        // let tile_offset_x = (-((scroll_x >> 3) as i16)) as u16;
        let tile_offset_x = (-((scroll_x >> 3) as i16)) as u16;

        let vert_tile_count = if SmsVdpInternal::resolution(s) == Low {
            28u16
        } else {
            32u16
        };
        let vert_tile_height = 8 * vert_tile_count;

        let scroll_y = s.y_scroll() as u16;
        let logical_y = (v + scroll_y as u16) % vert_tile_height;
        let pixel_offset_y = logical_y & 7;
        let tile_offset_y = logical_y >> 3;

        {
            let mut write_tile = |tile, tile_line, start_x| {
                let current_tile_address = s.name_table_address() + 2 * tile;
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
                for j in 0..8usize {
                    let tile_col = if horiz_flip { (7 - j) } else { j };
                    let x = j + start_x;
                    if x >= 256 {
                        break;
                    }
                    if line_buffer[x] & 0x80 != 0
                        || (priority && palette_indices[tile_col] as usize > 0)
                    {
                        line_buffer[x] = s.cram(palette_indices[tile_col] as u16 + palette) as u8;
                    }
                }
            };

            // first, draw region 3/4
            for tile in 23..if vert_scroll_locked { 32 } else { 23 } {
                write_tile(
                    32 * (v >> 3) + (tile_offset_x.wrapping_add(tile)) % 32,
                    v & 7,
                    tile as usize * 8 + pixel_offset_x as usize,
                )
            }

            // now draw region 1 or 2
            for tile in 0..if vert_scroll_locked { 24 } else { 32 } {
                write_tile(
                    32 * tile_offset_y + (tile_offset_x.wrapping_add(tile)) % 32,
                    pixel_offset_y,
                    tile as usize * 8 + pixel_offset_x as usize,
                );
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

pub struct FakeSmsVdpGraphicsImpler;

impl SmsVdpGraphics for FakeSmsVdpGraphicsImpler {
    #[inline]
    fn draw_line(&mut self) -> Result<(), SmsVdpGraphicsError> {
        Ok(())
    }
}
