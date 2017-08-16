
pub trait Irq {
    fn requesting_mi(&self) -> bool;
    fn requesting_nmi(&self) -> bool;
}
