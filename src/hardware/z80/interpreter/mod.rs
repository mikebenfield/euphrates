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
