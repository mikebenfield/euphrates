//! An implementation of `Z80Irq` for the Sega Master System.

use impler::{Cref, Impl, Mref, Ref};

use super::sms_player_input::SmsPlayerInput;
use super::sms_vdp::SmsVdpIrq;
use super::z80::Z80Irq;

/// What's the state of the IRQ system?
///
/// This is just necessary to debounce the pause button, which is what triggers
/// nonmaskable interrupts in the SMS (of all things... you'd think it would be
/// the reset button). `SmsZ80IrqState` maintains a flag indicating whether the
/// current pause button press has already been accepted as an interrupt. If so,
/// it doesn't report another. The next time it detects the pause button is not
/// pressed, it turns the flag off.
pub trait SmsZ80IrqState {
    fn already_accepted(&self) -> bool;
    fn set_already_accepted(&mut self, x: bool);
}

impl SmsZ80IrqState for bool {
    fn already_accepted(&self) -> bool {
        *self
    }

    fn set_already_accepted(&mut self, x: bool) {
        *self = x
    }
}

pub struct SmsZ80IrqStateImpl;

impl<T> SmsZ80IrqState for T
where
    T: Impl<SmsZ80IrqStateImpl> + ?Sized,
    T::Impler: SmsZ80IrqState,
{
    #[inline]
    fn already_accepted(&self) -> bool {
        self.make().already_accepted()
    }

    #[inline]
    fn set_already_accepted(&mut self, x: bool) {
        self.make_mut().set_already_accepted(x)
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
    T: SmsVdpIrq + SmsPlayerInput + SmsZ80IrqState + ?Sized,
{
    #[inline]
    fn requesting_mi(&self) -> Option<u8> {
        self.0._0().get()
    }

    #[inline]
    fn requesting_nmi(&mut self) -> bool {
        let z = self.0.mut_0();
        match (z.pause(), z.already_accepted()) {
            (true, false) => {
                z.set_already_accepted(true);
                return true;
            }
            (false, _) => {
                z.set_already_accepted(false);
            }
            _ => {}
        }
        return false;
    }
}
