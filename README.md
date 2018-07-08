# Euphrates: A Sega Master System Emulator

Euphrates is an emulator for the Sega Master System. The goal is to produce an
emulator that is suitable for interactive use as well as for training AI
video game players.

## Status

The emulator works and plays lots of games. But some other games don't work,
including those from CodeMasters, and this is still a prerelease project.

If you just want to play a game, see the `euphrates_cli` subcrate.

## Future

In roughly chronological order, here are the tasks I'm planning on:

- [ ] get the emulate CodeMasters memory mapper working;

- [ ] emulate the YM2413 FM sound unit;

- [ ] get a game controller working;

- [ ] get Game Gear emulation working;

- [ ] implement alternate emulations of the memory mappers using OS virtual
memory via `mmap`;

- [ ] implement an alternate emulation of the Z80 processor via binary
translation to x86-64 machine code (the Z80 and the x86-64 are, in a sense,
distant relatives; in particular, the condition flags are very similar);

- [ ] (maybe) emulate other game consoles or computers (the GameBoy and
GameBoy Color are likely targets, since they use a CPU related to the Z80).

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
