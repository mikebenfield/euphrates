use std;

use impler::Impl;

use super::Kind;
use super::*;

/// The memory and registers of the VDP.
pub trait SmsVdpInternal {
    /// A latch register used by the GameGear VDP.
    ///
    /// For the Game Gear VDP only, using the data port to write to CRAM (that
    /// is, calling `write_data` when `code() == 3`) when the address register
    /// is even will not actually write to CRAM, but will instead write to this
    /// latch register. Writing when the address register is even will write to
    /// the u16 in the CRAM address determined by bits [1,5] of `address()`.
    /// The low byte of the value is determined by latch; the high byte is
    /// the value just passed to `write_data`.
    ///
    /// SMS and SMS2 VDPs do not use this latch register; for them, it doesn't
    /// matter what this returns.
    fn cram_latch(&self) -> u8;

    /// Set the value of the CRAM latch register used by the GameGear VDP.
    ///
    /// See the documentation for the `cram_latch` method.
    fn set_cram_latch(&mut self, u8);

    /// Buffer for data port reads.
    fn data_buffer(&self) -> u8;

    /// Set the value of the buffer for data port reads.
    fn set_data_buffer(&mut self, u8);

    /// Byte indicating three status flags.
    fn status_flags(&self) -> u8;

    /// Set the status flags byte.
    fn set_status_flags(&mut self, u8);

    /// Flag indicating whether the control port has been written to.
    fn control_flag(&self) -> bool;

    /// Set the flag indicating whether the control port has been written to.
    fn set_control_flag(&mut self, bool);

    /// Has the line interrupt flag been triggered?
    fn line_interrupt_pending(&self) -> bool;

    /// Set the line interrupt pending flag.
    fn set_line_interrupt_pending(&mut self, bool);

    /// Has this interrupt been triggered since the last time an interrupt was
    /// taken?
    fn new_irq(&self) -> bool;

    fn set_new_irq(&mut self, bool);

    /// Vertical scroll.
    ///
    /// Games set the `y_scroll` value by setting register 9, but the value used
    /// is only updated once per frame, so we need to store an additional byte.
    fn y_scroll(&self) -> u8;

    /// Set the vertical scroll.
    fn set_y_scroll(&mut self, u8);

    /// Get the TvSystem supported by this VDP.
    fn tv_system(&self) -> TvSystem;

    /// Set the TvSystem for this VDP.
    fn set_tv_system(&mut self, TvSystem);

    /// Is this a Sms, Sms2, or Gg VDP?
    ///
    /// There is no setter for `kind` because a given implementation may not
    /// support all 3.
    fn kind(&self) -> Kind;

    /// The horizontal counter.
    ///
    /// `h` is incremented as the VDP processes pixels across each line,
    /// beginning at 0. Since there are 342 pixels in each line,
    /// `h` thus should vary in [0, 341].
    fn h(&self) -> u16;

    /// Set the value of the horizontal counter.
    fn set_h(&mut self, u16);

    /// The vertical counter.
    ///
    /// Beginning at 0 at each frame, `v` is incremented after the VDP processes
    /// each line. There are 262 total lines in the NTSC TV system, so there `v`
    /// will vary in [0, 261]. There are 313 total lines in the PAL TV system,
    /// so there `v` will vary in [0, 312].
    fn v(&self) -> u16;

    /// Set the value of the vertical counter.
    fn set_v(&mut self, u16);

    /// The line counter.
    ///
    /// The liner counter is used by the VDP to determine when to trigger a line
    /// interrupt. After rendering an active line, and the first line past the
    /// active line, the line counter is decremented. If it wraps past 0, a line
    /// interrupt is triggered and it is reset to the value of register 10.
    fn line_counter(&self) -> u8;

    /// Set the value of the line counter.
    fn set_line_counter(&mut self, u8);

    /// The code/address register.
    ///
    /// The upper 2 bits contain a code used to determine the effects
    /// of writes to the control and data ports. The lower 14 bits contain
    /// an address used in accesses to the VRAM and CRAM through reads
    /// from the control and data ports.
    fn code_address(&self) -> u16;

    /// Set the value of the code/address register.
    fn set_code_address(&mut self, u16);

    /// For how many cycles has this VDP been running?
    ///
    /// The VDP takes one cycle to process a pixel; since there are 342 pixels
    /// per line, this will be 342 times the number of times `draw_line` from
    /// `machine::SmsVdpInternal` has been called.
    fn cycles(&self) -> u64;

