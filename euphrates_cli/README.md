# Euphrates CLI: A Command Line Interface for the Euphrates Emulator

Euphrates is an emulator for the Sega Master System, Sega Game Gear, and
Sega SG-1000 Game consoles.

It's available as a library, in the `euphrates` crate. This crate,
`euphrates_cli`, provides a command line interface based on SDL2.

To run `euphrates_cli`, you'll need to install Rust and SDL2.

## Invoking euphrates_cli

To play most Sega Master System ROMs, execute from this directory:
```
cargo run --release -- rom --rom PATH_TO_ROM
```

To play Codemasters games like Fantastic Dizzy:
```
cargo run --release -- rom --rom PATH_TO_ROM --memory_map codemasters --tv pal
```

To play Game Gear games:
```
cargo run --release -- rom --rom PATH_TO_ROM --kind gg
```

To play SG-1000 games:
```
cargo run --release -- rom --rom PATH_TO_ROM --memory_map sg1000_2
```

Compilation will take a few minutes.

During gameplay, control player 1 with keys WASDFG. Control player 2 with keys
IJKL;'. Space is reset; P is pause (or, for the Game Gear, start).

## Saving and restoring

If you want to be able to save states and playback, you should use the
`--save_directory` option like this:

```
cargo run --release -- rom --rom PATH_TO_ROM --save_directory PATH_TO_DIRECTORY
```

During gameplay, press `x` to save state. Press `r` to start recording gameplay
and `R` to save recorded gameplay.

Resume from a saved state using
```
cargo run --release -- load --loadfile PATH_TO_SAVED_STATE
```

Run recorded gameplay using
```
cargo run --release -- loadrecord --loadfile PATH_TO_RECORDED_GAMEPLAY
```

To playback recorded gameplay at max speed and time the result:
```
cargo run --release -- playback --loadfile PATH_TO_RECORDED_GAMEPLAY --frequency unlimited
```

## Miscellaneous features

If you have an x86-64 processor with BMI2 instructions, you can get better
performance like this (this will require a recompile):

```
cargo run --release --features euphrates_x64` -- rom --rom PATH_TO_ROM
```

You can disable debugging features (see below) as follows:

```
cargo run --release -- rom --rom PATH_TO_ROM --debug false
```

This will provide a performance improvement (but one that is probably
unnecessary for gameplay on modern computers).

## Debugging

Euphrates provides some debugging features (as long as you didn't invoke
it with `--debug false`).

During gameplay, you can press:

* `n` to print disassembly around the current program counter;

* `N` to print a whole-program disassembly;

* `h` to hold execution;

* `y` to step through one Z80 instruction;

* `H` to resume execution;

* `z` to display the current status of the emulated Z80 CPU.

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
