// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

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

pub mod io;
pub mod irq;
pub mod memory_map;
pub mod vdp;
pub mod z80;
