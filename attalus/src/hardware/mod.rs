//! # Sega Master System hardware components
//!
//! For each component there is
//!
//! 1. A basic Hardware type.
//! 2. A trait for public use.
//!
//! For instance, in the [`z80`] module, there is a [`Z80Hardware`] struct containing
//! the values of the registers and internal flags of the emulated Z80 CPU. There
//! is also a [`Z80`] trait, which is what users of the [`z80`] module will use to
//! interface with [`z80`]. The [`Z80`] trait is a subtrait of the [`Log`],
//! [`Io`], and [`MemoryMapper`] traits because those are the traits the Z80 needs
//! to access to perform its functions.
//!
//! [`Z80Hardware`]: z80/struct.Z80Hardware.html
//! [`z80`]: z80/index.html
//! [`Z80`]: z80/trait.Z80.html
//! [`Log`]: ../log/trait.Log.html
//! [`Io`]: io/trait.Io.html
//! [`MemoryMapper`]: memory_mapper/trait.MemoryMapper.html

pub mod io16;
pub mod sn76489;
pub mod irq;
pub mod memory16;
pub mod sms_vdp;
pub mod z80;
