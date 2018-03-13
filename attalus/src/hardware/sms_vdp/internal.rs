// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use super::{TvSystem, Kind};

/// Methods giving access to the registers and other internal components of the
/// VDP.
///
/// Implementing this trait is just a matter of writing getters and setters.
pub trait T {
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

    /// Vertical scroll.
    ///
    /// Games set the `y_scroll` value by setting register 9, but the value
    /// used is only updated once per frame, so we need to store the an
    /// additional byte.
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
    /// from the control and data ports. See FIXME
    fn code_address(&self) -> u16;

    /// Set the value of the code/address register.
    fn set_code_address(&mut self, u16);

    /// For how many cycles has this VDP been running?
    ///
    /// The VDP takes one cycle to process a pixel; since there are 342 pixels
    /// per line, this will be 364 times the number of times `run_line` has been
    /// called. reference FIXME
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
    /// The result of a call with `index > 31` is indefined.
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
}

pub trait Impler<S>
where
    S: ?Sized,
{
    fn cram_latch(&S) -> u8;
    fn set_cram_latch(&mut S, u8);
    fn data_buffer(&S) -> u8;
    fn set_data_buffer(&mut S, u8);
    fn status_flags(&S) -> u8;
    fn set_status_flags(&mut S, u8);
    fn control_flag(&S) -> bool;
    fn set_control_flag(&mut S, bool);
    fn line_interrupt_pending(&S) -> bool;
    fn set_line_interrupt_pending(&mut S, bool);
    fn y_scroll(&S) -> u8;
    fn set_y_scroll(&mut S, u8);
    fn tv_system(&S) -> TvSystem;
    fn set_tv_system(&mut S, TvSystem);
    fn kind(&S) -> Kind;
    fn h(&S) -> u16;
    fn set_h(&mut S, u16);
    fn v(&S) -> u16;
    fn set_v(&mut S, u16);
    fn line_counter(&S) -> u8;
    fn set_line_counter(&mut S, u8);
    fn code_address(&S) -> u16;
    fn set_code_address(&mut S, u16);
    fn cycles(&S) -> u64;
    fn set_cycles(&mut S, u64);
    unsafe fn vram_unchecked(&S, index: u16) -> u8;
    unsafe fn set_vram_unchecked(&mut S, index: u16, value: u8);
    unsafe fn cram_unchecked(&S, index: u16) -> u16;
    unsafe fn set_cram_unchecked(&mut S, index: u16, value: u16);
    unsafe fn register_unchecked(&S, index: u16) -> u8;
    unsafe fn set_register_unchecked(&mut S, index: u16, value: u8);
}

pub trait Impl {
    type Impler: Impler<Self>;
}

impl<S> T for S
where
    S: Impl,
{
    #[inline]
    fn cram_latch(&self) -> u8 {
        <S::Impler as Impler<Self>>::cram_latch(self)
    }

    #[inline]
    fn set_cram_latch(&mut self, x: u8) {
        <S::Impler as Impler<Self>>::set_cram_latch(self, x)
    }

    #[inline]
    fn data_buffer(&self) -> u8 {
        <S::Impler as Impler<Self>>::data_buffer(self)
    }

    #[inline]
    fn set_data_buffer(&mut self, x: u8) {
        <S::Impler as Impler<Self>>::set_data_buffer(self, x)
    }

    #[inline]
    fn status_flags(&self) -> u8 {
        <S::Impler as Impler<Self>>::status_flags(self)
    }

    #[inline]
    fn set_status_flags(&mut self, x: u8) {
        <S::Impler as Impler<Self>>::set_status_flags(self, x)
    }

    #[inline]
    fn control_flag(&self) -> bool {
        <S::Impler as Impler<Self>>::control_flag(self)
    }

    #[inline]
    fn set_control_flag(&mut self, x: bool) {
        <S::Impler as Impler<Self>>::set_control_flag(self, x)
    }

    #[inline]
    fn line_interrupt_pending(&self) -> bool {
        <S::Impler as Impler<Self>>::line_interrupt_pending(self)
    }

    #[inline]
    fn set_line_interrupt_pending(&mut self, x: bool) {
        <S::Impler as Impler<Self>>::set_line_interrupt_pending(self, x)
    }

    #[inline]
    fn y_scroll(&self) -> u8 {
        <S::Impler as Impler<Self>>::y_scroll(self)
    }

    #[inline]
    fn set_y_scroll(&mut self, x: u8) {
        <S::Impler as Impler<Self>>::set_y_scroll(self, x)
    }

    #[inline]
    fn tv_system(&self) -> TvSystem {
        <S::Impler as Impler<Self>>::tv_system(self)
    }

    #[inline]
    fn set_tv_system(&mut self, x: TvSystem) {
        <S::Impler as Impler<Self>>::set_tv_system(self, x)
    }

    #[inline]
    fn kind(&self) -> Kind {
        <S::Impler as Impler<Self>>::kind(self)
    }

    #[inline]
    fn h(&self) -> u16 {
        <S::Impler as Impler<Self>>::h(self)
    }

    #[inline]
    fn set_h(&mut self, x: u16) {
        <S::Impler as Impler<Self>>::set_h(self, x)
    }

    #[inline]
    fn v(&self) -> u16 {
        <S::Impler as Impler<Self>>::v(self)
    }

    #[inline]
    fn set_v(&mut self, x: u16) {
        <S::Impler as Impler<Self>>::set_v(self, x)
    }

    #[inline]
    fn line_counter(&self) -> u8 {
        <S::Impler as Impler<Self>>::line_counter(self)
    }

    #[inline]
    fn set_line_counter(&mut self, x: u8) {
        <S::Impler as Impler<Self>>::set_line_counter(self, x)
    }

    #[inline]
    fn code_address(&self) -> u16 {
        <S::Impler as Impler<Self>>::code_address(self)
    }

    #[inline]
    fn set_code_address(&mut self, x: u16) {
        <S::Impler as Impler<Self>>::set_code_address(self, x)
    }

    #[inline]
    fn cycles(&self) -> u64 {
        <S::Impler as Impler<Self>>::cycles(self)
    }

    #[inline]
    fn set_cycles(&mut self, x: u64) {
        <S::Impler as Impler<Self>>::set_cycles(self, x)
    }

    #[inline]
    unsafe fn vram_unchecked(&self, index: u16) -> u8 {
        <S::Impler as Impler<Self>>::vram_unchecked(self, index)
    }

    #[inline]
    unsafe fn set_vram_unchecked(&mut self, index: u16, value: u8) {
        <S::Impler as Impler<Self>>::set_vram_unchecked(self, index, value)
    }

    #[inline]
    unsafe fn cram_unchecked(&self, index: u16) -> u16 {
        <S::Impler as Impler<Self>>::cram_unchecked(self, index)
    }

    #[inline]
    unsafe fn set_cram_unchecked(&mut self, index: u16, value: u16) {
        <S::Impler as Impler<Self>>::set_cram_unchecked(self, index, value)
    }

    #[inline]
    unsafe fn register_unchecked(&self, index: u16) -> u8 {
        <S::Impler as Impler<Self>>::register_unchecked(self, index)
    }

    #[inline]
    unsafe fn set_register_unchecked(&mut self, index: u16, value: u8) {
        <S::Impler as Impler<Self>>::set_register_unchecked(self, index, value)
    }
}

