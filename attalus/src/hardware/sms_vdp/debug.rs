use utilities;

use host_multimedia::SimpleGraphics;

use super::*;

pub fn draw_tiles<V, G>(v: &V, graphics: &mut G) -> Result<(), SmsVdpGraphicsError>
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
                    let color = vdp_color_to_simple_color(v
                        .cram(palette_indices[j] as u16 + palette)
                        as u8);
                    graphics.paint(pixel_x + j as u32, pixel_y + line as u32, color);
                }
            }
        }
    }
    graphics.render().map_err(|e| SmsVdpGraphicsError::Graphics(e))
}
