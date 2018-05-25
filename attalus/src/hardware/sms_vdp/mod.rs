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
///
/// Note that, while an actual NTSC VDP runs at about 60 fps and an actual PAL
/// VDP runs at about 50 fps, in this implementation the timing is controlled
/// elsewhere, so setting these values here does not directly control timing.
///
/// Instead, this controls how many total scanlines there are (262 vs 313) and
/// some fiddly details of how the V counter is read.
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

/// Bit 7 of the status register indicates whether a frame interrupt has been
/// triggered (but not necessarily requested).
pub const FRAME_INTERRUPT_FLAG: u8 = 0x80;

/// Bit 6 of the status register indicates whether too many sprites have
/// attempted to be rendered on a single line.
pub const SPRITE_OVERFLOW_FLAG: u8 = 0x40;


/// Bit 5 of the status register indicates whether two sprites overlapped.
pub const SPRITE_COLLISION_FLAG: u8 = 0x20;
