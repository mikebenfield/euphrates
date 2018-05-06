//! The status of player input on the Sega Master System.

/// Bit flags for Joypad Port A.
///
/// Note that a button press is indicated by a flag *not* being set.
pub mod joypad_a_bits {
    pub const JOYPAD2_DOWN: u8 = 0b10000000;
    pub const JOYPAD2_UP: u8 = 0b01000000;
    pub const JOYPAD1_B: u8 = 0b00100000;
    pub const JOYPAD1_A: u8 = 0b00010000;
    pub const JOYPAD1_RIGHT: u8 = 0b00001000;
    pub const JOYPAD1_LEFT: u8 = 0b00000100;
    pub const JOYPAD1_DOWN: u8 = 0b00000010;
    pub const JOYPAD1_UP: u8 = 0b00000001;
}

/// Bit flags for Joypad Port B.
///
/// Note that a button press is indicated by a flag *not* being set.
pub mod joypad_b_bits {
    pub const B_TH: u8 = 0b10000000;
    pub const A_TH: u8 = 0b01000000;
    pub const CONT: u8 = 0b00100000;
    pub const RESET: u8 = 0b00010000;
    pub const JOYPAD2_B: u8 = 0b00001000;
    pub const JOYPAD2_A: u8 = 0b00000100;
    pub const JOYPAD2_RIGHT: u8 = 0b00000010;
    pub const JOYPAD2_LEFT: u8 = 0b00000001;
}

/// What buttons are being pressed this frame?
///
/// Since button presses are indicated by flags *not* being set,
/// `Default::default()` will set `joypad_a` and `joypad_b` to `0xFF`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SmsPlayerInputState {
    /// Joypad Port A (which doesn't completely correspond to Joypad 1; see
    /// `joypad_a_bits`).
    pub joypad_a: u8,
    /// Joypad Port B (which doesn't completely correspond to Joypad 2; see
    /// `joypad_b_bits`).
    pub joypad_b: u8,

    /// The pause button.
    pub pause: bool,
}

impl Default for SmsPlayerInputState {
    #[inline]
    fn default() -> Self {
        SmsPlayerInputState {
            joypad_a: 0xFF,
            joypad_b: 0xFF,
            pause: false,
        }
    }
}

impl SmsPlayerInput for SmsPlayerInputState {
    #[inline]
    fn joypad_a(&self) -> u8 {
        self.joypad_a
    }

    #[inline]
    fn set_joypad_a(&mut self, x: u8) {
        self.joypad_a = x
    }

    #[inline]
    fn joypad_b(&self) -> u8 {
        self.joypad_b
    }

    #[inline]
    fn set_joypad_b(&mut self, x: u8) {
        self.joypad_b = x
    }

    #[inline]
    fn pause(&self) -> bool {
        self.pause
    }

    #[inline]
    fn set_pause(&mut self, x: bool) {
        self.pause = x
    }

    #[inline]
    fn state(&self) -> SmsPlayerInputState {
        *self
    }

    #[inline]
    fn set_state(&mut self, x: SmsPlayerInputState) {
        *self = x
    }
}

pub trait SmsPlayerInput {
    fn joypad_a(&self) -> u8;
    fn set_joypad_a(&mut self, x: u8);
    fn joypad_b(&self) -> u8;
    fn set_joypad_b(&mut self, x: u8);
    fn pause(&self) -> bool;
    fn set_pause(&mut self, x: bool);
    fn state(&self) -> SmsPlayerInputState;
    fn set_state(&mut self, x: SmsPlayerInputState);
}

pub trait SmsPlayerInputImpl {
    type Impler: SmsPlayerInput;

    fn close<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Self::Impler) -> T;

    fn close_mut<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self::Impler) -> T;
}

impl<T> SmsPlayerInput for T
where
    T: SmsPlayerInputImpl,
{
    #[inline]
    fn joypad_a(&self) -> u8 {
        self.close(|z| z.joypad_a())
    }

    #[inline]
    fn set_joypad_a(&mut self, x: u8) {
        self.close_mut(|z| z.set_joypad_a(x))
    }

    #[inline]
    fn joypad_b(&self) -> u8 {
        self.close(|z| z.joypad_b())
    }

    #[inline]
    fn set_joypad_b(&mut self, x: u8) {
        self.close_mut(|z| z.set_joypad_b(x))
    }

    #[inline]
    fn pause(&self) -> bool {
        self.close(|z| z.pause())
    }

    #[inline]
    fn set_pause(&mut self, x: bool) {
        self.close_mut(|z| z.set_pause(x))
    }

    #[inline]
    fn state(&self) -> SmsPlayerInputState {
        self.close(|z| z.state())
    }

    #[inline]
    fn set_state(&mut self, x: SmsPlayerInputState) {
        self.close_mut(|z| z.set_state(x))
    }
}
