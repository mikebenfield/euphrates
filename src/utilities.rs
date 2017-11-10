// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

pub fn to16(lo: u8, hi: u8) -> u16 {
    ((hi as u16) << 8) | (lo as u16)
}

pub fn to8(x: u16) -> (u8, u8) {
    ((x & 0xFF) as u8, ((x & 0xFF00) >> 8) as u8)
}

pub fn set_bit(dest: &mut u8, bit: u8) {
    *dest |= 1 << bit;
}

pub fn clear_bit(dest: &mut u8, bit: u8) {
    *dest &= !(1 << bit);
}
