// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use super::internal;
use super::replaceable;
use super::{Kind, Resolution, SPRITE_COLLISION_FLAG, SPRITE_OVERFLOW_FLAG, TvSystem};

/// Methods providing a higher level view of the internal components of the VDP
pub trait T: internal::T {
    /// A number in [0, 3], determined by the upper 2 bits of
    /// `code_address`.
    ///
    /// This code is used to determine whether writes to the control port should
    /// read from VRAM or write to a register, and whether writes to the data
    /// port should go to VRAM or CRAM. (See `write_control` and `write_data`.)
    /// XXX
    #[inline]
    fn code(&self) -> u8 {
        ((self.code_address() & 0xC000) >> 14) as u8
    }

    /// The lower 14 bits of `code_address`.
    ///
    /// Used by `write_control` and `write_data` to determine which address in
    /// VRAM or CRAM to write to.
    #[inline]
    fn address(&self) -> u16 {
        self.code_address() & 0x3FFF
    }

    /// Set the lower 14 bits of `code_address` (the upper 2 bits of `x` are
    /// discarded).
    #[inline]
    fn set_address(&mut self, x: u16) {
        let y = x & 0x3FFF;
        let current_code_address = self.code_address();
        self.set_code_address((current_code_address & 0xC000) | y);
    }

    /// Mode select bit 1.
    ///
    /// Bit 4 of register 1 is mode select bit 1. See documentation of
    /// `resolution`.
    #[inline]
    fn m1(&self) -> bool {
        unsafe { self.register_unchecked(1) & (1 << 4) != 0 }
    }

    /// Mode select bit 2.
    ///
    /// Bit 1 of register 0 is mode select bit 2. See documentation of
    /// `resolution`.
    #[inline]
    fn m2(&self) -> bool {
        unsafe { self.register_unchecked(0) & (1 << 1) != 0 }
    }

    /// Mode select bit 3.
    ///
    /// Bit 3 of register 1 is mode select bit 3. See documentation of
    /// `resolution`.
    #[inline]
    fn m3(&self) -> bool {
        unsafe { self.register_unchecked(1) & (1 << 3) != 0 }
    }

    /// Mode select bit 4.
    ///
    /// Bit 2 of register 0 is mode select bit 4. See documentation of
    /// `resolution`.
    #[inline]
    fn m4(&self) -> bool {
        unsafe { self.register_unchecked(0) & (1 << 2) != 0 }
    }

    /// High, Medium, or Low resolution?
    ///
    /// The SMS VDP is always in Low resolution. The SMS2 and GG VDPs have
    /// resolutions determined by mode select bits of registers 0 and 1. (See
    /// methods `m1`, `m2`, `m3`, and `m4`.)
    ///
    /// Actually, there are more modes than just these. The TMS9918 on which the
    /// VDP is based had three documented display modes selected with three mode
    /// select bits, and several undocumented modes selected by various
    /// combinations of these bits. But these VDPs have a fourth mode select
    /// bit. On the SMS, this is used to select "Mode 4", the usual SMS mode,
    /// and the SMS2 and GG VDPs can select higher resolution variations of Mode
    /// 4 using combinations of the mode select bits. The earlier modes still
    /// function on actual VDP hardware, but they are not implemented here, nor
    /// is the API of this trait really conducive to supporting them, because
    /// the `Resolution` enum is not adequate to represent them. I consider this
    /// a reasonable sacrifice because only one game released in the west used
    /// these legacy modes (F-16 Fighting Falcon). The non-western games that
    /// use these modes are apparently ports of games from Sega's earlier
    /// SG-1000 system or are ports of MSX games.
    ///
    /// This returns `Low` resolution for any of the unsupported or invalid
    /// modes, but does send a `Memo` indicating an unsupported mode was
    /// selected.
    #[inline]
    fn resolution(&self) -> Resolution {
        use self::Resolution::*;
        use self::Kind::*;
        match (self.m4(), self.m3(), self.m2(), self.m1(), self.kind()) {
            (true, false, false, false, _) => Low,
            (true, false, true, false, _) => Low,
            (true, true, false, false, _) => Low,
            (true, true, true, false, Sms) => Low,
            (true, false, true, true, Sms2) => Medium,
            (true, false, true, true, Gg) => Medium,
            (true, true, true, false, Sms2) => High,
            (true, true, true, true, Sms2) => Low,
            (true, true, true, true, Gg) => Low,
            (_m4, _m3, _m2, _m1, _) => {
                // XXX - memo
                Low
            }
        }
    }