    /// Set the number of cycles this VDP has been running.
    fn set_cycles(&mut self, u64);

    /// Access the Video RAM.
    ///
    /// The VRAM should be 16 KiB (`0x4000` bytes); accesses in that range
    /// should be safe. Accesses out of that range are undefined.
    unsafe fn vram_unchecked(&self, index: u16) -> u8;

    /// Set values in the Video RAM.
    ///
    /// The result for `index` greater than or equal to `0x4000` is undefined.
    unsafe fn set_vram_unchecked(&mut self, index: u16, value: u8);

    /// Access the Color RAM.
    ///
    /// The Color RAM is an array of 32 `u8`s for a SMS or SMS2 VDP, and an
    /// array of 32 `u16`s for a GG VDP. An SMS or SMS2 VDP returns the value of
    /// the `u8` at `index` in the least significant byte of the returned `u16`.
    ///
    /// The result of a call with `index > 31` is undefined.
    unsafe fn cram_unchecked(&self, index: u16) -> u16;

    /// Set values in the Color RAM.
    ///
    /// The SMS and SMS2 VDPs should use the least significant byte of `value`.
    ///
    /// The result for `index > 31` is undefined.
    unsafe fn set_cram_unchecked(&mut self, index: u16, value: u16);

    /// Access a numbered register.
    ///
    /// The VDP has 11 internal numbered registers.
    ///
    /// Undefined for `index > 10`.
    unsafe fn register_unchecked(&self, index: u16) -> u8;

    /// Set the value of a numbered register.
    ///
    /// Undefined for `index > 10`.
    unsafe fn set_register_unchecked(&mut self, index: u16, value: u8);

    /// A number in [0, 3], determined by the upper 2 bits of `code_address`.
    ///
    /// This code is used to determine whether writes to the control port should
    /// read from VRAM or write to a register, and whether writes to the data
    /// port should go to VRAM or CRAM. (See `write_control` and `write_data`.)
    #[inline]
    fn code(&self) -> u8 {
        (self.code_address() >> 14) as u8
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
        use self::Kind::*;
        use self::Resolution::*;

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
        if self.kind() == Kind::Gg {
            160
        } else {
            256
        }
    }

    /// Are sprites tall (bit 1 of register 1)?
    ///
    /// Normally, sprites are 8x8 pixels. When this is enabled, they are 8x16
    /// pixels.
    #[inline]
    fn tall_sprites(&self) -> bool {
        unsafe { self.register_unchecked(1) & (1 << 1) != 0 }
    }

