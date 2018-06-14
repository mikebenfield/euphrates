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

#[inline]
pub fn gg_color_to_simple_color(color: u16) -> SimpleColor {
    let blue = (0x0F00 & color) >> 4;
    let green = 0x00F0 & color;
    let red = (0x000F & color) << 4;
    SimpleColor {
        red: red as u8,
        green: green as u8,
        blue: blue as u8,
    }
}

impl<T> SmsVdpGraphics for SimpleSmsVdpGraphicsImpler<T>
where
    T: SimpleGraphics + SmsVdpInternal + ?Sized,
{
    fn draw_line(&mut self) -> Result<(), SmsVdpGraphicsError> {
        let s = self.0.mut_0();
        match (s.m1(), s.m2(), s.m3(), s.m4()) {
            (_, _, _, true) => draw_line_mode4(s),
            (false, false, false, _) => draw_line_graphics1(s),
            (false, true, false, _) => draw_line_graphics2(s),
            _ => unimplemented!(),
        }
    }
}

/// The actual palette of the TMS9918.
///
/// see http://www.smspower.org/Development/Palette
pub static TMS9918_PALETTE: [SimpleColor; 16] = [
    // Transparent
    SimpleColor {
        red: 0x00,
        green: 0x00,
        blue: 0x00,
    },
    // Black
    SimpleColor {
        red: 0x00,
        green: 0x00,
        blue: 0x00,
    },
    // Medium Green
    SimpleColor {
        red: 0x21,
        green: 0xC8,
        blue: 0x42,
    },
    // Light Green
    SimpleColor {
        red: 0x5E,
        green: 0xDC,
        blue: 0x78,
    },
    // Dark Blue
    SimpleColor {
        red: 0x54,
        green: 0x55,
        blue: 0xED,
    },
    // Light Blue
    SimpleColor {
        red: 0x7D,
        green: 0x76,
        blue: 0xFC,
    },
    // Dark Red
    SimpleColor {
        red: 0xD4,
        green: 0x52,
        blue: 0x4D,
    },
    // Cyan
    SimpleColor {
        red: 0x42,
        green: 0xEB,
        blue: 0xF5,
    },
    // Medium Red
    SimpleColor {
        red: 0xFC,
        green: 0x55,
        blue: 0x54,
    },
    // Light Red
    SimpleColor {
        red: 0xFF,
        green: 0x79,
        blue: 0x78,
    },
    // Dark Yellow
    SimpleColor {
        red: 0xD4,
        green: 0xC1,
        blue: 0x54,
    },
    // Light Yellow
    SimpleColor {
        red: 0xE6,
        green: 0xCE,
        blue: 0x80,
    },
    // Dark Green
    SimpleColor {
        red: 0x21,
        green: 0xB0,
        blue: 0x3B,
    },
    // Magenta
    SimpleColor {
        red: 0xC9,
        green: 0x5B,
        blue: 0xBA,
    },
    // Gray
    SimpleColor {
        red: 0xCC,
        green: 0xCC,
        blue: 0xCC,
    },
    // White
    SimpleColor {
        red: 0xFF,
        green: 0xFF,
        blue: 0xFF,
    },
];

