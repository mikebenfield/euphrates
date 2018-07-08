# Euphrates CLI: A Command Line Interface for the Euphrates Emulator

Make sure SDL is installed.

To play a Sega Master System ROM:
```
cargo run --release -- rom --rom PATH_TO_ROM
```

If you want to be able to save states and playback:

```
cargo run --release -- rom --rom PATH_TO_ROM --savedirectory PATH_TO_DIRECTORY
```

Gameplay: Keys WASD are directions. F and G are controller buttons.. Space is
reset. Z saves your state, in whatever directory you've specified. R begins
recording gameplay, and Shift+R stops recording and saves the recorded gameplay.

To restore state:

```
cargo run --release -- load --loadfile PATH_TO_SAVED_STATE
```

To play back recorded gameplay:

```
cargo run --release -- playback --loadfile PATH_TO_SAVED_GAMEPLAY
```

Note that some games don't work yet.

If you're on x86-64 and you have `clang` (used for its an assembler), you can
get about 10% better performance by specifying the extra option
```
--features euphrates_x64
```
with `cargo`.

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
