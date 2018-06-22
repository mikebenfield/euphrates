//! An implementation of `Z80Irq` for the Sega Master System.

use impler::{Cref, Impl, Mref, Ref};

use super::sms_vdp::SmsVdpInternal;
use super::z80::Z80Irq;

pub trait SmsPauseInterrupt {
    fn requesting_interrupt(&self) -> bool;
    fn take_interrupt(&mut self);
    fn set_pause(&mut self, _: bool);
}

pub struct SmsPauseInterruptImpl;

impl<T> SmsPauseInterrupt for T
where
    T: Impl<SmsPauseInterruptImpl> + ?Sized,
    T::Impler: SmsPauseInterrupt,
{
    #[inline(always)]
    fn requesting_interrupt(&self) -> bool {
        self.make().requesting_interrupt()
    }

    #[inline(always)]
    fn take_interrupt(&mut self) {
        self.make_mut().take_interrupt()
    }

    #[inline(always)]
    fn set_pause(&mut self, x: bool) {
        self.make_mut().set_pause(x)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum SmsPauseInterruptState {
    Free,
    InterruptTaken,
    InterruptNeeded,
}

impl Default for SmsPauseInterruptState {
    #[inline]
    fn default() -> Self {
        SmsPauseInterruptState::Free
    }
}

impl SmsPauseInterrupt for SmsPauseInterruptState {
    #[inline]
    fn requesting_interrupt(&self) -> bool {
        *self == SmsPauseInterruptState::InterruptNeeded
    }

    #[inline]
    fn take_interrupt(&mut self) {
        *self = SmsPauseInterruptState::InterruptTaken
    }

    #[inline]
    fn set_pause(&mut self, x: bool) {
        use self::SmsPauseInterruptState::*;
        match (x, *self) {
            (true, Free) => *self = InterruptNeeded,
            (false, InterruptTaken) => *self = Free,
            _ => {}
        }
    }
}

/// An Impler for Z80Irq.
///
/// `T` must implement `SmsVdpIrq`, `SmsPlayerInput`, and `SmsZ80IrqState`.
pub struct SmsZ80IrqImpler<T: ?Sized>(Ref<T>);

impl<T: ?Sized> SmsZ80IrqImpler<T> {
    #[inline(always)]
    pub fn new<'a>(t: &'a T) -> Cref<'a, Self> {
        Cref::Own(SmsZ80IrqImpler(unsafe { Ref::new(t) }))
    }

    #[inline(always)]
    pub fn new_mut<'a>(t: &'a mut T) -> Mref<'a, Self> {
        Mref::Own(SmsZ80IrqImpler(unsafe { Ref::new_mut(t) }))
    }
}

impl<T> Z80Irq for SmsZ80IrqImpler<T>
where
    T: SmsVdpInternal + SmsPauseInterrupt + ?Sized,
{
    #[inline]
    fn requesting_mi(&self) -> Option<u8> {
        if SmsVdpInternal::requesting_mi(self.0._0()) {
            Some(0xFF)
        } else {
            None
        }
    }

    #[inline]
    fn requesting_nmi(&self) -> bool {
        SmsPauseInterrupt::requesting_interrupt(self.0._0())
    }

    #[inline]
    fn take_nmi(&mut self) {
        SmsPauseInterrupt::take_interrupt(self.0.mut_0())
    }
}