/// The SMS used these approximations of the original TMS9918 palette.
pub static TMS9918_PALETTE_SMS: [SimpleColor; 16] = [
    // Transparent
    SimpleColor {
        red: 0x00,
        green: 0x00,
        blue: 0x00,
    },
    // Black
    SimpleColor {
        red: 0x00,
        green: 0x00,
        blue: 0x00,
    },
    // Medium Green
    SimpleColor {
        red: 0x00,
        green: 0xAA,
        blue: 0x00,
    },
    // Light Green
    SimpleColor {
        red: 0x00,
        green: 0xFF,
        blue: 0x00,
    },
    // Dark Blue
    SimpleColor {
        red: 0x00,
        green: 0x00,
        blue: 0x55,
    },
    // Light Blue
    SimpleColor {
        red: 0x00,
        green: 0x00,
        blue: 0xFF,
    },
    // Dark Red
    SimpleColor {
        red: 0x55,
        green: 0x00,
        blue: 0x00,
    },
    // Cyan
    SimpleColor {
        red: 0x00,
        green: 0xFF,
        blue: 0xFF,
    },
    // Medium Red
    SimpleColor {
        red: 0xAA,
        green: 0x00,
        blue: 0x00,
    },
    // Light Red
    SimpleColor {
        red: 0xFF,
        green: 0x00,
        blue: 0x00,
    },
    // Dark Yellow
    SimpleColor {
        red: 0x55,
        green: 0x55,
        blue: 0x00,
    },
    // Light Yellow
    SimpleColor {
        red: 0xFF,
        green: 0xFF,
        blue: 0x00,
    },
    // Dark Green
    SimpleColor {
        red: 0x00,
        green: 0x55,
        blue: 0x00,
    },
    // Magenta
    SimpleColor {
        red: 0xFF,
        green: 0x00,
        blue: 0xFF,
    },
    // Gray
    SimpleColor {
        red: 0x55,
        green: 0x55,
        blue: 0x55,
    },
    // White
    SimpleColor {
        red: 0xFF,
        green: 0xFF,
        blue: 0xF,
    },
];

pub fn draw_sprites_tms<T>(s: &mut T) -> Result<(), SmsVdpGraphicsError>
where
    T: SimpleGraphics + SmsVdpInternal + ?Sized,
{
    let sprites_large = s.register(1) & 2 != 0;
    let sprites_zoom = s.register(1) & 1 != 0;
    let sprite_size = match (sprites_large, sprites_zoom) {
        (true, true) => 32,
        (false, false) => 8,
        _ => 16,
    };

    let v = s.v();

    let sprite_pattern_table = ((s.register(6) & 0x7) as u16) << 11;
    let sprite_attribute_table = ((s.register(5) & 0x7F) as u16) << 7;

    let mut sprites_on_line = 0;

    let mut line = [false; 256];

    for i in 0..32 {
        let y = s.vram(sprite_attribute_table + 4 * i).wrapping_add(1) as u16;
        if y == 0xD1 {
            break;
        }
        let x = s.vram(sprite_attribute_table + 4 * i + 1) as u16;
        let sprite_line = v.wrapping_sub(y);
        if sprite_line >= sprite_size {
            continue;
        }

        sprites_on_line += 1;
        if sprites_on_line > 4 {
            let mut status = s.status_flags() & 0xE0;
            status |= i as u8;
            status |= SPRITE_OVERFLOW_FLAG;
            s.set_status_flags(status);
            return Ok(());
        }

        let sprite_y = if sprites_zoom {
            sprite_line / 2
        } else {
            sprite_line
        };
        let name = s.vram(sprite_attribute_table + 4 * i + 2) as u16
            & if sprites_large { 0xFC } else { 0xFF };

        let last_byte = s.vram(sprite_attribute_table + 4 * i + 3);
        let early_clock = last_byte & 0x80 != 0;
        let color = last_byte & 0xF;
        let color1 = TMS9918_PALETTE[color as usize];

        let line_pattern_index = sprite_pattern_table + name * 8 + sprite_y;

        let pattern = s.vram(line_pattern_index);
        let pattern2 = if sprites_large {
            Some(s.vram(line_pattern_index + 16))
        } else {
            None
        };

        let mut render_pattern = |mut pattern: u8, mut screen_x: u16| {
            let mut draw = |x| {
                if line[x as usize] {
                    s.trigger_sprite_collision();
                    return;
                }
                line[x as usize] = true;
                s.paint(x as u32, v as u32, color1);
            };
            if sprites_zoom {
                for _ in 0..8 {
                    // FIX
                    if screen_x >= 256 {
                        continue;
                    }
                    if pattern & 0x80 != 0 {
                        draw(screen_x);
                        draw(screen_x + 1);
                    }
                    pattern <<= 1;
                    screen_x = screen_x.wrapping_add(2);
                }
            } else {
                for _ in 0..8 {
                    if screen_x >= 256 {
                        continue;
                    }
                    if pattern & 0x80 != 0 {
                        draw(screen_x);
                    }
                    pattern <<= 1;
                    screen_x = screen_x.wrapping_add(1);
                }
            }
        };

        let screen_x = x.wrapping_sub(if early_clock { 32 } else { 0 });
        render_pattern(pattern, screen_x);
        if let Some(pattern2) = pattern2 {
            render_pattern(pattern2, screen_x.wrapping_add(8));
        }
    }

    Ok(())
}

