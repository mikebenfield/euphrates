# Attalus: A Sega Master System Emulator

Attalus is an emulator for the Sega Master System. The goal is to produce an
emulator that is suitable for interactive use as well as for training AI
video game players.

## Status

The emulator works and plays lots of games. But some other games don't work,
including those from CodeMasters, and this is still a prerelease project.

If you just want to play a game; see the `attalus_cli` subcrate.

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

Attalus is Copyright 2017, Michael Benfield.

You may copy, modify, and/or distribute Attalus under either the terms of the
Apache License, version 2 (see the file LICENSE-APACHE or
<http://www.apache.org/licenses/LICENSE-2.0>) or the MIT license (see the file
LICENSE-MIT or <http://opensource.org/licenses/MIT>), at your option.
