//! An implementation of `Z80Irq` for the Sega Master System.

use std::cell::RefCell;
use std::rc::Rc;

use hardware::sms_vdp::SmsVdpInternal;
use hardware::z80::Z80Irq;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum SmsPauseInterruptState {
    Free,
    InterruptTaken,
    InterruptNeeded,
}

pub struct SmsZ80IrqImpler<'a, V: 'a> {
    pub vdp: Rc<RefCell<&'a mut V>>,
    pub pause_interrupt: &'a mut SmsPauseInterruptState,
}

impl Default for SmsPauseInterruptState {
    #[inline]
    fn default() -> Self {
        SmsPauseInterruptState::Free
    }
}

impl SmsPauseInterruptState {
    #[inline]
    pub fn pause_pressed(&mut self, x: bool) {
        if x && *self == SmsPauseInterruptState::Free {
            *self = SmsPauseInterruptState::InterruptNeeded;
        }
        if !x && *self == SmsPauseInterruptState::InterruptTaken {
            *self = SmsPauseInterruptState::Free;
        }
    }
}

impl<'a, V: 'a> Z80Irq for SmsZ80IrqImpler<'a, V>
where
    V: SmsVdpInternal,
{
    #[inline]
    fn requesting_mi(&mut self) -> Option<u8> {
        if self.vdp.borrow().requesting_interrupt() {
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
