//! An implementation of `Z80Irq` for the Sega Master System.

use impler::{ConstOrMut, Impler, ImplerImpl};

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

pub trait SmsZ80IrqStateImpl {
    type Impler: SmsZ80IrqState + ?Sized;

    fn close<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Self::Impler) -> T;

    fn close_mut<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self::Impler) -> T;
}

impl<T> SmsZ80IrqState for T
where
    T: SmsZ80IrqStateImpl + ?Sized,
{
    #[inline]
    fn already_accepted(&self) -> bool {
        self.close(|z| z.already_accepted())
    }

    #[inline]
    fn set_already_accepted(&mut self, x: bool) {
        self.close_mut(|z| z.set_already_accepted(x))
    }
}

/// An Impler for Z80Irq.
///
/// `T` must implement `SmsVdpIrq`, `SmsPlayerInput`, and `SmsZ80IrqState`.
pub struct SmsZ80IrqImpler<T: ?Sized>(ConstOrMut<T>);

unsafe impl<T: ?Sized> ImplerImpl for SmsZ80IrqImpler<T> {
    type T = T;

    #[inline]
    unsafe fn new(c: ConstOrMut<Self::T>) -> Self {
        SmsZ80IrqImpler(c)
    }

    #[inline]
    fn get(&self) -> &ConstOrMut<Self::T> {
        &self.0
    }

    #[inline]
    fn get_mut(&mut self) -> &mut ConstOrMut<Self::T> {
        &mut self.0
    }
}

impl<T> Z80Irq for SmsZ80IrqImpler<T>
where
    T: SmsVdpIrq + SmsPlayerInput + SmsZ80IrqState + ?Sized,
{
    #[inline]
    fn requesting_mi(&self) -> Option<u8> {
        self._0().get()
    }

    #[inline]
    fn requesting_nmi(&mut self) -> bool {
        let z = self.mut_0();
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
