# Mizu
[![Build status](https://github.com/Amjad50/mizu/workflows/Rust/badge.svg)](https://github.com/Amjad50/mizu/actions?query=workflow%3ARust)
[![codecov](https://codecov.io/gh/Amjad50/mizu/branch/master/graph/badge.svg)](https://codecov.io/gh/Amjad50/mizu)

Mizu is a Gameboy emulator built in Rust.

## progress
- [x] fully functional CPU (passes blargg `cpu_instr` test)
- [x] functional PPU (not time accurate)
- [x] functional Mbc1 mapper (still without saving battery cartridge)
- [x] Timer
- [ ] Serial (not sure if its important, since we don't have more than one GB, but maybe we can use the internet? An idea...)
- [x] Audio
- [x] Automatic testing, and with easy interface

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
