//! An implementation of `Z80Irq` for the Sega Master System.

use super::z80::Z80Irq;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum SmsPauseInterruptState {
    Free,
    InterruptTaken,
    InterruptNeeded,
}

pub struct SmsZ80IrqImpler<'a> {
    pub vdp_interrupt: bool,
    pub pause_interrupt: &'a mut SmsPauseInterruptState,
}

impl Default for SmsPauseInterruptState {
    #[inline]
    fn default() -> Self {
        SmsPauseInterruptState::Free
    }
}

impl<'a> Z80Irq for SmsZ80IrqImpler<'a> {
    #[inline]
    fn requesting_mi(&mut self) -> Option<u8> {
        if self.vdp_interrupt {
            Some(0xFF)
        } else {
            None
        }
    }

    #[inline]
    fn requesting_nmi(&mut self) -> bool {
        *self.pause_interrupt == SmsPauseInterruptState::InterruptNeeded
    }

    #[inline]
    fn take_nmi(&mut self) {
        *self.pause_interrupt = SmsPauseInterruptState::InterruptTaken
    }
}
