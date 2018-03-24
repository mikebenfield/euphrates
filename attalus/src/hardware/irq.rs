pub trait Irq {
    #[inline]
    fn requesting_mi(&self) -> Option<u8> {
        None
    }

    #[inline]
    fn requesting_nmi(&self) -> bool {
        false
    }

    /// The Z80 responds to nonmaskable interrupts due to the change in voltage
    /// in the NMI pin from high to low, so it will not continually execute
    /// interrupts when the voltage is held low. In software, that means we need
    /// to tell the device the interrupt is being executed and to stop requesting
    /// it.
    #[inline]
    fn clear_nmi(&mut self) {
    }
}
