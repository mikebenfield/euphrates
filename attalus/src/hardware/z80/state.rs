use super::InterruptMode;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[repr(C)]
pub struct Z80State {
    pub cycles: u64,
    pub registers: [u16; 13],
    pub halted: bool,
    pub iff1: bool,
    pub iff2: bool,
    pub interrupt_mode: InterruptMode,
}

pub trait Savable {
    fn save(&self) -> Z80State;
}

pub trait Restorable {
    fn restore(&Z80State) -> Self;
}

impl Savable for Z80State {
    fn save(&self) -> Z80State {
        *self
    }
}

impl Restorable for Z80State {
    fn restore(t: &Z80State) -> Self {
        *t
    }
}
