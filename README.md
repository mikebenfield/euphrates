# Euphrates: A Sega Master System Emulator

Euphrates is an emulator for the Sega Master System, Sega Game Gear, and
Sega SG-1000 Game consoles.

This repository consists of these crates:

- `euphrates`, the main emulation library;

- `euphrates_sdl`, providing audio and video types for use with Euphrates;

- `euphrates_x64`, a small library that can make Euphrates a bit more
performant on x86-64 systems with BMI2 instructions;

- `euphrates_cli`, an application to run Euphrates from the command line
using `euphrates_sdl` for audio and video.

If you just want to play some games, see the `euphrates_cli` crate. All
officially released games that I've tested work. However, note that an excellent
end user experience for playing games is not currently a high priority for
Euphrates (for instance, you'll have to play with your keyboard, and you'll have
to specify command line options for which memory map you want to use). Instead,
Euphrates is intended to (eventually) be a platform for exploring artificial
intelligence techniques.

See the README in the `euphrates` crate for more on Euphrates and its goals.

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
