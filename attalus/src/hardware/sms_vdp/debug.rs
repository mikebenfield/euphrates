use utilities;

use host_multimedia::SimpleGraphics;

use super::*;

pub fn draw_tiles<V, G>(v: &V, graphics: &mut G) -> Result<(), SmsVdpGraphicsError>
where
    V: ?Sized + SmsVdpInternal,
    G: ?Sized + SimpleGraphics,
{
    match (v.m1(), v.m2(), v.m3(), v.m4()) {
        (_, _, _, true) => draw_tiles_mode4(v, graphics),
        (false, false, false, _) => draw_tiles_graphics1(v, graphics),
        (false, true, false, _) => draw_tiles_graphics2(v, graphics),
        _ => Ok(()),
    }
}

fn draw_tiles_graphics1<V, G>(v: &V, graphics: &mut G) -> Result<(), SmsVdpGraphicsError>
where
    V: ?Sized + SmsVdpInternal,
    G: ?Sized + SimpleGraphics,
{
    let pattern_table = ((v.register(4) & 0x7) as u16) << 11;
    let name_table = ((v.register(2) & 0xF) as u16) << 10;
    let color_table = (v.register(3) as u16) << 6;

    graphics
        .set_resolution(256, 192)
        .map_err(|e| SmsVdpGraphicsError::Graphics(e))?;

    for tile_x in 0..32 {
        for tile_y in 0..24 {
            let name = v.vram(name_table + tile_y * 32 + tile_x) as u16;
            let color_entry = name / 8;
            let color = v.vram(color_table + color_entry as u16);
            let color0 = TMS9918_PALETTE[color as usize & 0xF];
            let color1 = TMS9918_PALETTE[color as usize >> 4];
            for tile_line in 0..8 {
                let mut pattern = v.vram(pattern_table + name + tile_line);
                for i in 0..8 {
                    graphics.paint(
                        tile_x as u32 * 8 + i,
                        tile_y as u32 * 8 + tile_line as u32,
                        if pattern * 0x80 == 0 { color0 } else { color1 },
                    );
                }
            }
        }
    }

    graphics
        .render()
        .map_err(|e| SmsVdpGraphicsError::Graphics(e))
}

fn draw_tiles_graphics2<V, G>(v: &V, graphics: &mut G) -> Result<(), SmsVdpGraphicsError>
where
    V: ?Sized + SmsVdpInternal,
    G: ?Sized + SimpleGraphics,
{
    let pattern_table = ((v.register(4) & 4) as u16) << 11;
    let name_table = ((v.register(2) & 0xF) as u16) << 10;
    let color_table = ((v.register(3) & 0x80) as u16) << 6;

    graphics
        .set_resolution(256, 192)
        .map_err(|e| SmsVdpGraphicsError::Graphics(e))?;

    for tile_y in 0..24 {
        let third = tile_y / 8;
        for tile_x in 0..32 {
            let name = v.vram(name_table + tile_y * 32 + tile_x) as u16;
            let pattern_address = pattern_table + 2048 * third;
            let color_address = color_table + 2048 * third;
            for tile_line in 0..8 {
                let color = v.vram(color_address + name * 8 + tile_line);
                let color0 = TMS9918_PALETTE[color as usize & 0xF];
                let color1 = TMS9918_PALETTE[color as usize >> 4];
                let mut pattern = v.vram(pattern_address + name * 8 + tile_line);
                for i in 0..8 {
                    graphics.paint(
                        tile_x as u32 * 8 + i,
                        tile_y as u32 * 8 + tile_line as u32,
                        if pattern & 0x80 == 0 { color0 } else { color1 },
                    );
                    pattern <<= 1;
                }
            }
        }
    }

    graphics
        .render()
        .map_err(|e| SmsVdpGraphicsError::Graphics(e))
}

fn draw_tiles_mode4<V, G>(v: &V, graphics: &mut G) -> Result<(), SmsVdpGraphicsError>
where
    V: ?Sized + SmsVdpInternal,
    G: ?Sized + SimpleGraphics,
{
    let vert_tile_count = if v.resolution() == Resolution::Low {
        28u16
    } else {
        32u16
    };
    let height = 8 * vert_tile_count;
    graphics
        .set_resolution(256, height as u32)
        .map_err(|e| SmsVdpGraphicsError::Graphics(e))?;
    for tile_x in 0..32 {
        for tile_y in 0..vert_tile_count {
            let current_tile_address = v.name_table_address() + 2 * (32 * tile_y + tile_x);
            let low_byte = v.vram(current_tile_address);
            let high_byte = v.vram(current_tile_address.wrapping_add(1));
            let pixel_x = (8 * tile_x) as u32;
            let pixel_y = (8 * tile_y) as u32;
            let palette = ((high_byte & 8) << 1) as u16;
            let pattern_index = utilities::to16(low_byte, high_byte & 1);
            for line in 0..8 {
                let palette_indices: [u8; 8] =
                    unsafe { v.pattern_address_to_palette_indices(pattern_index * 32, line) };
                for j in 0..8usize {
                    let color = vdp_color_to_simple_color(
                        v.cram(palette_indices[j] as u16 + palette) as u8,
                    );
                    graphics.paint(pixel_x + j as u32, pixel_y + line as u32, color);
                }
            }
        }
    }

    let y_scroll = v.y_scroll();
    for i in 0..256 {
        use host_multimedia::SimpleColor;
        graphics.paint(
            i,
            y_scroll as u32 % height as u32,
            SimpleColor {
                red: 0xF0,
                green: 0,
                blue: 0xF0,
            },
        );
    }
    if v.vert_scroll_locked() {
        for i in 0..height as u32 {
            use host_multimedia::SimpleColor;
            graphics.paint(
                24 * 8,
                i,
                SimpleColor {
                    red: 0xF0,
                    green: 0xF0,
                    blue: 0,
                },
            );
        }
    }

    let x_scroll = -(v.x_scroll() as i8) as u8;
    for i in 0..height as u32 {
        use host_multimedia::SimpleColor;
        graphics.paint(
            x_scroll as u32,
            i,
            SimpleColor {
                red: 0xF0,
                green: 0,
                blue: 0xF0,
            },
        );
    }
    if v.horiz_scroll_locked() {
        for i in 0..256 {
            use host_multimedia::SimpleColor;
            graphics.paint(
                i,
                16,
                SimpleColor {
                    red: 0xF0,
                    green: 0xF0,
                    blue: 0,
                },
            );
        }
    }

    graphics
        .render()
        .map_err(|e| SmsVdpGraphicsError::Graphics(e))
}
