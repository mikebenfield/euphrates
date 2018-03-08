// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

pub mod internal;
pub mod higher;
pub mod machine;
pub mod replaceable;
pub mod simple;
pub mod part;

mod emulator;

pub use self::emulator::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum TvSystem {
    Ntsc,
    Pal,
}

impl Default for TvSystem {
    fn default() -> TvSystem {
        TvSystem::Ntsc
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Kind {
    Sms,
    Sms2,
    Gg,
}

impl Default for Kind {
    fn default() -> Kind {
        Kind::Sms
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Resolution {
    Low = 192,
    Medium = 224,
    High = 240,
}

const FRAME_INTERRUPT_FLAG: u8 = 0x80;
const SPRITE_OVERFLOW_FLAG: u8 = 0x40;
const SPRITE_COLLISION_FLAG: u8 = 0x20;
