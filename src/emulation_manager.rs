// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify this file
// under the terms of the GNU General Public License, version 3, as published by
// the Free Software Foundation. You should have received a copy of the GNU
// General Public License along with Attalus. If not, see
// <http://www.gnu.org/licenses/>.

// use ::sdl_wrap;
use ::log;
use ::hardware::z80::*;
use ::hardware::vdp::*;
use ::hardware::io::sms2::*;
use ::hardware::memory_map::*;

pub struct EmulationManager
{
    z80: Z80<Sms2Io>,
}

impl EmulationManager {
    pub fn new(smm: SegaMemoryMap) -> EmulationManager {
        let io = Sms2Io::new(smm);
        EmulationManager {
            z80: Z80::new(io),
        }
    }

    pub fn main_loop<S>(&mut self, screen: &mut S, n: usize)
    where
    S: Screen
    {
        for i in 0usize..n {
            log_major!("EM: loop {}", i);

            self.z80.io.vdp.draw_line(screen);

            let vdp_cycles = self.z80.io.vdp.cycles;
            let z80_target_cycles = 2 * vdp_cycles / 3;
            Z80Interpreter {}.run(&mut self.z80, z80_target_cycles);

            // if sdl_wrap::event::check_quit() {
            //     break;
            // }
        }

    }
}
