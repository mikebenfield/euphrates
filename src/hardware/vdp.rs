// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use std;
use std::error::Error;

use ::bits::*;
use super::irq::Irq;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ScreenError(pub String);

impl<T: Error> From<T> for ScreenError {
    fn from(t: T) -> ScreenError {
        ScreenError(t.description().to_string())
    }
}

pub trait Screen {
    fn paint(&mut self, x: usize, y: usize, color: u8);
    fn render(&mut self) -> Result<(), ScreenError>;
    fn set_resolution(&mut self, width: usize, height: usize) -> Result<(), ScreenError>;
}

pub struct NoScreen;

impl Screen for NoScreen {
    fn paint(&mut self, _: usize, _: usize, _: u8) {}
    fn render(&mut self) -> Result<(), ScreenError> { Ok(()) }
    fn set_resolution(&mut self, _: usize, _: usize) -> Result<(), ScreenError> { Ok(()) }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum System {
    NTSC, PAL,
}

impl Default for System {
    fn default() -> System { NTSC }
}

use self::System::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Version {
    SMS, SMS2, GG, MD,
}

use self::Version::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Resolution {
    Low = 192, Medium = 224, High = 240,
}

use self::Resolution::*;

impl Default for Version {
    fn default() -> Version { SMS }
}

bitflags! {
    struct StatusFlags: u8 {
        const FRAME_INTERRUPT = 0b10000000;
        const SPRITE_OVERFLOW = 0b01000000;
        const SPRITE_COLLISION = 0b00100000;
        const LINE_INTERRUPT = 0b00010000;
        const CONTROL_FLAG = 0b00001000;
    }
}

#[derive(Copy)]
pub struct Vdp {
    pub cycles: u64,
    pub version: Version,
    pub system: System,
    status_flags: StatusFlags,
    h: u16,
    v: u16,
    address0: u16,
    buffer: u8,
    reg: [u8; 11],
    cram: [u8; 32],
    vram: [u8; 0x4000],
    line_counter: u8,
}

impl std::fmt::Debug for Vdp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "Vdp \
            {{ \n\
                status_flags: {:?}, \n\
                h: {:?}, \n\
                v: {:?}, buffer: {:?}, address0: {:?}, \n\
                reg: {:?}, \n\
                cram: {:?}, \n\
                vram: {:?} (...) \n
            }}",
            self.status_flags,
            self.h,
            self.v,
            self.buffer,
            self.address0,
            self.reg,
            self.cram,
            &self.vram[0..32]
        )
    }
}

impl Default for Vdp {
    fn default() -> Vdp {
        Vdp {
            cycles: 0,
            version: Default::default(),
            system: Default::default(),
            status_flags: StatusFlags::empty(),
            h: 0,
            v: 0,
            address0: 0,
            buffer: 0,
            reg: [0; 11],
            cram: [0; 32],
            vram: [0; 0x4000],
            line_counter: 0,
        }
    }
}

impl Clone for Vdp {
    fn clone(&self) -> Vdp {
        *self
    }
}