    /// How many total lines on a VDP of this TV system?
    #[inline]
    fn total_lines(&self) -> u16 {
        if self.tv_system() == TvSystem::Ntsc {
            262
        } else {
            313
        }
    }

    /// Active lines are logically those which are actually rendered on
    /// screen (but for the Game Gear, that's not literally true).
    ///
    /// For the SMS VDP, there are always 192 active lines. For the SMS2 and
    /// Game Gear VDP, the number of active lines is determined by the
    /// `Resolution`, which in turn is determined by the setting of various mode
    /// select bits in VDP registers 0 and 1. See the documentation of the
    /// `resolution` method.
    #[inline]
    fn active_lines(&self) -> u16 {
        if self.kind() == Kind::Sms {
            192
        } else {
            self.resolution() as u16
        }
    }

    /// How many lines are *really* rendered on screen?
    ///
    /// For the SMS and SMS2, the number of visible lines is always the same as
    /// the number of active lines. But for the Game Gear VDP, there are always
    /// 144 lines visible. These are the middle lines of the "active" ones.
    /// Thus, on the Game Gear, when there are 192 active lines, lines [24, 168)
    /// are visible. When there are 224 active lines, lines [42, 186) are
    /// visible. When there are 256 active lines, lines [56, 200) are visible
    /// (except that the Game Gear apparently does not function in 256 line
    /// mode.)
    #[inline]
    fn visible_lines(&self) -> u16 {
        if self.kind() == Kind::Gg {
            144
        } else {
            self.active_lines()
        }
    }

    /// This is always 256 for the SMS and SMS2 VDPs and 160 for the Game Gear
    /// VDP. The Game Gear acts as if it has 256 active columns, but only
    /// columns [48, 209) are actually displayed.
    #[inline]
    fn visible_columns(&self) -> u16 {
        if self.kind() == Kind::Gg { 160 } else { 256 }
    }

    /// Are sprites tall (bit 1 of register 1)?
    ///
    /// Normally, sprites are 8x8 pixels. When this is enabled, they are
    /// 8x16 pixels.
    ///
    /// XXX
    #[inline]
    fn tall_sprites(&self) -> bool {
        unsafe { self.register_unchecked(1) & (1 << 1) != 0 }
    }

    /// Are sprites zoomed (bit 0 of register 0)?
    ///
    /// Normal sprites are 8x8 pixels while tall ones are 8x16. Zooming them
    /// makes them 16x16 or 16x32. There is still only data for 64 or 128 pixels
    /// in the sprite pattern table, but each logical pixel becomes 4 screen
    /// pixels.
    #[inline]
    fn zoomed_sprites(&self) -> bool {
        unsafe { self.register_unchecked(0) & 1 != 0 }
    }

    /// Where in VRAM is the name table?
    ///
    /// Bits 1, 2, and 3 of register 2 are bits 11, 12, and 13 of this value.
    /// Except in Medium or High resolution modes, when bit 11 is always set.
    ///
    /// Guaranteed to return a value no larger than 0x3800, which means there is
    /// just enough room to fit in a name table of 32x32x2 bytes and safely call
    /// `vram_unchecked`.
    #[inline]
    fn name_table_address(&self) -> u16 {
        let address = unsafe { (self.register_unchecked(2) as u16 & 0xE) << 10 };
        if self.resolution() == Resolution::Low {
            address
        } else {
            address | (1 << 11)
        }
    }

    /// Name table mask.
    ///
    /// Whenever looking up a value in the name table, you should AND your
    /// address with this mask. This is due to a bug in the SMS VDP in which
    /// bit 0 of register 2 was ANDed with bit 10 of such an address.
    #[inline]
    fn name_table_mask(&self) -> u16 {
        if self.kind() == Kind::Sms {
            unsafe { (self.register_unchecked(2) as u16) << 10 | 0xFBFF }
        } else {
            0xFF
        }
    }

