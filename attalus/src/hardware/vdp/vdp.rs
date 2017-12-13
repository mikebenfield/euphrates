// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either kind 3 of the license or (at your option) any later
// kind. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use std::convert::{AsMut, AsRef};
use std;

use hardware::irq::Irq;
use memo::{Inbox, Outbox};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum TvSystem {
    Ntsc,
    Pal,
}

impl Default for TvSystem {
    fn default() -> TvSystem {
        Ntsc
    }
}

use self::TvSystem::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Kind {
    Sms,
    Sms2,
    Gg,
}

use self::Kind::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Resolution {
    Low = 192,
    Medium = 224,
    High = 240,
}

use self::Resolution::*;

impl Default for Kind {
    fn default() -> Kind {
        Sms
    }
}

bitflags! {
    #[derive(Serialize, Deserialize)]
    struct StatusFlags: u8 {
        const FRAME_INTERRUPT = 0b10000000;
        const SPRITE_OVERFLOW = 0b01000000;
        const SPRITE_COLLISION = 0b00100000;
        const LINE_INTERRUPT = 0b00010000;
        const CONTROL_FLAG = 0b00001000;
    }
}

#[derive(Copy)]
pub struct Component {
    pub cycles: u64,
    pub kind: Kind,
    pub tv_system: TvSystem,
    status_flags: StatusFlags,
    pub h: u16,
    pub v: u16,
    pub address0: u16,
    pub buffer: u8,
    pub reg: [u8; 11],
    pub cram: [u8; 32],
    pub vram: [u8; 0x4000],
    pub line_counter: u8,
    id: u32,
}

