//! The Video Display Processor of the Sega Master System.

mod graphics;
mod line;
mod vdp_interface;
mod vdp_internal;
mod vdp_irq;

pub mod replaceable;

pub use self::graphics::*;
pub use self::line::*;
pub use self::vdp_interface::*;
pub use self::vdp_internal::*;
pub use self::vdp_irq::*;

/// NTSC (largely North American and Japan) or PAL (largely Europe and South
/// America) TV system.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum TvSystem {
    Ntsc,
    Pal,
}

impl Default for TvSystem {
    #[inline]
    fn default() -> TvSystem {
        TvSystem::Ntsc
    }
}

/// Master System, Master System 2, or Game Gear VDP?
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Kind {
    Sms,
    Sms2,
    Gg,
}

impl Default for Kind {
    #[inline]
    fn default() -> Kind {
        Kind::Sms2
    }
}

/// Low, Medium or High resolution?
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Resolution {
    Low = 192,
    Medium = 224,
    High = 240,
}

pub const FRAME_INTERRUPT_FLAG: u8 = 0x80;
pub const SPRITE_OVERFLOW_FLAG: u8 = 0x40;
pub const SPRITE_COLLISION_FLAG: u8 = 0x20;