    /// Are sprites zoomed (bit 0 of register 1)?
    ///
    /// Normal sprites are 8x8 pixels while tall ones are 8x16. Zooming them
    /// makes them 16x16 or 16x32. There is still only data for 64 or 128 pixels
    /// in the sprite pattern table, but each logical pixel becomes 4 screen
    /// pixels.
    #[inline]
    fn zoomed_sprites(&self) -> bool {
        unsafe { self.register_unchecked(1) & 1 != 0 }
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
            (address & 0xF7FF) | 0x0700
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
        let address = (self.sprite_attribute_table_address() + 2 * sprite_index + 128)
            & self.sprite_attribute_table_mask();
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
    /// Guaranteed to return a value no bigger than `0x3FFE0`, and thus this
    /// address and the next 31 can be safely used in `vram_unchecked`. If
    /// `tall_sprites()` is true, guaranteed to return a value no bigger than
    /// `0x3FC0`, so this address and the next 63 can be safely used in
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

        (self.sprite_pattern_table_address() + actual_pattern_index * 32)
            & self.sprite_pattern_table_mask()
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
    #[inline]
    fn horiz_scroll_locked(&self) -> bool {
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

    /// Given a pattern at `address`, return the indices of the desired palette.
    ///
    /// `address + 4 * line + 3` should be less than 0x4000. If `address` was
    /// correctly obtained from `sprite_pattern_address` and `line` is smaller
    /// than 8 (or smaller than 16 if `tall_sprites` is true), this will be the
    /// case.
    #[inline]
    unsafe fn pattern_address_to_palette_indices(&self, address: u16, line: u16) -> [u8; 8] {
        debug_assert!(line < 16);
        let bitplanes_address = address + 4 * line;
        debug_assert!(bitplanes_address + 3 < 0x4000);
        let pattern = [
            self.vram_unchecked(bitplanes_address),
            self.vram_unchecked(bitplanes_address + 1),
            self.vram_unchecked(bitplanes_address + 2),
            self.vram_unchecked(bitplanes_address + 3),
        ];
        replaceable::PATTERN_TO_PALETTE_INDICES(pattern)
    }

    /// Are frame interrupts enabled (bit 5 of register 1)?
    #[inline]
    fn frame_irq_enabled(&self) -> bool {
        unsafe { self.register_unchecked(1) & (1 << 5) != 0 }
    }

    /// Are line interrupts enabled (bit 4 of register 0)?
    #[inline]
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

    /// Hardware method: is the VDP requesting an interrupt?
    #[inline]
    fn requesting_mi(&self) -> bool {
        let frame_interrupt = self.status_flags() & FRAME_INTERRUPT_FLAG != 0;
        let line_interrupt = self.line_interrupt_pending();
        self.new_irq()
            && ((frame_interrupt && self.frame_irq_enabled())
                || (line_interrupt && self.line_irq_enabled()))
    }

    /// Set power-on register settings.
    ///
    /// These are set according to Sega's official documentation, which I have
    /// for the Mark III and Game Gear.
    fn reset_defaults(&mut self) {
        if self.kind() == Kind::Gg {
            // according to Sega Game Gear Hardware Reference Manual, pg 21
            self.set_register(0, 0);
            self.set_register(1, 0);
            self.set_register(2, 0); // (not set, according to manual)
            self.set_register(3, 0); // (not set, according to manual)
            self.set_register(4, 0); // (not set, according to manual)
            self.set_register(5, 0); // (not set, according to manual)
            self.set_register(6, 0); // (not set, according to manual)
            self.set_register(7, 0);
            self.set_register(8, 0);
            self.set_register(9, 0);
            self.set_register(10, 1); // really? that's what the manual says
        } else {
            // according to Sega Mk3 Software Reference Manual, pg 18
            self.set_register(0, 0x36);
            self.set_register(1, 0xA0);
            self.set_register(2, 0xFF);
            self.set_register(3, 0xFF);
            self.set_register(4, 0xFF);
            self.set_register(5, 0xFF);
            self.set_register(6, 0xFB);
            self.set_register(7, 0x00);
            self.set_register(8, 0x00);
            self.set_register(9, 0x00);
            self.set_register(10, 0xFF);
        }
    }
}

/// For use in the Impler pattern for `SmsVdpInternal`.
pub struct SmsVdpInternalImpl;

impl<T> SmsVdpInternal for T
where
    T: Impl<SmsVdpInternalImpl> + ?Sized,
    T::Impler: SmsVdpInternal,
{
    #[inline]
    fn cram_latch(&self) -> u8 {
        self.make().cram_latch()
    }

    #[inline]
    fn set_cram_latch(&mut self, x: u8) {
        self.make_mut().set_cram_latch(x)
    }

    #[inline]
    fn data_buffer(&self) -> u8 {
        self.make().data_buffer()
    }

    #[inline]
    fn set_data_buffer(&mut self, x: u8) {
        self.make_mut().set_data_buffer(x)
    }

    #[inline]
    fn status_flags(&self) -> u8 {
        self.make().status_flags()
    }

    #[inline]
    fn set_status_flags(&mut self, x: u8) {
        self.make_mut().set_status_flags(x)
    }

    #[inline]
    fn control_flag(&self) -> bool {
        self.make().control_flag()
    }

    #[inline]
    fn set_control_flag(&mut self, x: bool) {
        self.make_mut().set_control_flag(x)
    }

    #[inline]
    fn line_interrupt_pending(&self) -> bool {
        self.make().line_interrupt_pending()
    }

    #[inline]
    fn set_line_interrupt_pending(&mut self, x: bool) {
        self.make_mut().set_line_interrupt_pending(x)
    }

    #[inline]
    fn new_irq(&self) -> bool {
        self.make().new_irq()
    }

    #[inline]
    fn set_new_irq(&mut self, x: bool) {
        self.make_mut().set_new_irq(x)
    }

    #[inline]
    fn y_scroll(&self) -> u8 {
        self.make().y_scroll()
    }

    #[inline]
    fn set_y_scroll(&mut self, x: u8) {
        self.make_mut().set_y_scroll(x)
    }

    #[inline]
    fn tv_system(&self) -> TvSystem {
        self.make().tv_system()
    }

    #[inline]
    fn set_tv_system(&mut self, x: TvSystem) {
        self.make_mut().set_tv_system(x)
    }

    #[inline]
    fn kind(&self) -> Kind {
        self.make().kind()
    }

    #[inline]
    fn h(&self) -> u16 {
        self.make().h()
    }

    #[inline]
    fn set_h(&mut self, x: u16) {
        self.make_mut().set_h(x)
    }

    #[inline]
    fn v(&self) -> u16 {
        self.make().v()
    }

    #[inline]
    fn set_v(&mut self, x: u16) {
        self.make_mut().set_v(x)
    }

    #[inline]
    fn line_counter(&self) -> u8 {
        self.make().line_counter()
    }

    #[inline]
    fn set_line_counter(&mut self, x: u8) {
        self.make_mut().set_line_counter(x)
    }

    #[inline]
    fn code_address(&self) -> u16 {
        self.make().code_address()
    }

    #[inline]
    fn set_code_address(&mut self, x: u16) {
        self.make_mut().set_code_address(x)
    }

    #[inline]
    fn cycles(&self) -> u64 {
        self.make().cycles()
    }

    #[inline]
    fn set_cycles(&mut self, x: u64) {
        self.make_mut().set_cycles(x)
    }

    #[inline]
    unsafe fn vram_unchecked(&self, index: u16) -> u8 {
        self.make().vram_unchecked(index)
    }

    #[inline]
    unsafe fn set_vram_unchecked(&mut self, index: u16, value: u8) {
        self.make_mut().set_vram_unchecked(index, value)
    }

    #[inline]
    unsafe fn cram_unchecked(&self, index: u16) -> u16 {
        self.make().cram_unchecked(index)
    }

    #[inline]
    unsafe fn set_cram_unchecked(&mut self, index: u16, value: u16) {
        self.make_mut().set_cram_unchecked(index, value)
    }

    #[inline]
    unsafe fn register_unchecked(&self, index: u16) -> u8 {
        self.make().register_unchecked(index)
    }

    #[inline]
    unsafe fn set_register_unchecked(&mut self, index: u16, value: u8) {
        self.make_mut().set_register_unchecked(index, value)
    }
}

/// The state of the VDP.
///
/// Suitable for serializing.
#[derive(Clone, Copy)]
pub struct SmsVdpState {
    pub cycles: u64,
    pub kind: Kind,
    pub tv_system: TvSystem,
    pub cram_latch: u8,
    pub status_flags: u8,
    /// Control flag is bit 0. Line interrupt flag is bit 1.
    /// new_irq is bit 2.
    pub other_flags: u8,
    pub h: u16,
    pub v: u16,
    pub address: u16,
    pub buffer: u8,
    pub reg: [u8; 11],
    pub cram: [u16; 32],
    pub vram: [u8; 0x4000],
    pub line_counter: u8,
    pub y_scroll: u8,
}

mod _impl0 {
    use super::super::*;
    use super::SmsVdpState;

    #[derive(PartialEq, Eq, Hash, Serialize, Deserialize)]
    struct SmsVdpStateDerive {
        cycles: u64,
        kind: Kind,
        tv_system: TvSystem,
        cram_latch: u8,
        status_flags: u8,
        other_flags: u8,
        h: u16,
        v: u16,
        address: u16,
        buffer: u8,
        reg: [u8; 11],
        cram: [u16; 32],
        vram: [[[u8; 0x20]; 0x20]; 0x10],
        line_counter: u8,
        y_scroll: u8,
    }
    impl_serde_via!{SmsVdpState, SmsVdpStateDerive}
    impl_hash_via!{SmsVdpState, SmsVdpStateDerive}
    impl_partial_eq_via!{SmsVdpState, SmsVdpStateDerive}
    impl Eq for SmsVdpState {}
    impl Default for SmsVdpState {
        fn default() -> Self {
            SmsVdpState {
                cycles: 0,
                kind: Default::default(),
                tv_system: Default::default(),
                cram_latch: 0,
                status_flags: 0,
                other_flags: 0,
                h: 0,
                v: 0,
                address: 0,
                reg: [0; 11],
                buffer: 0,
                cram: [Default::default(); 32],
                vram: [Default::default(); 0x4000],
                line_counter: 0,
                y_scroll: 0,
            }
        }
    }
}

impl std::fmt::Debug for SmsVdpState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        format_args!(
            "SmsVdpState \
            {{ \n\
                status_flags: {:?}, \n\
                h: {:?}, \n\
                v: {:?}, buffer: {:?}, address: {:?}, \n\
                reg: {:?}, \n\
                cram: {:?}, \n\
                vram: {:?} (...) \n
            }}",
            self.status_flags,
            self.h,
            self.v,
            self.buffer,
            self.address,
            self.reg,
            self.cram,
            &self.vram[0..32]
        ).fmt(f)
    }
}