pub fn draw_line_graphics1<T>(s: &mut T) -> Result<(), SmsVdpGraphicsError>
where
    T: SimpleGraphics + SmsVdpInternal + ?Sized,
{
    let pattern_table = ((s.register(4) & 0x7) as u16) << 11;
    let name_table = ((s.register(2) & 0xF) as u16) << 10;
    let color_table = (s.register(3) as u16) << 6;

    let v = s.v();

    if v >= 192 {
        if v + 1 == s.total_lines() {
            s.render().map_err(|e| SmsVdpGraphicsError::Graphics(e))?;
        }
        return Ok(());
    }

    s.set_resolution(256, 192)
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

    let tile_y = v / 8;
    let tile_line = v % 8;
    for tile_x in 0..32 {
        let name = s.vram(name_table + tile_y * 32 + tile_x) as u16;
        let color_entry = name / 8;
        let color = s.vram(color_table + color_entry as u16);
        let color0 = TMS9918_PALETTE[color as usize & 0xF];
        let color1 = TMS9918_PALETTE[color as usize >> 4];
        let mut pattern = s.vram(pattern_table + name + tile_line);
        for i in 0..8 {
            s.paint(
                tile_x as u32 * 8 + i,
                v as u32,
                if pattern & 0x80 == 0 { color0 } else { color1 },
            );
            pattern <<= 1;
        }
    }

    draw_sprites_tms(s)
}

pub fn draw_line_graphics2<T>(s: &mut T) -> Result<(), SmsVdpGraphicsError>
where
    T: SimpleGraphics + SmsVdpInternal + ?Sized,
{
    let pattern_table = ((s.register(4) & 4) as u16) << 11;
    let name_table = ((s.register(2) & 0xF) as u16) << 10;
    let color_table = ((s.register(3) & 0x80) as u16) << 6;

    let v = s.v();

    if v >= 192 {
        if v + 1 == s.total_lines() {
            s.render().map_err(|e| SmsVdpGraphicsError::Graphics(e))?;
        }
        return Ok(());
    }

    s.set_resolution(256, 192)
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

    let tile_y = v / 8;
    let tile_line = v % 8;

    let third = v / 64;
    let pattern_address = pattern_table + 2048 * third;
    let color_address = color_table + 2048 * third;

    for tile_x in 0..32 {
        let name = s.vram(name_table + tile_y * 32 + tile_x) as u16;
        let color = s.vram(color_address + name * 8 + tile_line);
        let color0 = TMS9918_PALETTE[color as usize & 0xF];
        let color1 = TMS9918_PALETTE[color as usize >> 4];
        let mut pattern = s.vram(pattern_address + name * 8 + tile_line);
        for i in 0..8 {
            s.paint(
                tile_x as u32 * 8 + i,
                v as u32,
                if pattern & 0x80 == 0 { color0 } else { color1 },
            );
            pattern <<= 1;
        }
    }

    draw_sprites_tms(s)
}

