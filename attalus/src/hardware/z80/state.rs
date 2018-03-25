use super::InterruptMode;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[repr(C)]
pub struct T {
    pub cycles: u64,
    pub registers: [u16; 13],
    pub halted: bool,
    pub iff1: bool,
    pub iff2: bool,
    pub interrupt_mode: InterruptMode,
}

pub trait Savable {
    fn save(&self) -> T;
}

pub trait Restorable {
    fn restore(&T) -> Self;
}

impl Savable for T {
    fn save(&self) -> T {
        *self
    }
}

impl Restorable for T {
    fn restore(t: &T) -> Self {
        *t
    }
}
