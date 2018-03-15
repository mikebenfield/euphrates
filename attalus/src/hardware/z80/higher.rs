// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use super::internal;
use super::Reg8;

/// Carry flag
pub const CF: u8 = 1 << 0;

/// Subtraction flag
pub const NF: u8 = 1 << 1;

/// Parity/Overflow flag
pub const PF: u8 = 1 << 2;

/// Undocumented flag
pub const XF: u8 = 1 << 3;

/// Half carry flag
pub const HF: u8 = 1 << 4;

/// Undocumented flag
pub const YF: u8 = 1 << 5;

/// Zero flag
pub const ZF: u8 = 1 << 6;

/// Sign flag
pub const SF: u8 = 1 << 7;

pub trait T: internal::T {
    /// Increment the Z80's `cycles` by `x`.
    #[inline]
    fn inc_cycles(&mut self, x: u64) {
        let c = self.cycles();
        self.set_cycles(c + x);
    }

    /// Increment the lower 7 bits of the R register by the given amount,
    /// maintaining the high bit.
    #[inline]
    fn inc_r(&mut self, x: u8) {
        let r = self.reg8(Reg8::R);
        let ir = r.wrapping_add(x) & 0x7F;
        self.set_reg8(Reg8::R, ir | (r & 0x80));
    }

    /// Are these bits set in the F register?
    #[inline]
    fn is_set_flag(&self, x: u8) -> bool {
        (self.reg8(Reg8::F) & x) == x
    }

    /// Set the bits in the F register that are set in `x`.
    #[inline]
    fn set_flag(&mut self, x: u8) {
        let f = self.reg8(Reg8::F);
        self.set_reg8(Reg8::F, f | x);
    }

    /// Clear the bits in the F register that are set in `x`.
    #[inline]
    fn clear_flag(&mut self, x: u8) {
        let f = self.reg8(Reg8::F);
        self.set_reg8(Reg8::F, f & !x);
    }

    /// Set or clear the bits in the F register that are set in `x` according to
    /// whether `y` is true.
    #[inline]
    fn set_flag_by(&mut self, x: u8, y: bool) {
        if y {
            self.set_flag(x);
        } else {
            self.clear_flag(x);
        }
    }

    /// Set or clear the SF flag according to whether the sign bit of `x` is set.
    #[inline]
    fn set_sign(&mut self, x: u8) {
        if x & 0x80 != 0 {
            self.set_flag(SF);
        } else {
            self.clear_flag(SF);
        }
    }

    /// Set or clear the ZF flag according to whether `x` is zero.
    #[inline]
    fn set_zero(&mut self, x: u8) {
        if x == 0 {
            self.set_flag(ZF);
        } else {
            self.clear_flag(ZF);
        }
    }

    /// Set or clear the PF flag according to whether `x` has even parity.
    #[inline]
    fn set_parity(&mut self, x: u8) {
        if x.count_ones() % 2 == 0 {
            self.set_flag(PF);
        } else {
            self.clear_flag(PF);
        }
    }
}
