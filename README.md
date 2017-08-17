# Attalus: A Sega Master System Emulator

Attalus is an emulator for the Sega Master System. The goal is to produce an
emulator that is suitable for interactive use as well as for training AI
video game players.

Currently, I've written a software interpreter for the Z80 that works well,
having been tested against Udo Monk's `z80sim` emulator.

## Plan of attack

Here, roughly in chronological order, are the major tasks I'm planning on,
some of which I've already accomplished:

- [x] emulate the Z80 processor using a software interpreter;

- [x] emulate the standard Sega memory mapper with an index array; 

- [ ] emulate the Video Display Processor, drawing graphics with SDL;

- [ ] emulate controller input so that it's possible to play some games;

- [ ] emulate the SN76489 sound chip;

- [ ] implement VDP modes and other features necessary to also play Game Gear
games;

- [ ] emulate the CodeMasters memory mapper with an index array;

- [ ] emulate the YM2413 FM sound unit;

- [ ] implement some sort of graphical interface for settings and maybe ROM
selection;

- [ ] implement alternate emulations of the memory mappers using OS virtual
memory via `mmap`;

- [ ] implement an alternate emulation of the Z80 processor via binary
translation to x86-64 machine code (the Z80 and the x86-64 are, in a sense,
distant relatives; in particular, the condition flags are very similar);

- [ ] (maybe) emulate other game consoles or computers (the GameBoy and
GameBoy Color are likely targets, since they use a CPU related to the Z80).

## License

Attalus is Copyright 2017, Michael Benfield.

You may distribute and/or modify Attalus under the terms of the GNU General
Public License, version 3, as published by the Free Software Foundation.

This program is distributed in the hope that it will be useful, but WITHOUT
ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with
Attalus (see the file LICENSE). If not, see <http://www.gnu.org/licenses/>.
