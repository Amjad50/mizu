# Mizu
[![Build status](https://github.com/Amjad50/mizu/workflows/Rust/badge.svg)](https://github.com/Amjad50/mizu/actions?query=workflow%3ARust)
[![codecov](https://codecov.io/gh/Amjad50/mizu/branch/master/graph/badge.svg)](https://codecov.io/gh/Amjad50/mizu)
[![crate.io](https://img.shields.io/crates/v/mizu)](https://crates.io/crates/mizu)

Mizu is an accurate Gameboy emulator built in Rust.


## Features
- Emulating The original gameboy (DMG) and gameboy color hardware.
- Passing most hardware tests (see [TESTING.md](./TESTING.md)).
- Bettery save support.
- Accurate RTC emulation for MBC3 mapper.
- Accurate APU emulation with 48KHz audio.
- SFML gui front-end.
- Robust testing framework for continous testing.
- Easily change emulation speed.
- Functional mappers:
    - NoMapper
    - MBC1
    - MBC2
    - MBC3
    - MBC5
- Printer emulation

# Controls
The SFML front-end provide these keyboard bindings:

## Gameboy

| Key | Gameboy |
| --- | ------- |
| J   | B       |
| K   | A       |
| U   | Select  |
| I   | Start   |
| W   | Up      |
| S   | Down    |
| A   | Left    |
| D   | Right   |

## Extra

| Key   | Function              |
| ----- | --------------------- |
| Enter | A+B+Select+Start\*    |
| +     | Increase 5 to FPS\*\* |
| -     | Recude 5 from FPS\*\* |
| P     | Open Printer          |

> \* I made this because in `Zelda: Link's awakening` you need to press
> all of these buttons on the same frame to bring the save menu, which is annoying.

> \*\* FPS here is not the same with normal games FPS, where low FPS just makes the game
> laggy, here FPS control the emulation speed. Normally it will run on `60` FPS.
> If the user set FPS to `30` it will emulate in half the speed, this include audio,
> and CPU emulation.

## Printer window keys

| Key   | Function                   |
| ----- | -------------------------- |
| C     | Clear current image buffer |
| S     | Save image buffer to file  |

# Printer
Gameboy Printer is a serial device that can be connected to the gameboy and
used by some cartridges to print images. Popular cartidges that uses it are:
- Gameboy Camera.
- Zelda: Link's Awakening DX (to print images from the album).
- Pokemon (several versions) (to print pokemon info from the Pokedex).

The printer can be opened by pressing the `P` key.

The printer emulation allows to save the printed images into disk. The window
will only show `160x144` pixels, but the image is scrollable.

# Building and Installation
For installing or building `mizu` we would use `cargo`.

## Building
If you want to use the development version of `mizu` you can build the project
yourself. After downloading/cloning the repository, it can be build with:
```
$ cargo build
```
With release option:
```
$ cargo build --release
```

## Installing
If you want to use the latest stable version from `crates.io`:
```
$ cargo install mizu
```


# Yet another gameboy emulator?
Why not?. it is fun and educational, but even though I'm planning to make it as accurate as I can. If you want to see cool emulators, check my previous work [Plastic].

# References
### General Gameboy
- [Pandocs (of course)](https://gbdev.io/pandocs/)
- [Gameboy manual](http://www.codeslinger.co.uk/pages/projects/gameboy/files/GB.pdf)
- [GameBoy complete techincal reference](https://gekkio.fi/files/gb-docs/gbctr.pdf)
- [The cycle accurate gameboy docs](https://github.com/AntonioND/giibiiadvance/blob/master/docs/TCAGBD.pdf)
- [Mooneye accurate emulator, includes great tests](https://github.com/Gekkio/mooneye-gb)
### CPU instructions
- [Opcode table](https://gbdev.io/gb-opcodes//optables/dark)
- [Simple opcodes explaination](http://gameboy.mongenel.com/dmg/opcodes.html)
### Debugging and testing
- [blargg gameboy tests](https://gbdev.gg8.se/files/roms/blargg-gb-tests/)
- [BGB emulator](https://bgb.bircd.org/)
- [GB test roms](https://github.com/retrio/gb-test-roms)
- [DMG ACID PPU test](https://github.com/mattcurrie/dmg-acid2)
- [Mealybug PPU stress tear tests](https://github.com/mattcurrie/mealybug-tearoom-tests)


[Plastic]: https://github.com/Amjad50/plastic
