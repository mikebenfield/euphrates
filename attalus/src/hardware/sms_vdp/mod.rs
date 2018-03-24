//! The Sega Master System's Video Display Processor.
//!
//! The traits here are organized into submodules generally following the Impler
//! pattern.

pub mod internal;
pub mod higher;
pub mod machine;
pub mod replaceable;
pub mod simple;
pub mod part;

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