    /// Horizontal scroll.
    ///
    /// XXX
    /// Taken directly from register 8.
    #[inline]
    fn x_scroll(&self) -> u8 {
        unsafe { self.register_unchecked(8) }
    }

    /// Is the leftmost tile column blanked (bit 5 of register 0)? (Convenience
    /// method.)
    ///
    /// When the leftmost tile column (that is, the leftmost 8 pixels of the
    /// screen) are blanked, those pixels are painted with
    /// `backdrop_color_index`, regardless of anything indicated by the tile or
    /// sprite data in the VRAM.
    #[inline]
    fn left_column_blank(&self) -> bool {
        unsafe { self.register_unchecked(0) & (1 << 5) != 0 }
    }

    /// Is the display monochrome and unsynced (bit 0 of register 0)?
    ///
    /// This is apparently a leftover bit from the TMS9918. It has an effect on
    /// the SMS and SMS2 VDP, but I'm not sure what that effect is and I doubt
    /// any games use it. Consequently the most resonable thing to do in emulation
    /// is probably to ignore this bit.
    #[inline]
    fn display_monochrome(&self) -> bool {
        unsafe { self.register_unchecked(0) & 1 != 0 }
    }

    /// An index into the sprite palette (that is, the second palette)
    /// indicating the color to use when `left_column_blank` is on (low 4 bits
    /// of register 7). (Convenience method.)
    ///
    /// Guaranteed to return a value no greater than 15, and thus can be added
    /// to 16 and still used in `cram_unchecked`.
    #[inline]
    fn backdrop_color_index(&self) -> u8 {
        unsafe { self.register_unchecked(7) & 0xF }
    }

    /// Where in VRAM is the sprite attribute table located?
    ///
    /// Bits 1 through 6 of register 5 form bits 8 through 13 of this address.
    /// Guaranteed to return a value no greater than `0x3F00`. Thus we can add
    /// up to 255 to it and still safely use it as an index into
    /// `vram_unchecked`.
    #[inline]
    fn sprite_attribute_table_address(&self) -> u16 {
        unsafe { (self.register_unchecked(5) as u16 & 0x7E) << 7 }
    }

    /// Sprite attribute address mask.
    ///
    /// The sprite attribute table is where a sprite's x and y coordinates and
    /// pattern index are found. When looking up these items, this mask should
    /// be ANDed with the VRAM address. This is necessary due to a bug in the
    /// SMS VDP, which treated bit 0 of register 5 as a mask against bit 7 of
    /// such lookups. This was fixed in the SMS2 and GG VDPs.
    #[inline]
    fn sprite_attribute_table_mask(&self) -> u16 {
        if self.kind() == Kind::Sms {
            let low_bit = unsafe { self.register_unchecked(5) as u16 & 1 };
            low_bit << 7 | 0xFF7F
        } else {
            0xFFFF
        }
    }

    /// What is the logical pixel x coordinate of this sprite?
    ///
    /// There are only 64 sprites in the sprite attribute table, and this
    /// function is undefined for `sprite_index > 63`.
    ///
    /// This value is found in VRAM at `(sprite_attribute_table_address() + 2 *
    /// sprite_index + 128) & sprite_attribute_table_mask()`.
    #[inline]
    unsafe fn sprite_x(&self, sprite_index: u16) -> u8 {
        let address = (self.sprite_attribute_table_address() + 2 * sprite_index + 128) &
            self.sprite_attribute_table_mask();
        self.vram_unchecked(address)
    }

    /// What is the logical pixel y coordinate of this sprite?
    ///
    /// There are only 64 sprites in the sprite attribute table, and the
    /// result of this function is undefined for `sprite_index > 63`.
    #[inline]
    unsafe fn sprite_y(&self, sprite_index: u16) -> u8 {
        debug_assert!(sprite_index <= 63);
        let address = self.sprite_attribute_table_address() + sprite_index;
        self.vram_unchecked(address).wrapping_add(1)
    }

