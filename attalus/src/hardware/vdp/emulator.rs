// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use std;
use ::errors::{Error, CommonKind};
use ::utilities;
use super::Component;
use ::host_multimedia::{SimpleColor, SimpleGraphics};

type Result<T> = std::result::Result<T, Error<CommonKind>>;

pub trait Emulator<HostGraphics> {
    fn draw_line(&mut self, vdp: &mut Component, graphics: &mut HostGraphics) -> Result<()>;

    fn can_draw_sprites(&self) -> bool;

    fn draw_sprites(&mut self, vdp: &mut Component, graphics: &mut HostGraphics) -> Result<()>;

    fn can_draw_tiles(&self) -> bool;

    fn draw_tiles(&mut self, vdp: &mut Component, graphics: &mut HostGraphics) -> Result<()>;
}

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, Hash)]
pub struct SimpleEmulator;

#[inline]
pub fn vdp_color_to_simple_color(color: u8) -> SimpleColor {
    let blue = (0x30 & color) << 2;
    let green = (0x0C & color) << 4;
    let red = (0x03 & color) << 6;
    SimpleColor { red, green, blue }
}

impl<S> Emulator<S> for SimpleEmulator
where
    S: SimpleGraphics
{
    fn draw_line(&mut self, vdp: &mut Component, graphics: &mut S) -> Result<()> {
        use super::Resolution::*;

        fn finish<S: SimpleGraphics>(vdp: &mut Component, graphics: &mut S) -> Result<()> {
            vdp.drew_line();
            if vdp.v == 0 {
                graphics.render()?;
            }
            return Ok(());
        }

        if vdp.v >= vdp.active_lines() {
            return finish(vdp, graphics);
        }

        graphics.set_resolution(
            256,
            vdp.active_lines() as u32,
        )?;

        if !vdp.display_visible() {
            for x in 0 .. 256 {
                graphics.paint(x, vdp.v as u32, SimpleColor {red: 0, green: 0, blue: 0});
            }
            return finish(vdp, graphics);
        }

        let mut line_buffer = [0x80u8; 256];

        let v = vdp.v as usize;

        // draw sprites
        let sprite_height = if vdp.tall_sprites() { 16 } else { 8 };
        let sprites_rendered = 0u8;
        for i in 0..64 {
            let sprite_y = vdp.sprite_y(i) as usize;
            if sprite_y == 0xD1 && vdp.resolution() == Low {
                break;
            }
            let sprite_line = v.wrapping_sub(sprite_y);
            if sprite_line >= sprite_height {
                continue;
            }
            if sprites_rendered == 8 {
                vdp.trigger_sprite_overflow();
                break;
            }
            let sprite_n = vdp.sprite_n(i);
            let pattern_addr = vdp.sprite_pattern_address(sprite_n) as usize;

            let palette_indices: [usize; 8] = vdp.pattern_address_to_palette_indices(
                pattern_addr,
                sprite_line
            );
            let sprite_x = vdp.sprite_x(i) as usize;
            let shift_x = if vdp.shift_sprites() { 8 } else { 0 };
            for i in 0 .. 8 {
                let render_x = sprite_x.wrapping_add(i).wrapping_sub(shift_x);
                if render_x > 255 {
                    break;
                }
                if line_buffer[render_x] != 0x80 {
                    vdp.trigger_sprite_collision();
                    continue;
                }
                if palette_indices[i] != 0 {
                    line_buffer[render_x] = vdp.cram[palette_indices[i] + 16];
                }
            }
        }

        // draw tiles
        let scrolled_v = (v + vdp.y_scroll() as usize) % if vdp.resolution() == Low { 28*8 } else { 32*8 };
        let tile_index_base = (scrolled_v / 8) * 32;
        let tile_line = scrolled_v % 8;
        for tile in 0..32u16 {
            let current_tile_address =
                vdp.tile_address(2 * (tile + tile_index_base as u16)) as usize;
            let low_byte = vdp.vram[current_tile_address as u16 as usize];
            let high_byte = vdp.vram[current_tile_address.wrapping_add(1) as u16 as usize];
            let vert_flip = 4 & high_byte != 0;
            let horiz_flip = 2 & high_byte != 0;
            let priority = 0x10 & high_byte != 0;
            let palette = ((high_byte & 8) << 1) as usize;
            let pattern_index = utilities::to16(low_byte, high_byte & 1) as usize;
            let tile_line_really = if vert_flip {
                7 - tile_line
            } else {
                tile_line
            };
            let palette_indices: [usize; 8] = vdp.pattern_address_to_palette_indices(
                pattern_index * 32,
                tile_line_really
            );
            for j in 0 .. 8 {
                let x = if horiz_flip {
                    tile as usize * 8 + (7 - j)
                } else {
                    tile as usize * 8 + j
                } as u8;

                let scrolled_x = x.wrapping_add(vdp.x_scroll()) as usize;
                if line_buffer[scrolled_x] == 0x80 || (priority && palette_indices[j] > 0) {
                    line_buffer[scrolled_x] = vdp.cram[palette_indices[j] + palette];
                }
            }
        }

        if vdp.left_column_blank() {
            let color = vdp.cram[16 + vdp.backdrop_color() as usize];
            for i in 0..8 {
                line_buffer[i] = color;
            }
        }

        for x in 0..256 {
            let color = vdp_color_to_simple_color(line_buffer[x as usize]);
            graphics.paint(x, v as u32, color);
        }

        return finish(vdp, graphics);
    }

    fn can_draw_sprites(&self) -> bool {
        false
    }

    fn draw_sprites(&mut self, _vdp: &mut Component, _graphics: &mut S) -> Result<()> {
        unimplemented!();
    }

    fn can_draw_tiles(&self) -> bool {
        false
    }

    fn draw_tiles(&mut self, _vdp: &mut Component, _graphics: &mut S) -> Result<()> {
        unimplemented!();
    }
}
