// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

pub trait Irq {
    #[inline(always)]
    fn requesting_mi(&self) -> Option<u8> {
        None
    }

    #[inline(always)]
    fn requesting_nmi(&self) -> bool {
        false
    }

    /// The Z80 responds to nonmaskable interrupts due to the change in voltage
    /// in the NMI pin from high to low, so it will not continually execute
    /// interrupts when the voltage is held low. In software, that means we need
    /// to tell the device the interrupt is being executed and to stop requesting
    /// it.
    #[inline(always)]
    fn clear_nmi(&mut self) {
    }
}
