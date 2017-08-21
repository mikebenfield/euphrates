// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either version 3 of the license or (at your option) any later
// version. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

use std;

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

    pub fn main_loop<S>(
        &mut self,
        screen: &mut S,
        palette_screen: &mut S,
        sprite_screen: &mut S,
        tile_screen: &mut S,
        n: usize
    ) -> Result<(), ScreenError>
    where
        S: Screen
    {
        use sdl_wrap;

        let mut milliseconds = 0u64;

        for i in 0usize..n {
            log_major!("EM: loop {}", i);

            self.z80.io.vdp.draw_line(screen)?;

            self.z80.io.vdp.draw_palettes(palette_screen)?;

            self.z80.io.vdp.draw_sprites(sprite_screen)?;

            self.z80.io.vdp.draw_tiles(tile_screen)?;

            let vdp_cycles = self.z80.io.vdp.cycles;
            let z80_target_cycles = 2 * vdp_cycles / 3;
            Z80Interpreter {}.run(&mut self.z80, z80_target_cycles);

            if sdl_wrap::event::check_quit() {
                break;
            }

            if self.z80.io.vdp.read_v() == 0 {
                let new_milliseconds = self.z80.cycles / 5000;
                let diff = new_milliseconds - milliseconds;
                let duration = std::time::Duration::from_millis(diff);
                std::thread::sleep(duration);
                milliseconds = new_milliseconds;
            }
        }
        Ok(())
    }
}
