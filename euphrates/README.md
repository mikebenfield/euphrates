# Euphrates: A Game Console Emulator

Euphrates is an emulator for the Sega Master System, Sega Game Gear, and
Sega SG-1000 Game consoles.

This crate, `euphrates`, is the main emulation library. It features emulation
for the Z80 processor and other hardware components including the Sega Master
System VDP. It also has a disassembler and debugger.

Euphrates is also available as a command line program; see the `euphrates_cli`
crate.

Euphrates is intended to be a platform for exploring artificial intelligence and
reinforcement learning, but there are a few features that need to be added.

## Future

The goal is to make Euphrates very fast, so that AI search methods may run
extensive simulations as quickly as possible.

To that end, the most significant feature I will be adding soon is

- [ ] dynamic recompilation,

and some time after that I will add

- [ ] GPU emulation of the video display processor.

Dynamic recompilation will be a fairly large undertaking. In the meantime,
some relatively straightforward features I will probably add are

- [ ] Colecovision emulation;

- [ ] sound emulation via sine waves instead of square wves.

Some bigger non-priority features I may add someday include

- [ ] MSX emulation;

- [ ] MSX2 emulation;

- [ ] ZX Spectrum emulation;

- [ ] Gameboy emulation.

Finally, it's conceivable but not particularly likely that I will add
emulation for systems based on processors other than the Z80.

## Bugs

- Two VDP graphical modes are not implemented. These are not used by any
official Master System, Game Gear, or SG-1000 game, but I'll need to implement
them for the Colecovision.

## License

Euphrates is Copyright 2018, Michael Benfield.

Euphrates is free software: you can redistribute it and/or modify it under the
terms of the GNU General Public License as published by the Free Software
Foundation, either version 3 of the License, or (at your option) any later
version.

Euphrates is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with
Euphrates (see the file LICENSE). If not, see <https://www.gnu.org/licenses/>.
