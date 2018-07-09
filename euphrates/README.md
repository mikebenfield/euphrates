# Euphrates: A Sega Master System Emulator

Euphrates is an emulator for the Sega Master System, Sega Game Gear, and
Sega SG-1000 Game consoles.

This crate, `euphrates`, is the main emulation library. It features emulation
for the Z80 processor and other hardware components including the Sega Master
System VDP. It also has a disassembler and infrastructure for other debugging
tools.

Euphrates is also available as a command line program; see the `euphrates_cli`
crate.

Euphrates is intended to be a platform for exploring artificial intelligence and
reinforcement learning.

## Future

The goal is to make Euphrates very fast, so that AI search methods may run
extensive simulations as quickly as possible.

To that end, the two most significant features I want to add to Euphrates
are

- [ ] dynamic recompilation;

- [ ] using native virtual memory to emulate memory banking.

Dynamic recompilation will be a pretty large undertaking. In the meantime,
some relatively straightforward features I will probably add are

- [ ] Colecovision emulation;

- [ ] An instruction-stepping debugger (this existed previously, but
I removed it).

Some bigger non-priority features I may add someday include

- [ ] MSX emulation;

- [ ] MSX2 emulation;

- [ ] ZX Spectrum emulation;

- [ ] Gameboy emulation.

Finally, it's conceivable but not particularly likely that I will add
emulation for systems based on processors other than the Z80.

## Bugs

- Compilation is obscenely slow. This may not be clear when compiling the
`euphrates` library on its own, but it will become apparent when, for instance,
compiling `euphrates_cli`. This is largely due to extensive use of type
parameters.

- There are a few graphical glitches on a few games.

- If you load a saved state, sound may be glitchy for a few seconds.

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
