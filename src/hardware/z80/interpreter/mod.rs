// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

mod instructions;
mod run;

use ::message::{Receiver, Sender};
use super::{Z80, Z80Emulator, Z80Message};
use self::run::run;
use ::hardware::io::Io;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct Z80Interpreter;

impl<I, R> Z80Emulator<I, R> for Z80Interpreter
where
    I: Io<R>,
    R: Receiver<Z80Message> + Receiver<<I::MemoryMap as Sender>::Message>,
{
    fn run(&mut self, receiver: &mut R, z: &mut Z80<I>, cycles: u64)
    {
        run(receiver, z, cycles);
    }
}