impl Vdp {
    fn code(&self) -> u8 {
        ((self.address0 & 0xC000) >> 14) as u8
    }
    fn address(&self) -> u16 {
        self.address0 & 0x3FFF
    }
    #[allow(dead_code)]
    fn disable_vert_scroll(&self) -> bool {
        self.reg[0] & (1 << 7) != 0
    }
    #[allow(dead_code)]
    fn disable_horiz_scroll(&self) -> bool {
        self.reg[0] & (1 << 6) != 0
    }
    #[allow(dead_code)]
    fn left_column_blank(&self) -> bool {
        self.reg[0] & (1 << 5) != 0
    }
    fn line_irq_enable(&self) -> bool {
        self.reg[0] & (1 << 4) != 0
    }
    #[allow(dead_code)]
    fn shift_sprites(&self) -> bool {
        self.reg[0] & (1 << 3) != 0
    }
    #[allow(dead_code)]
    fn m4(&self) -> bool {
        self.reg[0] & (1 << 2) != 0
    }
    fn m2(&self) -> bool {
        self.reg[0] & (1 << 1) != 0
    }
    #[allow(dead_code)]
    fn nosync(&self) -> bool {
        self.reg[0] & 1 != 0
    }
    fn total_lines(&self) -> u16 {
        if self.system == NTSC { 262 } else { 313 }
    }
    fn resolution(&self) -> Resolution {
        match (self.version, self.m1(), self.m2(), self.m3()) {
            (SMS, _, _, _) => Low,
            (_, true, true, false) => Medium,
            (_, false, true, true) => High,
            (_, _, _, _) => Low,
        }
    }
    fn active_lines(&self) -> u16 {
        match self.resolution() {
            Low => 192,
            Medium => 224,
            High => 240,
        }
    }
    #[allow(dead_code)]
    fn display_visible(&self) -> bool {
        self.reg[1] & (1 << 6) != 0
    }
    fn frame_irq_enable(&self) -> bool {
        self.reg[1] & (1 << 5) != 0
    }
    #[allow(dead_code)]
    fn m1(&self) -> bool {
        self.reg[0] & (1 << 4) != 0
    }
    #[allow(dead_code)]
    fn m3(&self) -> bool {
        self.reg[0] & (1 << 3) != 0
    }
    #[allow(dead_code)]
    fn tall_sprites(&self) -> bool {
        self.reg[1] & 2 != 0
    }
    #[allow(dead_code)]
    fn zoom_sprites(&self) -> bool {
        self.reg[0] & 1 != 0
    }
    #[allow(dead_code)]
    fn name_table_address(&self) -> u16 {
        if self.version == SMS {
            ((self.reg[2] as u16) & 0x0F) << 10
        } else if self.resolution() == Low {
            ((self.reg[2] as u16) & 0x0E) << 10
        } else {
            (((self.reg[2] as u16) & 0x0C) << 10) | 0x0700
        }
    }
    fn tile_address(&self, tile_offset: u16) -> u16 {
        if self.version == SMS {
            (self.name_table_address() | 0x03FF) & (tile_offset | 0xF800)
        } else {
            self.name_table_address() + tile_offset
        }
    }
    fn sprite_address(&self) -> u16 {
        if self.version == SMS {
            (self.reg[5] as u16 & 0x7F) << 7
        } else {
            (self.reg[5] as u16 & 0x7E) << 7
        }
    }
    fn sprite_pattern_base_address(&self) -> u16 {
        // MacDonald's VDP documentation says the SMS Vdp does something
        // strange, but that doesn't appear to be true. At least, the games
        // I've tested so far clear reg 6, which, if I implement MacDonald's
        // scheme, causes the sprite patterns to be fetched from the wrong
        // portion of vram
        (self.reg[6] as u16 & 0x04) << 11
    }
    fn sprite_pattern_address(&self, pattern_index: u8) -> u16 {
        self.sprite_pattern_base_address() | (pattern_index as u16 * 32)
    }
    #[allow(dead_code)]
    fn backdrop_color(&self) -> u8 {
        self.reg[7] & 0x0F
    }
    #[allow(dead_code)]
    fn x_scroll(&self) -> u8 {
        self.reg[8]
    }
    #[allow(dead_code)]
    fn y_scroll(&self) -> u8 {
        self.reg[9]
    }
    fn reg_line_counter(&self) -> u8 {
        self.reg[10]
    }

    fn sprite_y(&self, i: u8) -> u8 {
        debug_assert!(i <= 63);
        let address = (i as usize) | ((self.sprite_address() & 0xFF00) as usize);
        self.vram[address]
    }
    fn sprite_x(&self, i: u8) -> u8 {
        debug_assert!(i <= 63);
        let address = if self.version == SMS {
            (2*i) as usize | (self.sprite_address() as usize)
        } else {
            ((2*i + 128) as usize) | (self.sprite_address() as usize)
        };
        self.vram[address]
    }
    fn sprite_n(&self, i: u8) -> u8 {
        debug_assert!(i <= 63);
        let address = if self.version == SMS {
            (2*i) as usize | (self.sprite_address() as usize)
        } else {
            ((2*i + 128) as usize) | (self.sprite_address() as usize)
        } + 1;
        self.vram[address]
    }

