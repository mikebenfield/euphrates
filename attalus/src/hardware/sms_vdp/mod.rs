//! The Sega Master System's Video Display Processor.

pub mod replaceable;
mod internal;
mod machine;
mod simple;
mod state;

pub use self::internal::*;
pub use self::state::*;
pub use self::simple::*;
pub use self::machine::*;

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

pub mod manifests {
    use memo::{Descriptions::*, Manifest, PayloadType::*};

    pub const DEVICE: &'static str = &"SMS VDP";

    static CONTROL_WRITE_MANIFEST: Manifest = Manifest {
        device: DEVICE,
        summary: "Control write",
        payload_type: U8,
        descriptions: Strings(&["value"]),
    };

    pub static CONTROL_WRITE: &'static Manifest = &CONTROL_WRITE_MANIFEST;

    static CONTROL_READ_MANIFEST: Manifest = Manifest {
        device: DEVICE,
        summary: "Control read",
        payload_type: U8,
        descriptions: Strings(&["value"]),
    };

    pub static CONTROL_READ: &'static Manifest = &CONTROL_READ_MANIFEST;

    static DATA_WRITE_MANIFEST: Manifest = Manifest {
        device: DEVICE,
        summary: "Data write",
        payload_type: U8,
        descriptions: Strings(&["value"]),
    };

    pub static DATA_WRITE: &'static Manifest = &DATA_WRITE_MANIFEST;

    static DATA_READ_MANIFEST: Manifest = Manifest {
        device: DEVICE,
        summary: "Data read",
        payload_type: U8,
        descriptions: Strings(&["reported value", "buffered value"]),
    };

    pub static DATA_READ: &'static Manifest = &DATA_READ_MANIFEST;

    static H_READ_MANIFEST: Manifest = Manifest {
        device: DEVICE,
        summary: "H read",
        payload_type: U8,
        descriptions: Strings(&["actual value", "reported value"]),
    };

    pub static H_READ: &'static Manifest = &H_READ_MANIFEST;

    static V_READ_MANIFEST: Manifest = Manifest {
        device: DEVICE,
        summary: "V read",
        payload_type: U16,
        descriptions: Strings(&["actual value", "reported value"]),
    };

    pub static V_READ: &'static Manifest = &V_READ_MANIFEST;

    static VRAM_WRITE_MANIFEST: Manifest = Manifest {
        device: DEVICE,
        summary: "VRAM write",
        payload_type: U16,
        descriptions: Strings(&["address", "value"]),
    };

    pub static VRAM_WRITE: &'static Manifest = &VRAM_WRITE_MANIFEST;

    static CRAM_WRITE_MANIFEST: Manifest = Manifest {
        device: DEVICE,
        summary: "CRAM write",
        payload_type: U16,
        descriptions: Strings(&["address", "value"]),
    };

    pub static CRAM_WRITE: &'static Manifest = &CRAM_WRITE_MANIFEST;

    static SET_FRAME_INTERRUPT_MANIFEST: Manifest = Manifest {
        device: DEVICE,
        summary: "Frame interrupt set",
        payload_type: U16,
        descriptions: Strings(&["line"]),
    };

    pub static SET_FRAME_INTERRUPT: &'static Manifest = &SET_FRAME_INTERRUPT_MANIFEST;

    static SET_LINE_INTERRUPT_MANIFEST: Manifest = Manifest {
        device: DEVICE,
        summary: "Line interrupt set",
        payload_type: U16,
        descriptions: Strings(&["line", "new line counter"]),
    };

    pub static SET_LINE_INTERRUPT: &'static Manifest = &SET_LINE_INTERRUPT_MANIFEST;

    static RENDERING_MANIFEST: Manifest = Manifest {
        device: DEVICE,
        summary: "Rendering",
        payload_type: U64,
        descriptions: Strings(&[]),
    };

    pub static RENDERING: &'static Manifest = &RENDERING_MANIFEST;

    static REGISTER_SET_MANIFEST: Manifest = Manifest {
        device: DEVICE,
        summary: "Register set",
        payload_type: U8,
        descriptions: Strings(&["register", "value"]),
    };

    pub static REGISTER_SET: &'static Manifest = &REGISTER_SET_MANIFEST;
}
