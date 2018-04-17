//! Provides a simple type implementating the traits in `sms_vdp`.

use super::*;
use super::{Kind, TvSystem};

pub struct T;

impl<S> SmsVdpInternalImpler<S> for T
where
    S: AsMut<SmsVdpState> + AsRef<SmsVdpState>,
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
        s.as_ref().status_flags & state::CONTROL_FLAG != 0
    }

    #[inline]
    fn set_control_flag(s: &mut S, x: bool) {
        if x {
            s.as_mut().status_flags |= state::CONTROL_FLAG;
        } else {
            s.as_mut().status_flags &= !state::CONTROL_FLAG;
        }
    }

    #[inline]
    fn line_interrupt_pending(s: &S) -> bool {
        s.as_ref().status_flags & state::LINE_INTERRUPT_FLAG != 0
    }

    #[inline]
    fn set_line_interrupt_pending(s: &mut S, x: bool) {
        if x {
            s.as_mut().status_flags |= state::LINE_INTERRUPT_FLAG;
        } else {
            s.as_mut().status_flags &= !state::LINE_INTERRUPT_FLAG;
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
        s.as_ref().address
    }

    #[inline]
    fn set_code_address(s: &mut S, x: u16) {
        s.as_mut().address = x;
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
        *s.as_mut().cram.get_unchecked_mut(index as usize) = value as u8;
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