    fn inc_address(&mut self) {
        let addr = self.address0;
        self.address0 = (addr.wrapping_add(1) & 0x3FFF) | (addr & 0xC000);
    }
    pub fn read_v(&self) -> u8 {
        let result =
            match (self.system, self.resolution(), self.v) {
                (NTSC, Low, 0...0xDA) => self.v,
                (NTSC, Low, _) => self.v-6,
                (NTSC, Medium, 0...0xEA) => self.v,
                (NTSC, Medium, _) => self.v-6,
                (NTSC, High, 0...0xFF) => self.v,
                (NTSC, High, _) => self.v-0x100,
                (PAL, Low, 0...0xF2) => self.v,
                (PAL, Low, _) => self.v-57,
                (PAL, Medium, 0...0xFF) => self.v,
                (PAL, Medium, 0x100...0x102) => self.v-0x100,
                (PAL, Medium, _) => self.v-57,
                (PAL, High, 0...0xFF) => self.v,
                (PAL, High, 0x100...0x10A) => self.v-0x100,
                (PAL, High, _) => self.v-57,
            };
        result as u8
    }

    pub fn read_h(&self) -> u8 {
        (self.h >> 1) as u8
    }

    pub fn read_data(&mut self) -> u8 {
        let current_buffer = self.buffer;
        self.buffer = self.cram[(self.address() % 32) as usize];
        self.inc_address();
        self.status_flags.remove(CONTROL_FLAG);
        current_buffer
    }

    pub fn read_control(&mut self) -> u8 {
        let current_status = self.status_flags.bits;
        self.status_flags.bits = 0;
        current_status
    }

    pub fn write_data(&mut self, x: u8) {
        match (self.code(), self.version) {
            // XXX - no Game Gear yet
            (3, _) => {
                self.cram[(self.address() % 32) as usize] = x;
            },
            _      => {
                self.vram[self.address() as usize] = x;
            }
        }
        self.inc_address();
        self.status_flags.remove(CONTROL_FLAG);
    }

    pub fn write_control(&mut self, x: u8) {
        if self.status_flags.contains(CONTROL_FLAG) {
            self.address0 = self.address0 & 0x00FF | (x as u16) << 8;
            if self.code() == 0 {
                self.buffer = self.vram[self.address() as usize];
                self.address0 = (self.address0.wrapping_add(1)) & 0x3FFF | (self.code() as u16) << 13;
            } else if self.code() == 2 && (x & 0x0F) <= 10 {
                self.reg[(x & 0x0F) as usize] = self.address0 as u8;
            }
            self.status_flags.remove(CONTROL_FLAG);
        } else {
            self.address0 = (self.address0 & 0xFF00) | x as u16;
            self.status_flags.insert(CONTROL_FLAG);
        }
    }