serde_struct_arrays!{
    impl_serde,
    Component,
    [cycles, kind, tv_system, status_flags, h, v, address0, buffer, reg,
    cram, line_counter, id,],
    [vram: [u8; 0x4000],],
    []
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum VdpQueryResult {
    Bool(bool),
    Resolution(Resolution),
    Number(u32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum VdpQuery {
    DisableVertScroll,
    DisableHorizScroll,
    LeftColumnBlank,
    LineIrqEnable,
    ShiftSprites,
    Resolution,
    ActiveLines,
    DisplayVisible,
    FrameIrqEnable,
    TallSprites,
    ZoomSprites,
    XScroll,
    YScroll,
    RegLineCounter,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Matchable)]
pub enum Memo {
    ReadV { actual: u16, reported: u8 },

    ReadH { actual: u16, reported: u8 },

    ReadData(u8),

    ReadControl(u8),

    WriteData(u8),

    WriteControl(u8),
}

impl Outbox for Component {
    type Memo = Memo;

    #[inline]
    fn id(&self) -> u32 {
        self.id
    }

    #[inline]
    fn set_id(&mut self, id: u32) {
        self.id = id;
    }
}

impl std::fmt::Debug for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "vdp::Component \
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

impl Default for Component {
    fn default() -> Self {
        Component {
            cycles: 0,
            kind: Default::default(),
            tv_system: Default::default(),
            status_flags: StatusFlags::empty(),
            h: 0,
            v: 0,
            address0: 0,
            reg: [0; 11],
            buffer: 0,
            cram: [0; 32],
            vram: [0; 0x4000],
            line_counter: 0,
            id: 0,
        }
    }
}

impl Clone for Component {
    fn clone(&self) -> Self {
        *self
    }
}

/// Functions that here are implemented in a straightforward way, but may be
/// replaced for better performance.
pub mod replaceable {
    pub fn simple_pattern_to_palette_indices(mut pattern: [u8; 4]) -> [u8; 8] {
        let mut result = [0u8; 8];
        for i in 0..8 {
            result[i] |= (pattern[0] & 0x80) >> 7;
            result[i] |= (pattern[1] & 0x80) >> 6;
            result[i] |= (pattern[2] & 0x80) >> 5;
            result[i] |= (pattern[3] & 0x80) >> 4;
            pattern[0] <<= 1;
            pattern[1] <<= 1;
            pattern[2] <<= 1;
            pattern[3] <<= 1;
        }
        result
    }

    /// This is provided as a static mutable variable so it can be replaced
    /// with a higher performing implementation, such as the one in
    /// the crate `attalus_x64`.
    /// 
    /// This is used by the VDP implementation to interpret bits in tiles and
    /// sprites as indices into a palette.
    /// 
    /// It should do this bit scatter/gather operation.
    /// b07,b06,b05,b04,b03,b02,b01,b00
    /// b17,b16,b15,b14,b13,b12,b11,b10
    /// b27,b26,b25,b24,b23,b22,b21,b20
    /// b37,b36,b35,b34,b33,b32,b31,b30
    /// ----->
    ///   0,  0,  0,  0,b37,b27,b17,b07
    ///   0,  0,  0,  0,b36,b26,b16,b06
    ///   0,  0,  0,  0,b35,b25,b15,b05
    ///   0,  0,  0,  0,b34,b24,b14,b04
    ///   0,  0,  0,  0,b33,b23,b13,b03
    ///   0,  0,  0,  0,b32,b22,b12,b02
    ///   0,  0,  0,  0,b31,b21,b11,b01
    ///   0,  0,  0,  0,b30,b20,b10,b00
    /// above, the bits are listed in logical order as they appear in the arrays
    /// arrays. If the arrays are reinterpreted as unsigned little endian
    /// integers, the operation looks like this:
    /// b37,b36,b35,b34,b33,b32,b31,b30
    /// b27,b26,b25,b24,b23,b22,b21,b20
    /// b17,b16,b15,b14,b13,b12,b11,b10
    /// b07,b06,b05,b04,b03,b02,b01,b00
    /// ----->
    ///   0,  0,  0,  0,b30,b20,b10,b00
    ///   0,  0,  0,  0,b31,b21,b11,b01
    ///   0,  0,  0,  0,b32,b22,b12,b02
    ///   0,  0,  0,  0,b33,b23,b13,b03
    ///   0,  0,  0,  0,b34,b24,b14,b04
    ///   0,  0,  0,  0,b35,b25,b15,b05
    ///   0,  0,  0,  0,b36,b26,b16,b06
    ///   0,  0,  0,  0,b37,b27,b17,b07
    pub static mut PATTERN_TO_PALETTE_INDICES: fn(pattern: [u8; 4]) -> [u8; 8] =
        simple_pattern_to_palette_indices;
}

impl Component {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn code(&self) -> u8 {
        ((self.address0 & 0xC000) >> 14) as u8
    }
    pub fn address(&self) -> u16 {
        self.address0 & 0x3FFF
    }
    pub fn disable_vert_scroll(&self) -> bool {
        self.reg[0] & (1 << 7) != 0
    }
    pub fn disable_horiz_scroll(&self) -> bool {
        self.reg[0] & (1 << 6) != 0
    }
    pub fn left_column_blank(&self) -> bool {
        self.reg[0] & (1 << 5) != 0
    }
    pub fn line_irq_enable(&self) -> bool {
        self.reg[0] & (1 << 4) != 0
    }
    pub fn shift_sprites(&self) -> bool {
        self.reg[0] & (1 << 3) != 0
    }
    pub fn m4(&self) -> bool {
        self.reg[0] & (1 << 2) != 0
    }
    pub fn m2(&self) -> bool {
        self.reg[0] & (1 << 1) != 0
    }
    pub fn nosync(&self) -> bool {
        self.reg[0] & 1 != 0
    }
    pub fn total_lines(&self) -> u16 {
        if self.tv_system == Ntsc { 262 } else { 313 }
    }
    pub fn resolution(&self) -> Resolution {
        match (self.kind, self.m1(), self.m2(), self.m3()) {
            (Sms, _, _, _) => Low,
            (_, true, true, false) => Medium,
            (_, false, true, true) => High,
            (_, _, _, _) => Low,
        }
    }
    pub fn active_lines(&self) -> u16 {
        match self.resolution() {
            Low => 192,
            Medium => 224,
            High => 240,
        }
    }
    pub fn display_visible(&self) -> bool {
        self.reg[1] & (1 << 6) != 0
    }
    pub fn frame_irq_enable(&self) -> bool {
        self.reg[1] & (1 << 5) != 0
    }
    pub fn m1(&self) -> bool {
        self.reg[0] & (1 << 4) != 0
    }
    pub fn m3(&self) -> bool {
        self.reg[0] & (1 << 3) != 0
    }
    pub fn tall_sprites(&self) -> bool {
        self.reg[1] & 2 != 0
    }
    pub fn zoom_sprites(&self) -> bool {
        self.reg[0] & 1 != 0
    }
    pub fn name_table_address(&self) -> u16 {
        if self.kind == Sms {
            ((self.reg[2] as u16) & 0x0F) << 10
        } else if self.resolution() == Low {
            ((self.reg[2] as u16) & 0x0E) << 10
        } else {
            (((self.reg[2] as u16) & 0x0C) << 10) | 0x0700
        }
    }
    pub fn tile_address(&self, tile_offset: u16) -> u16 {
        if self.kind == Sms {
            (self.name_table_address() | 0x03FF) & (tile_offset | 0xF800)
        } else {
            self.name_table_address() + tile_offset
        }
    }
    pub fn sprite_address(&self) -> u16 {
        if self.kind == Sms {
            (self.reg[5] as u16 & 0x7F) << 7
        } else {
            (self.reg[5] as u16 & 0x7E) << 7
        }
    }
    pub fn sprite_pattern_base_address(&self) -> u16 {
        // MacDonald's VDP documentation says the Sms Component does something
        // strange, but that doesn't appear to be true. At least, the games
        // I've tested so far clear reg 6, which, if I implement MacDonald's
        // scheme, causes the sprite patterns to be fetched from the wrong
        // portion of vram
        (self.reg[6] as u16 & 0x04) << 11
    }
    pub fn sprite_pattern_address(&self, pattern_index: u8) -> u16 {
        self.sprite_pattern_base_address() | (pattern_index as u16 * 32)
    }
    pub fn backdrop_color(&self) -> u8 {
        self.reg[7] & 0x0F
    }
    pub fn x_scroll(&self) -> u8 {
        self.reg[8]
    }
    pub fn y_scroll(&self) -> u8 {
        self.reg[9]
    }
    pub fn reg_line_counter(&self) -> u8 {
        self.reg[10]
    }
    pub fn sprite_y(&self, i: u8) -> u8 {
        debug_assert!(i <= 63);
        let address = (i as usize) | ((self.sprite_address() & 0xFF00) as usize);
        self.vram[address].wrapping_add(1)
    }
    pub fn sprite_x(&self, i: u8) -> u8 {
        debug_assert!(i <= 63);
        let address = if self.kind == Sms {
            (2 * i) as usize | (self.sprite_address() as usize)
        } else {
            ((2 * i + 128) as usize) | (self.sprite_address() as usize)
        };
        self.vram[address]
    }
    pub fn sprite_n(&self, i: u8) -> u8 {
        debug_assert!(i <= 63);
        let address = if self.kind == Sms {
            (2 * i) as usize | (self.sprite_address() as usize)
        } else {
            ((2 * i + 128) as usize) | (self.sprite_address() as usize)
        } + 1;
        self.vram[address]
    }
    pub fn inc_address(&mut self) {
        let addr = self.address0;
        self.address0 = (addr.wrapping_add(1) & 0x3FFF) | (addr & 0xC000);
    }
    pub fn trigger_sprite_overflow(&mut self) {
        self.status_flags.insert(StatusFlags::SPRITE_OVERFLOW);
    }
    pub fn trigger_sprite_collision(&mut self) {
        self.status_flags.insert(StatusFlags::SPRITE_COLLISION);
    }

    pub fn query(&self, q: VdpQuery) -> VdpQueryResult {
        use self::VdpQuery::*;
        use self::VdpQueryResult::*;

        match q {
            DisableVertScroll => Bool(self.disable_vert_scroll()),
            DisableHorizScroll => Bool(self.disable_horiz_scroll()),
            LeftColumnBlank => Bool(self.left_column_blank()),
            LineIrqEnable => Bool(self.line_irq_enable()),
            ShiftSprites => Bool(self.shift_sprites()),
            VdpQuery::Resolution => VdpQueryResult::Resolution(self.resolution()),
            ActiveLines => Number(self.active_lines() as u32),
            DisplayVisible => Bool(self.display_visible()),
            FrameIrqEnable => Bool(self.frame_irq_enable()),
            TallSprites => Bool(self.tall_sprites()),
            ZoomSprites => Bool(self.zoom_sprites()),
            XScroll => Number(self.x_scroll() as u32),
            YScroll => Number(self.y_scroll() as u32),
            RegLineCounter => Number(self.reg_line_counter() as u32),
        }
    }

    pub fn drew_line(&mut self) {
        match (self.resolution(), self.v) {
            (Low, 0xC1) => self.status_flags.insert(StatusFlags::FRAME_INTERRUPT),
            (Medium, 0xE1) => self.status_flags.insert(StatusFlags::FRAME_INTERRUPT),
            (High, 0xF1) => self.status_flags.insert(StatusFlags::FRAME_INTERRUPT),
            _ => {}
        }
        if self.v <= self.active_lines() {
            self.line_counter = self.line_counter.wrapping_sub(1);
            if self.line_counter == 0xFF {
                self.line_counter = self.reg_line_counter();
                self.status_flags.insert(StatusFlags::LINE_INTERRUPT);
            }
        } else {
            self.line_counter = self.reg_line_counter();
        }
        self.v = (self.v + 1) % self.total_lines();
        self.cycles += 342;
    }

    #[inline]
    pub fn pattern_address_to_palette_indices(&self, address: usize, line: usize) -> [u8; 8] {
        debug_assert!(line < 16);
        let bitplanes_address = address + 4 * line;
        debug_assert!(bitplanes_address + 3 < self.vram.len());
        let pattern = [
            self.vram[bitplanes_address],
            self.vram[bitplanes_address + 1],
            self.vram[bitplanes_address + 2],
            self.vram[bitplanes_address + 3],
        ];
        unsafe { replaceable::PATTERN_TO_PALETTE_INDICES(pattern) }
    }
}

impl Irq for Component {
    fn requesting_mi(&self) -> Option<u8> {
        if (self.status_flags.contains(StatusFlags::FRAME_INTERRUPT) && self.frame_irq_enable()) ||
            (self.status_flags.contains(StatusFlags::LINE_INTERRUPT) && self.line_irq_enable())
        {
            Some(0xFF)
        } else {
            None
        }
    }
}

pub trait Machine: AsMut<Component> + AsRef<Component> + Inbox<Memo> {
    fn read_v(&mut self) -> u8 {
        let result = match (
            self.as_ref().tv_system,
            self.as_ref().resolution(),
            self.as_ref().v,
        ) {
            (Ntsc, Low, 0...0xDA) => self.as_ref().v,
            (Ntsc, Low, _) => self.as_ref().v - 6,
            (Ntsc, Medium, 0...0xEA) => self.as_ref().v,
            (Ntsc, Medium, _) => self.as_ref().v - 6,
            (Ntsc, High, 0...0xFF) => self.as_ref().v,
            (Ntsc, High, _) => self.as_ref().v - 0x100,
            (Pal, Low, 0...0xF2) => self.as_ref().v,
            (Pal, Low, _) => self.as_ref().v - 57,
            (Pal, Medium, 0...0xFF) => self.as_ref().v,
            (Pal, Medium, 0x100...0x102) => self.as_ref().v - 0x100,
            (Pal, Medium, _) => self.as_ref().v - 57,
            (Pal, High, 0...0xFF) => self.as_ref().v,
            (Pal, High, 0x100...0x10A) => self.as_ref().v - 0x100,
            (Pal, High, _) => self.as_ref().v - 57,
        };
        let id = self.as_ref().id();
        let v = self.as_ref().v;
        self.receive(
            id,
            Memo::ReadV {
                actual: v,
                reported: result as u8,
            },
        );
        result as u8
    }

    fn read_h(&mut self) -> u8 {
        let result = (self.as_ref().h >> 1) as u8;
        let id = self.as_ref().id();
        let h = self.as_ref().h;
        self.receive(
            id,
            Memo::ReadH {
                actual: h,
                reported: result,
            },
        );
        result
    }

    fn read_data(&mut self) -> u8 {
        let current_buffer = self.as_ref().buffer;
        self.as_mut().buffer = self.as_ref().cram[(self.as_ref().address() % 32) as usize];
        self.as_mut().inc_address();
        self.as_mut().status_flags.remove(StatusFlags::CONTROL_FLAG);
        let id = self.as_ref().id();
        self.receive(id, Memo::ReadData(current_buffer));
        current_buffer
    }

    fn read_control(&mut self) -> u8 {
        let current_status = self.as_ref().status_flags.bits;
        self.as_mut().status_flags.bits = 0;
        let id = self.as_ref().id();
        self.receive(id, Memo::ReadControl(current_status));
        current_status
    }

    fn write_data(&mut self, x: u8) {
        let id = self.as_ref().id();
        self.receive(id, Memo::WriteData(x));
        match (self.as_ref().code(), self.as_ref().kind) {
            // XXX - no Game Gear yet
            (3, _) => {
                self.as_mut().cram[(self.as_ref().address() % 32) as usize] = x;
            }
            _ => {
                self.as_mut().vram[self.as_ref().address() as usize] = x;
            }
        }
        self.as_mut().inc_address();
        self.as_mut().status_flags.remove(StatusFlags::CONTROL_FLAG);
    }

    fn write_control(&mut self, x: u8) {
        let id = self.as_ref().id();
        self.receive(id, Memo::WriteControl(x));
        let vdp = self.as_mut();
        if vdp.status_flags.contains(StatusFlags::CONTROL_FLAG) {
            vdp.address0 = vdp.address0 & 0x00FF | (x as u16) << 8;
            if vdp.code() == 0 {
                vdp.buffer = vdp.vram[vdp.address() as usize];
                vdp.address0 = (vdp.address0.wrapping_add(1)) & 0x3FFF | (vdp.code() as u16) << 13;
            } else if vdp.code() == 2 && (x & 0x0F) <= 10 {
                vdp.reg[(x & 0x0F) as usize] = vdp.address0 as u8;
            }
            vdp.status_flags.remove(StatusFlags::CONTROL_FLAG);
        } else {
            vdp.address0 = (vdp.address0 & 0xFF00) | x as u16;
            vdp.status_flags.insert(StatusFlags::CONTROL_FLAG);
        }
    }
}

pub trait MachineImpl {}

impl<T> Machine for T
where
    T: AsMut<Component> + AsRef<Component> + Inbox<Memo> + MachineImpl,
{
}
