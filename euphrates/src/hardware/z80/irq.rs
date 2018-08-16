pub trait Z80Irq {
    fn requesting_mi(&mut self) -> Option<u8>;
    fn requesting_nmi(&mut self) -> bool;
    fn take_nmi(&mut self);
}