    pub fn draw_line<S: Screen>(&mut self, screen: &mut S) -> Result<(), ScreenError> {

        fn done<S: Screen>(vdp: &mut Vdp, screen: &mut S) -> Result<(), ScreenError> {
            match (vdp.resolution(), vdp.v) {
                (Low, 0xC1) => vdp.status_flags.insert(FRAME_INTERRUPT),
                (Medium, 0xE1) => vdp.status_flags.insert(FRAME_INTERRUPT),
                (High, 0xF1) => vdp.status_flags.insert(FRAME_INTERRUPT),
                _ => {}
            }
            if vdp.v <= vdp.active_lines() {
                vdp.line_counter = vdp.line_counter.wrapping_sub(1);
                if vdp.line_counter == 0xFF {
                    vdp.line_counter = vdp.reg_line_counter();
                    vdp.status_flags.insert(LINE_INTERRUPT);
                }
            } else {
                vdp.line_counter = vdp.reg_line_counter();
            }
            vdp.v = (vdp.v + 1) % vdp.total_lines();
            if vdp.v == 0 {
                screen.render()?;
            }
            vdp.cycles += 342;
            Ok(())
        }

        if self.v >= self.active_lines() {
            return done(self, screen);
        }

        screen.set_resolution(
            256,
            self.active_lines() as usize
        )?;

        if !self.display_visible() {
            for x in 0 .. 256 {
                screen.paint(x, self.v as usize, 0);
            }
            return done(self, screen);
        }

        let mut line_buffer = [0x80u8; 256];

        let v = self.v as usize;

        // draw sprites
        let sprite_height = if self.tall_sprites() { 16 } else { 8 };
        let sprites_rendered = 0u8;
        for i in 0..64 {
            let sprite_y = self.sprite_y(i) as usize;
            if sprite_y == 0xD0 && self.resolution() == Low {
                break;
            }
            let sprite_line = v.wrapping_sub(sprite_y);
            if sprite_line >= sprite_height {
                continue;
            }
            if sprites_rendered == 8 {
                self.status_flags.insert(SPRITE_OVERFLOW);
                break;
            }
            let sprite_n = self.sprite_n(i);
            let pattern_addr = self.sprite_pattern_address(sprite_n) as usize;

            let palette_indices: [usize; 8] = self.pattern_address_to_palette_indices(
                pattern_addr,
                sprite_line
            );
            let sprite_x = self.sprite_x(i) as usize;
            let shift_x = if self.shift_sprites() { 8 } else { 0 };
            for i in 0 .. 8 {
                let render_x = sprite_x.wrapping_add(i).wrapping_sub(shift_x);
                if render_x > 255 {
                    break;
                }
                if line_buffer[render_x] != 0x80 {
                    self.status_flags.insert(SPRITE_COLLISION);
                    continue;
                }
                if palette_indices[i] != 0 {
                    line_buffer[render_x] = self.cram[palette_indices[i] + 16];
                }
            }
        }

        // draw tiles
        let scrolled_v = (v + self.y_scroll() as usize) % if self.resolution() == Low { 28*8 } else { 32*8 };
        let tile_index_base = (scrolled_v / 8) * 32;
        let tile_line = scrolled_v % 8;
        for tile in 0..32u16 {
            let current_tile_address =
                self.tile_address(2 * (tile + tile_index_base as u16)) as usize;
            let low_byte = self.vram[current_tile_address as u16 as usize];
            let high_byte = self.vram[current_tile_address.wrapping_add(1) as u16 as usize];
            let vert_flip = 4 & high_byte != 0;
            let horiz_flip = 2 & high_byte != 0;
            let priority = 0x10 & high_byte != 0;
            let palette = 16 * ((high_byte & 8) >> 3) as usize;
            let pattern_index = to16(low_byte, high_byte & 1) as usize;
            let tile_line_really = if vert_flip {
                7 - tile_line
            } else {
                tile_line
            };
            let palette_indices: [usize; 8] = self.pattern_address_to_palette_indices(
                pattern_index * 32,
                tile_line_really
            );
            for j in 0 .. 8 {
                let x = if horiz_flip {
                    tile as usize * 8 + (7 - j)
                } else {
                    tile as usize * 8 + j
                } as u8;

                let scrolled_x = x.wrapping_add(self.x_scroll()) as usize;
                if line_buffer[scrolled_x] == 0x80 || (priority && palette_indices[j] > 0) {
                    line_buffer[scrolled_x] = self.cram[palette_indices[j] + palette];
                }
            }
        }

        if self.left_column_blank() {
            let color = self.cram[16 + self.backdrop_color() as usize];
            for i in 0..8 {
                line_buffer[i] = color;
            }
        }

        for x in 0..256 {
            screen.paint(x, v, line_buffer[x]);
        }

        done(self, screen)
    }

    pub fn draw_palettes<S: Screen>(&self, screen: &mut S) -> Result<(), ScreenError> {
        if self.v != 0 {
            return Ok(());
        }
        screen.set_resolution(33, 1)?;
        for i in 0..16 {
            screen.paint(i, 0, self.cram[i]);
        }
        screen.paint(16, 0, 0xFF);
        for i in 17..32 {
            screen.paint(i, 0, self.cram[i - 1]);
        }
        screen.render()?;
        Ok(())
    }

