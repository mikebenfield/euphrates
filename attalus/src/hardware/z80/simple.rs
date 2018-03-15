// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use std;

use super::internal;
use super::state;
use super::{InterruptMode, Reg8, Reg16};

impl<S> internal::Impler<S> for state::T
where
    S: ?Sized + AsRef<state::T> + AsMut<state::T>,
{
    #[inline]
    fn cycles(s: &S) -> u64 {
        s.as_ref().cycles
    }

    #[inline]
    fn set_cycles(s: &mut S, x: u64) {
        s.as_mut().cycles = x;
    }

    #[inline]
    fn reg8(s: &S, reg8: Reg8) -> u8 {
        let byte_array: &[u8; 26] = unsafe { std::mem::transmute(&s.as_ref().registers) };
        unsafe { *byte_array.get_unchecked(reg8 as usize) }
    }

    #[inline]
    fn set_reg8(s: &mut S, reg8: Reg8, x: u8) {
        let byte_array: &mut [u8; 26] = unsafe { std::mem::transmute(&mut s.as_mut().registers) };
        unsafe {
            *byte_array.get_unchecked_mut(reg8 as usize) = x;
        }
    }

    #[inline]
    fn reg16(s: &S, reg16: Reg16) -> u16 {
        unsafe { *s.as_ref().registers.get_unchecked(reg16 as usize) }
    }

    #[inline]
    fn set_reg16(s: &mut S, reg16: Reg16, x: u16) {
        unsafe { *s.as_mut().registers.get_unchecked_mut(reg16 as usize) = x }
    }

    #[inline]
    fn halted(s: &S) -> bool {
        s.as_ref().halted
    }

    #[inline]
    fn set_halted(s: &mut S, x: bool) {
        s.as_mut().halted = x;
    }

    #[inline]
    fn iff1(s: &S) -> bool {
        s.as_ref().iff1
    }

    #[inline]
    fn set_iff1(s: &mut S, x: bool) {
        s.as_mut().iff1 = x;
    }

    #[inline]
    fn iff2(s: &S) -> bool {
        s.as_ref().iff2
    }

    #[inline]
    fn set_iff2(s: &mut S, x: bool) {
        s.as_mut().iff2 = x;
    }

    #[inline]
    fn interrupt_mode(s: &S) -> InterruptMode {
        s.as_ref().interrupt_mode
    }

    #[inline]
    fn set_interrupt_mode(s: &mut S, x: InterruptMode) {
        s.as_mut().interrupt_mode = x;
    }
}
