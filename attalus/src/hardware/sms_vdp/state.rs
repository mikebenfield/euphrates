use std;

use super::{Kind, TvSystem};

/// The VDP has an internal flag indicating whether the control register was the
/// one last written to. Since only 3 bits of the 8 bit status flags are used,
/// we use an extra bit of that register as the control flag.
pub const CONTROL_FLAG: u8 = 0x1;

/// Similar to `CONTROL_FLAG`, we use an extra bit of the 8 bit status flag
/// register to indicate if a line interrupt has occurred.
pub const LINE_INTERRUPT_FLAG: u8 = 0x2;

/// For now this cannot do the Game Gear VDP.
#[derive(Copy)]
pub struct T {
    pub cycles: u64,
    pub kind: Kind,
    pub tv_system: TvSystem,
    pub status_flags: u8,
    pub h: u16,
    pub v: u16,
    pub address: u16,
    pub buffer: u8,
    pub reg: [u8; 11],
    pub cram: [u8; 32],
    pub vram: [u8; 0x4000],
    pub line_counter: u8,
    pub y_scroll: u8,
}

serde_struct_arrays!{
    impl_serde,
    T,
    [cycles, kind, tv_system, status_flags, h, v, address, buffer, reg,
    cram, line_counter, y_scroll,],
    [vram: [u8; 0x4000],],
    []
}

impl std::fmt::Debug for T {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "vdp::Simple \
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
        )
    }
}

impl Default for T {
    fn default() -> Self {
        T {
            cycles: 0,
            kind: Default::default(),
            tv_system: Default::default(),
            status_flags: 0,
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

impl Clone for T {
    fn clone(&self) -> Self {
        *self
    }
}

pub trait Savable {
    fn save(&self) -> T;
}

pub trait Restorable {
    fn restore(&T) -> Self;
}

impl Savable for T {
    fn save(&self) -> T {
        *self
    }
}

impl Restorable for T {
    fn restore(t: &T) -> Self {
        *t
    }
}