    pub fn draw_sprites<S: Screen>(&self, screen: &mut S) -> Result<(), ScreenError> {
        if self.v != 0 {
            return Ok(());
        }
        let sprite_height = if self.tall_sprites() { 16 } else { 8 };
        screen.set_resolution(64, 8 * sprite_height)?;
        for i in 0 .. 64 {
            let sprite_n = self.sprite_n(i);
            let pattern_addr = self.sprite_pattern_address(sprite_n) as usize;
            for sprite_line in 0 .. sprite_height {
                let palette_indices: [usize; 8] = self.pattern_address_to_palette_indices(
                    pattern_addr,
                    sprite_line
                );
                for j in 0 .. 8 {
                    let x = (i as usize % 8) * 8 + j;
                    let y = (i as usize / 8) * sprite_height + sprite_line;
                    if palette_indices[j] != 0 {
                        screen.paint(x as usize, y as usize, self.cram[palette_indices[j] + 16]);
                    }
                }
            }
        }
        screen.render()?;
        Ok(())
    }

    pub fn draw_tiles<S: Screen>(&self, screen: &mut S) -> Result<(), ScreenError> {
        if self.v != 0 {
            return Ok(())
        }
        let tile_count = if self.resolution() == Low { 32*28 } else { 32*32 };
        screen.set_resolution(
            256,
            tile_count / 4, // tile_count * 64 pixels / (256 pixels/line) gives how many lines
        )?;
        for i in 0 .. tile_count {
            let current_tile_address =
                self.tile_address((2 * i) as u16) as usize;
            let low_byte = self.vram[current_tile_address as u16 as usize];
            let high_byte = self.vram[current_tile_address.wrapping_add(1) as u16 as usize];
            let vert_flip = 4 & high_byte != 0;
            let horiz_flip = 2 & high_byte != 0;
            let palette = 16 * ((high_byte & 8) >> 3) as usize;
            let pattern_index = to16(low_byte, high_byte & 1) as usize;
            for tile_line in 0 .. 8 {
                let palette_indices: [usize; 8] = self.pattern_address_to_palette_indices(
                    pattern_index * 32,
                    tile_line
                );
                let y = if vert_flip {
                    (i / 32) * 8 + (7 - tile_line)
                } else {
                    (i / 32) * 8 + tile_line
                };
                for j in 0 .. 8 {
                    let x = if horiz_flip {
                        (i % 32) * 8 + (7 - j)
                    } else {
                        (i % 32) * 8 + j
                    };
                    screen.paint(x as usize, y as usize, self.cram[palette_indices[j] + palette]);
                }
            }
        }
        let x_scroll = self.x_scroll() as usize;
        for y in 0..tile_count / 4 {
            screen.paint(0xFF - x_scroll, y as usize, 0x0F);
        }
        let y_scroll = self.y_scroll() as usize;
        for x in 0..256 {
            screen.paint(x, y_scroll, 0xF3);
        }
        screen.render()?;
        Ok(())
    }

    fn pattern_address_to_palette_indices(&self, address: usize, line: usize) -> [usize; 8] {
        debug_assert!(line < 8);
        let bitplanes_address = address + 4 * line;
        debug_assert!(bitplanes_address + 3 < self.vram.len());
        let mut bitplane0 = self.vram[bitplanes_address] as usize;
        let mut bitplane1 = self.vram[bitplanes_address + 1] as usize;
        let mut bitplane2 = self.vram[bitplanes_address + 2] as usize;
        let mut bitplane3 = self.vram[bitplanes_address + 3] as usize;
        let mut result = [0usize; 8];
        for i in 0 .. 8 {
            result[i] |= (bitplane0 & 0x80) >> 7;
            result[i] |= (bitplane1 & 0x80) >> 6;
            result[i] |= (bitplane2 & 0x80) >> 5;
            result[i] |= (bitplane3 & 0x80) >> 4;
            bitplane0 <<= 1;
            bitplane1 <<= 1;
            bitplane2 <<= 1;
            bitplane3 <<= 1;
        }
        result
    }
}

impl Irq for Vdp {
    fn requesting_mi(&self) -> bool {
        (self.status_flags.contains(FRAME_INTERRUPT) && self.frame_irq_enable()) ||
            (self.status_flags.contains(LINE_INTERRUPT) && self.line_irq_enable())
    }
    fn requesting_nmi(&self) -> bool {
        false
    }
}
