mod instructions;
mod execute;
// mod generated_dispatch;

pub use self::execute::execute1;

pub use self::instructions::{nonmaskable_interrupt, maskable_interrupt};

#[cfg(test)]
mod tests;
