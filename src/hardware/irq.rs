
/// Trait through which hardware may request an interrupt from the Z80.
/// Implementations may 
pub trait Irq {
	fn request_maskable_interrupt(&mut self) -> bool;

    fn request_nonmaskable_interrupt(&mut self);
}
