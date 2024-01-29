# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.1] - ...
### Added
- Audio resampling
- Added the test `blargg interrupt_time`
- Updated SFML to `0.21`

## [1.0.0] - 2022-05-27
### Added
- Ability to access `AudioBuffers` for each individual channel.
- Allowed to specify the SRAM save file and whether or not to save on shutdown.
- Documenting all public APIs and now its easier to use the library as backend emulation.

### Changed
- Changes in the PPU and passed some tests.
- Used `ciborium` instead of `serde_cbor` for SaveState. ([7091c3c])
- Moved to Rust edition 2021.

## [0.2.0] - 2021-02-18
### Added
- Save states with the [`save_state`](./save_state) library.
- Notifications UI in the front-end, used to display messages and errors.
- This `CHANGELOG` file.

### Changed
- Updated [SFML] to version `0.16.0`. (#2)

## [0.1.2] - 2021-02-18
### Added
- Added printer support.

### Fixed
- Show errors when saving/loading sram files for the cartridge. ([c6d446c])

## [0.1.1] - 2021-02-07
### Added
- The implementation of the gameboy (DMG) and gameboy color emulation.
- The implementation include the CPU, PPU, APU, and all components to produce
  a working emulator with high accuracy.
- The UI is built with [SFML].

### Fixed
- This is the first release and has **SO** many rewrites and bug fixes.

[Unreleased]: https://github.com/Amjad50/mizu/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/Amjad50/mizu/compare/v0.2.0...v1.0.0
[0.2.0]: https://github.com/Amjad50/mizu/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/Amjad50/mizu/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/Amjad50/mizu/compare/d3539ab...v0.1.1

[c6d446c]: https://github.com/Amjad50/mizu/commit/c6d446c 
[7091c3c]: https://github.com/Amjad50/mizu/commit/7091c3c

[SFML]: https://www.sfml-dev.org/