    /// Where in VRAM is the pattern for this sprite?
    ///
    /// There are only 64 sprites in the sprite attribute table, and the
    /// result of this function is undefined for `sprite_index > 63`.
    ///
    /// Guaranteed to return a value no bigger than `0x3FFC`, and thus this
    /// address and the next three can be safely used in `vram_unchecked`. If
    /// `tall_sprites()` is true, guaranteed to return a value no bigger than
    /// `0x3FF8`, so this address and the next 7 can be safely used in
    /// `vram_unchecked`.
    ///
    /// This address is `(pattern_index * 32 + sprite_pattern_table_address()) &
    /// sprite_pattern_table_mask()`, where `pattern_index` is the byte at VRAM
    /// address `(sprite_attribute_table_address() + 2 * sprite_index + 129) &
    /// sprite_attribute_table_mask`. Except that's not quite right: if
    /// `tall_sprites()`, then the bit 0 of `pattern_index` is set to 0. This
    /// way there are only 128 patterns in the pattern table, and they're each
    /// 64 bytes instead of 32 bytes to accommodate the extra 8 lines.
    #[inline]
    unsafe fn sprite_pattern_address(&self, sprite_index: u16) -> u16 {
        debug_assert!(sprite_index <= 63);
        let pattern_index_address = self.sprite_attribute_table_address() + 2 * sprite_index + 129;
        let pattern_index =
            self.vram_unchecked(pattern_index_address & self.sprite_attribute_table_mask()) as u16;

        let actual_pattern_index = if self.tall_sprites() {
            pattern_index & 0xFE
        } else {
            pattern_index
        };

        (self.sprite_pattern_table_address() + actual_pattern_index * 32) &
            self.sprite_pattern_table_mask()
    }

    /// Where in VRAM is the sprite pattern table located?
    ///
    /// Bit 2 of register 6 becomes bit 13 of this address. Guaranteed to be no
    /// bigger than `0x2000` (and thus you can add up to `0x1FFF` to it and use
    /// it as an index into `vram_unchecked`).
    #[inline]
    fn sprite_pattern_table_address(&self) -> u16 {
        unsafe { (self.register_unchecked(6) as u16 & 0x04) << 11 }
    }

    /// Sprite pattern address mask.
    ///
    /// In the SMS VDP only, the two lowest bits of register 6 act as an AND
    /// mask against bits 11 and 12 of a sprite's pattern address. This is
    /// apparently a bug in the VDP, and does not apply to the SMS2 or GG VDPs.
    /// In any case, this function returns a u16 that should be bitwise ANDed
    /// with any VRAM address to look up a pattern for a sprite.
    ///
    /// MacDonald says these two bits "act as an AND mask over bits 8 and 6 of
    /// the tile index." He only uses the term "tile index" here and one other
    /// place, and I'm not sure what the heck he's talking about. Either this is
    /// an error or else I just don't understand what he means.
    #[inline]
    fn sprite_pattern_table_mask(&self) -> u16 {
        if self.kind() == Kind::Sms {
            let low_bits = unsafe { self.register_unchecked(6) as u16 & 3 };
            low_bits << 11 | 0xE7FF
        } else {
            0xFFFF
        }
    }

    /// Set the sprite overflow flag.
    #[inline]
    fn trigger_sprite_overflow(&mut self) {
        let flags = self.status_flags();
        self.set_status_flags(flags | SPRITE_OVERFLOW_FLAG);
    }

    /// Set the sprite collision flag.
    #[inline]
    fn trigger_sprite_collision(&mut self) {
        let flags = self.status_flags();
        self.set_status_flags(flags | SPRITE_COLLISION_FLAG);
    }

    /// Register line counter, for interrupts.
    #[inline]
    fn reg_line_counter(&self) -> u8 {
        unsafe { self.register_unchecked(10) }
    }

    /// Is vertical scrolling locked on the right (bit 7 of register 0)?
    ///
    /// If vertical scrolling is locked, the rightmost 8 screen character
    /// columns (that is, the rightmost 64 screen pixels) have their screen y
    /// coordinate unaffected by `y_scroll`. This works on the GG VDP, but since
    /// screen pixel columns [209, 256) are not visible, only the 16 screen
    /// pixel columns [192, 209) are affected.
    ///
    /// See XXX for more about vertical scrolling.
    #[inline]
    fn vert_scroll_locked(&self) -> bool {
        unsafe { self.register_unchecked(0) & (1 << 7) != 0 }
    }