pub fn draw_line_mode4<T>(s: &mut T) -> Result<(), SmsVdpGraphicsError>
where
    T: SimpleGraphics + SmsVdpInternal + ?Sized,
{
    use self::Resolution::*;

    let v = s.v();

    let (display_y_start, display_y_end, display_x_start, display_x_end) = if s.kind() == Kind::Gg {
        (24, 168, 48, 208)
    } else {
        (0, s.active_lines(), 0, 256)
    };
    let height = display_y_end - display_y_start;
    let width = display_x_end - display_x_start;

    if v < display_y_start {
        return Ok(());
    }

    if v >= display_y_end {
        if v + 1 == s.total_lines() {
            s.render().map_err(|e| SmsVdpGraphicsError::Graphics(e))?;
        }
        return Ok(());
    }

    s.set_resolution(width as u32, height as u32)
        .map_err(|e| SmsVdpGraphicsError::Graphics(e))?;

    let y = (v - display_y_start) as u32;

    if !s.display_visible() {
        for x in 0..width {
            s.paint(
                x as u32,
                y,
                SimpleColor {
                    red: 0,
                    green: 0,
                    blue: 0,
                },
            );
        }
        return Ok(());
    }

    let mut colors: [SimpleColor; 32] = Default::default();

    if s.kind() == Kind::Gg {
        for i in 0..32 {
            colors[i] = gg_color_to_simple_color(s.cram(i as u16));
        }
    } else {
        for i in 0..32 {
            colors[i] = vdp_color_to_simple_color(s.cram(i as u16) as u8);
        }
    }

    let mut line_buffer = [0x80u8; 256];

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
            if render_x < display_x_start || render_x >= display_x_end {
                break;
            }
            if line_buffer[render_x] != 0x80 {
                s.trigger_sprite_collision();
                continue;
            }
            if palette_indices[j] != 0 {
                line_buffer[render_x] = palette_indices[j] + 16;
            }
            if s.zoomed_sprites() {
                let render_x2 = render_x + 1;
                if render_x2 < display_x_start || render_x2 >= display_x_end {
                    break;
                }
                if line_buffer[render_x2] != 0x80 {
                    s.trigger_sprite_collision();
                    continue;
                }
                if palette_indices[j] != 0 {
                    line_buffer[render_x] = palette_indices[j] + 16;
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
            let palette = (high_byte & 8) << 1;
            let pattern_index = utilities::to16(low_byte, high_byte & 1);
            let tile_line_really = if vert_flip { 7 - tile_line } else { tile_line };
            let palette_indices: [u8; 8] = unsafe {
                s.pattern_address_to_palette_indices(pattern_index * 32, tile_line_really)
            };
            for j in 0..8usize {
                let tile_col = if horiz_flip { (7 - j) } else { j };
                let x = j + start_x;
                if x < display_x_start {
                    continue;
                }
                if x >= display_x_end {
                    break;
                }
                if line_buffer[x] & 0x80 != 0
                    || (priority && palette_indices[tile_col] as usize > 0)
                {
                    line_buffer[x] = palette_indices[tile_col] + palette;
                }
            }
        };

        // first, draw region 3/4
        if s.kind() != Kind::Gg && vert_scroll_locked {
            for tile in 23..32 {
                write_tile(
                    32 * (v >> 3) + (tile_offset_x.wrapping_add(tile)) % 32,
                    v & 7,
                    tile as usize * 8 + pixel_offset_x as usize,
                )
            }
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
        for i in 0..8 {
            line_buffer[i] = 16 + s.backdrop_color_index();
        }
    }

    for x in display_x_start..display_x_end {
        let index = line_buffer[x as usize] as usize;
        let color = colors[index % 32];
        s.paint((x - display_x_start) as u32, y, color);
    }

    Ok(())
}

pub struct FakeSmsVdpGraphicsImpler;

impl SmsVdpGraphics for FakeSmsVdpGraphicsImpler {
    #[inline]
    fn draw_line(&mut self) -> Result<(), SmsVdpGraphicsError> {
        Ok(())
    }
}