impl SmsVdpInternal for SmsVdpState {
    #[inline]
    fn cram_latch(&self) -> u8 {
        self.cram_latch
    }

    #[inline]
    fn set_cram_latch(&mut self, x: u8) {
        self.cram_latch = x;
    }

    #[inline]
    fn data_buffer(&self) -> u8 {
        self.buffer
    }

    #[inline]
    fn set_data_buffer(&mut self, x: u8) {
        self.buffer = x;
    }

    #[inline]
    fn status_flags(&self) -> u8 {
        self.status_flags & 0xE0
    }

    #[inline]
    fn set_status_flags(&mut self, x: u8) {
        let low_bits = self.status_flags & 0x1F;
        let high_bits = x & 0xE0;
        self.status_flags = low_bits | high_bits;
    }

    #[inline]
    fn control_flag(&self) -> bool {
        self.other_flags & 1 != 0
    }

    #[inline]
    fn set_control_flag(&mut self, x: bool) {
        if x {
            self.other_flags |= 1
        } else {
            self.other_flags &= 0xFE
        }
    }

    #[inline]
    fn line_interrupt_pending(&self) -> bool {
        self.other_flags & 2 != 0
    }

    #[inline]
    fn set_line_interrupt_pending(&mut self, x: bool) {
        if x {
            self.other_flags |= 2
        } else {
            self.other_flags &= 0xFD
        }
    }