    /// Is horizontal scrolling locked on the top (bit 6 of register 0)?
    ///
    /// If horizontal scrolling is locked, the topmost 2 logical character columns
    /// (that is, the topmost 16 logical pixels) have their screen x coordinate
    /// unaffected by `x_scoll`. Since these pixels are not visible on the GG
    /// VDP, this effectively does nothing in that case.
    ///
    /// See XXX for more about horizontal scrolling.
    #[inline]
    fn horiz_scroll_lock(&self) -> bool {
        unsafe { self.register_unchecked(0) & (1 << 6) != 0 }
    }

    /// Is the display visible (bit 6 of register 1)?
    ///
    /// If not, just display every pixel black regardless of what is in CRAM or
    /// VRAM.
    #[inline]
    fn display_visible(&self) -> bool {
        unsafe { self.register_unchecked(1) & (1 << 6) != 0 }
    }

    /// Are sprites shifted 8 pixels to the left (bit 3 of register 0)?
    ///
    /// That is, the screen pixel column is reduced by 8. The purpose of this
    /// is to allow sprites to be partially drawn off the left edge of the screen.
    #[inline]
    fn shift_sprites(&self) -> bool {
        unsafe { self.register_unchecked(0) & (1 << 3) != 0 }
    }

    /// XXX - unsafe?
    #[inline]
    fn pattern_address_to_palette_indices(&self, address: u16, line: u16) -> [u8; 8] {
        debug_assert!(line < 16);
        let bitplanes_address = address + 4 * line;
        debug_assert!(bitplanes_address + 3 < 0x4000);
        let pattern = unsafe {
            [
                self.vram_unchecked(bitplanes_address),
                self.vram_unchecked(bitplanes_address + 1),
                self.vram_unchecked(bitplanes_address + 2),
                self.vram_unchecked(bitplanes_address + 3),
            ]
        };
        unsafe { replaceable::PATTERN_TO_PALETTE_INDICES(pattern) }
    }

    /// Are frame interrupts enabled (bit 5 of register 1).
    ///
    /// See XXX for more about frame interrupts.
    fn frame_irq_enabled(&self) -> bool {
        unsafe { self.register_unchecked(1) & (1 << 5) != 0 }
    }

    /// Are line interrupts enabled (bit 4 of register 0)?
    ///
    /// If line interrupts are enabled, then XXX
    fn line_irq_enabled(&self) -> bool {
        unsafe { self.register_unchecked(0) & (1 << 4) != 0 }
    }

    /// Safely access the Video RAM; panics for indices out of bounds.
    #[inline]
    fn vram(&self, index: u16) -> u8 {
        if index >= 0x4000 {
            panic!("VDP video RAM index {} out of bounds", index);
        }
        unsafe { self.vram_unchecked(index) }
    }

    /// Safely set values in the Video RAM; panics for indices out of bounds.
    #[inline]
    fn set_vram(&mut self, index: u16, value: u8) {
        if index >= 0x4000 {
            panic!("VDP video RAM index {} out of bounds", index);
        }
        unsafe {
            self.set_vram_unchecked(index, value);
        }
    }

    /// Safely access the Color RAM; panics for indices out of bounds.
    #[inline]
    fn cram(&self, index: u16) -> u16 {
        if index >= 32 {
            panic!("VDP color RAM index {} out of bounds", index);
        }
        unsafe { self.cram_unchecked(index) }
    }

    /// Safely set value in the Color RAM; will panic for `index > 31`.
    #[inline]
    fn set_cram(&mut self, index: u16, value: u16) {
        if index >= 32 {
            panic!("VDP color RAM index {} out of bounds", index);
        }
        unsafe {
            self.set_cram_unchecked(index, value);
        }
    }

    /// Safely access a numbered register; panics for `index > 10`.
    #[inline]
    fn register(&self, index: u16) -> u8 {
        if index > 10 {
            panic!("Register index {} out of bounds", index);
        }
        unsafe { self.register_unchecked(index) }
    }

    /// Safely set the value of a numbered register; panics if `index > 10`.
    #[inline]
    fn set_register(&mut self, index: u16, value: u8) {
        if index > 10 {
            panic!("Register index {} out of bounds", index);
        }
        unsafe {
            self.set_register_unchecked(index, value);
        }
    }
}

impl<S: internal::T> T for S {}
