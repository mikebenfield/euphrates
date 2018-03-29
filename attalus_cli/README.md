# Attalus CLI: A Command Line Interface for the Attalus Emulator

To play a Sega Master System ROM:
```
cargo run --release -- rom --rom PATH_TO_ROM
```

If you want to be able to save states and playback:

```
cargo run --release -- rom --rom PATH_TO_ROM --savedirectory PATH_TO_DIRECTORY
```

Gameplay: Keys WASD are directions. F and G are up/down. Space is reset. Z saves
your state, in whatever directory you've specified. R begins recording gameplay,
and Shift+R stops recording and saves the recorded gameplay.

To restore state:

```
cargo run --release -- load --loadfile PATH_TO_SAVED_STATE
```

To play back recorded gameplay:

```
cargo run --release -- playback --loadfile PATH_TO_SAVED_GAMEPLAY
```

Note that some games don't work yet. Note also that there is some assembly code,
in the attalus subcrate `attalus_x64`. As currently set up, on the x86-64
platform there's a hard requirement on clang to act as the assembler. Someday
I'll make the dependency on attalus_x64 optional.

## License

Attalus CLI is Copyright 2018, Michael Benfield.

You may copy, modify, and/or distribute Attalus CLI under either the terms of
the Apache License, version 2 (see the file LICENSE-APACHE or
<http://www.apache.org/licenses/LICENSE-2.0>) or the MIT license (see the file
LICENSE-MIT or <http://opensource.org/licenses/MIT>), at your option.