    #[inline]
    fn new_irq(&self) -> bool {
        self.other_flags & 4 != 0
    }

    #[inline]
    fn set_new_irq(&mut self, x: bool) {
        if x {
            self.other_flags |= 4
        } else {
            self.other_flags &= 0xFB
        }
    }

    #[inline]
    fn y_scroll(&self) -> u8 {
        self.y_scroll
    }

    #[inline]
    fn set_y_scroll(&mut self, x: u8) {
        self.y_scroll = x;
    }

    #[inline]
    fn tv_system(&self) -> TvSystem {
        self.tv_system
    }

    #[inline]
    fn set_tv_system(&mut self, x: TvSystem) {
        self.tv_system = x;
    }

    #[inline]
    fn kind(&self) -> Kind {
        self.kind
    }

    #[inline]
    fn h(&self) -> u16 {
        self.h
    }

    #[inline]
    fn set_h(&mut self, x: u16) {
        self.h = x;
    }

    #[inline]
    fn v(&self) -> u16 {
        self.v
    }

    #[inline]
    fn set_v(&mut self, x: u16) {
        self.v = x;
    }

    #[inline]
    fn line_counter(&self) -> u8 {
        self.line_counter
    }

    #[inline]
    fn set_line_counter(&mut self, x: u8) {
        self.line_counter = x;
    }

    #[inline]
    fn code_address(&self) -> u16 {
        self.address
    }

    #[inline]
    fn set_code_address(&mut self, x: u16) {
        self.address = x;
    }

    #[inline]
    fn cycles(&self) -> u64 {
        self.cycles
    }

    #[inline]
    fn set_cycles(&mut self, x: u64) {
        self.cycles = x;
    }

    #[inline]
    unsafe fn vram_unchecked(&self, index: u16) -> u8 {
        *self.vram.get_unchecked(index as usize)
    }

    #[inline]
    unsafe fn set_vram_unchecked(&mut self, index: u16, value: u8) {
        *self.vram.get_unchecked_mut(index as usize) = value;
    }

    #[inline]
    unsafe fn cram_unchecked(&self, index: u16) -> u16 {
        *self.cram.get_unchecked(index as usize)
    }

    #[inline]
    unsafe fn set_cram_unchecked(&mut self, index: u16, value: u16) {
        *self.cram.get_unchecked_mut(index as usize) = value;
    }

    #[inline]
    unsafe fn register_unchecked(&self, index: u16) -> u8 {
        *self.reg.get_unchecked(index as usize)
    }

    #[inline]
    unsafe fn set_register_unchecked(&mut self, index: u16, value: u8) {
        *self.reg.get_unchecked_mut(index as usize) = value;
    }
}
