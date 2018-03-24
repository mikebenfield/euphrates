//! Provides a simple type implementating the the traits in `sms_vdp`.
use std;

use super::{Kind, TvSystem};

/// The VDP has an internal flag indicating whether the control register was the
/// one last written to. Since only 3 bits of the 8 bit status flags are used,
/// we use an extra bit of that register as the control flag.
const CONTROL_FLAG: u8 = 0x1;

/// Similar to `CONTROL_FLAG`, we use an extra bit of the 8 bit status flag
/// register to indicate if a line interrupt has occurred.
const LINE_INTERRUPT_FLAG: u8 = 0x2;

/// For now this cannot do the Game Gear VDP.
#[derive(Copy)]
pub struct T {
    pub cycles: u64,
    pub kind: Kind,
    pub tv_system: TvSystem,
    status_flags: u8,
    pub h: u16,
    pub v: u16,
    pub address0: u16,
    pub buffer: u8,
    pub reg: [u8; 11],
    pub cram: [u8; 32],
    pub vram: [u8; 0x4000],
    pub line_counter: u8,
    pub y_scroll: u8,
    id: u32,
}

serde_struct_arrays!{
    impl_serde,
    T,
    [cycles, kind, tv_system, status_flags, h, v, address0, buffer, reg,
    cram, line_counter, id, y_scroll,],
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

impl Default for T {
    fn default() -> Self {
        T {
            cycles: 0,
            kind: Default::default(),
            tv_system: Default::default(),
            status_flags: 0,
            h: 0,
            v: 0,
            address0: 0,
            reg: [0; 11],
            buffer: 0,
            cram: [Default::default(); 32],
            vram: [Default::default(); 0x4000],
            line_counter: 0,
            id: 0,
            y_scroll: 0,
        }
    }
}

impl Clone for T {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> super::internal::Impler<S> for T
where
    S: AsMut<T> + AsRef<T>,
{
    #[inline]
    fn cram_latch(_s: &S) -> u8 {
        0
    }

    #[inline]
    fn set_cram_latch(_s: &mut S, _x: u8) {}

    #[inline]
    fn data_buffer(s: &S) -> u8 {
        s.as_ref().buffer
    }

    #[inline]
    fn set_data_buffer(s: &mut S, x: u8) {
        s.as_mut().buffer = x;
    }

    #[inline]
    fn status_flags(s: &S) -> u8 {
        s.as_ref().status_flags & 0xE0
    }

    #[inline]
    fn set_status_flags(s: &mut S, x: u8) {
        let low_bits = s.as_ref().status_flags & 0x1F;
        let high_bits = x & 0xE0;
        s.as_mut().status_flags = low_bits | high_bits;
    }

    #[inline]
    fn control_flag(s: &S) -> bool {
        s.as_ref().status_flags & CONTROL_FLAG != 0
    }

    #[inline]
    fn set_control_flag(s: &mut S, x: bool) {
        if x {
            s.as_mut().status_flags |= CONTROL_FLAG;
        } else {
            s.as_mut().status_flags &= !CONTROL_FLAG;
        }
    }

    #[inline]
    fn line_interrupt_pending(s: &S) -> bool {
        s.as_ref().status_flags & LINE_INTERRUPT_FLAG != 0
    }

    #[inline]
    fn set_line_interrupt_pending(s: &mut S, x: bool) {
        if x {
            s.as_mut().status_flags |= LINE_INTERRUPT_FLAG;
        } else {
            s.as_mut().status_flags &= !LINE_INTERRUPT_FLAG;
        }
    }

    #[inline]
    fn y_scroll(s: &S) -> u8 {
        s.as_ref().y_scroll
    }

    #[inline]
    fn set_y_scroll(s: &mut S, x: u8) {
        s.as_mut().y_scroll = x;
    }

    #[inline]
    fn tv_system(s: &S) -> TvSystem {
        s.as_ref().tv_system
    }

    #[inline]
    fn set_tv_system(s: &mut S, x: TvSystem) {
        s.as_mut().tv_system = x;
    }

    #[inline]
    fn kind(s: &S) -> Kind {
        s.as_ref().kind
    }

    #[inline]
    fn h(s: &S) -> u16 {
        s.as_ref().h
    }

    #[inline]
    fn set_h(s: &mut S, x: u16) {
        s.as_mut().h = x;
    }

    #[inline]
    fn v(s: &S) -> u16 {
        s.as_ref().v
    }

    #[inline]
    fn set_v(s: &mut S, x: u16) {
        s.as_mut().v = x;
    }

    #[inline]
    fn line_counter(s: &S) -> u8 {
        s.as_ref().line_counter
    }

    #[inline]
    fn set_line_counter(s: &mut S, x: u8) {
        s.as_mut().line_counter = x;
    }

    #[inline]
    fn code_address(s: &S) -> u16 {
        s.as_ref().address0
    }

    #[inline]
    fn set_code_address(s: &mut S, x: u16) {
        s.as_mut().address0 = x;
    }

    #[inline]
    fn cycles(s: &S) -> u64 {
        s.as_ref().cycles
    }

    #[inline]
    fn set_cycles(s: &mut S, x: u64) {
        s.as_mut().cycles = x;
    }

    #[inline]
    unsafe fn vram_unchecked(s: &S, index: u16) -> u8 {
        *s.as_ref().vram.get_unchecked(index as usize)
    }

    #[inline]
    unsafe fn set_vram_unchecked(s: &mut S, index: u16, value: u8) {
        *s.as_mut().vram.get_unchecked_mut(index as usize) = value;
    }

    #[inline]
    unsafe fn cram_unchecked(s: &S, index: u16) -> u16 {
        *s.as_ref().cram.get_unchecked(index as usize) as u16
    }

    #[inline]
    unsafe fn set_cram_unchecked(s: &mut S, index: u16, value: u16) {
        *s.as_mut().cram.get_unchecked_mut(index as usize) =  value as u8;
    }

    #[inline]
    unsafe fn register_unchecked(s: &S, index: u16) -> u8 {
        *s.as_ref().reg.get_unchecked(index as usize)
    }

    #[inline]
    unsafe fn set_register_unchecked(s: &mut S, index: u16, value: u8) {
        *s.as_mut().reg.get_unchecked_mut(index as usize) = value;
    }
}
