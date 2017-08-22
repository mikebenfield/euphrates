// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

mod instructions;
mod run;

use super::{Z80, Z80Emulator};
use self::run::run;
use ::hardware::io::Io;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct Z80Interpreter;

impl<I: Io> Z80Emulator<I> for Z80Interpreter {
    fn run(&mut self, z: &mut Z80<I>, cycles: u64) {
        run(z, cycles);
    }
}
