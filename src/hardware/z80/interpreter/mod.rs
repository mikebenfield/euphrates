mod instructions;
mod execute;

pub use self::execute::execute1;

pub use self::instructions::{nonmaskable_interrupt, maskable_interrupt};
