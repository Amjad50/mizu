mod apu;
mod cartridge;
mod cpu;
mod joypad;
mod memory;
mod ppu;
mod printer;
mod save_error;
mod serial;
mod timer;

#[cfg(test)]
mod tests;

use std::cell::RefCell;
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use save_state::Savable;

use cartridge::Cartridge;
use cpu::Cpu;
use memory::Bus;

pub use apu::AudioBuffers;
pub use cartridge::CartridgeError;
pub use joypad::JoypadButton;
pub use printer::Printer;
pub use save_error::SaveError;
pub use serial::SerialDevice;

/// The current version of state saved/loaded by
/// [`GameBoy::save_state`] / [`GameBoy::load_state`].
///
/// Loading a state that is not compatible with this version, results
/// in [`SaveError::UnmatchedSaveErrorVersion`]
pub const SAVE_STATE_VERSION: usize = 2;
const SAVE_STATE_MAGIC: &[u8; 4] = b"MST\xee";
const SAVE_STATE_ZSTD_COMPRESSION_LEVEL: i32 = 0; // default compression

/// Custom configuration for the [`GameBoy`] emulation inner workings
#[derive(Debug, Default, Clone, Copy, Savable)]
pub struct GameBoyConfig {
    /// Should the gameboy run in DMG mode? default is in CGB mode
    pub is_dmg: bool,
}

impl GameBoyConfig {
    pub fn boot_rom_len(&self) -> usize {
        if self.is_dmg {
            0x100
        } else {
            0x900
        }
    }
}

/// Builder struct container for [`GameBoy`] configurations and options.
pub struct GameBoyBuilder {
    config: GameBoyConfig,
    rom_file: PathBuf,
    boot_rom_file: Option<PathBuf>,
    sram_file: Option<PathBuf>,
    save_on_shutdown: bool,
}

impl GameBoyBuilder {
    /// Add custom [`GameBoyConfig`]
    pub fn config(mut self, config: GameBoyConfig) -> Self {
        self.config = config;
        self
    }

    /// Add boot rom file
    pub fn boot_rom_file<P: AsRef<Path>>(mut self, boot_rom_file: P) -> Self {
        self.boot_rom_file = Some(boot_rom_file.as_ref().to_path_buf());
        self
    }

    /// Add custom sram file,
    /// if this is not specified, the sram will be stored in the same directory
    /// as the rom file.
    pub fn sram_file<P: AsRef<Path>>(mut self, save_file: P) -> Self {
        self.sram_file = Some(save_file.as_ref().to_path_buf());
        self
    }

    /// Should the SRAM be saved on shutdown? (default: true)
    pub fn save_on_shutdown(mut self, save_on_shutdown: bool) -> Self {
        self.save_on_shutdown = save_on_shutdown;
        self
    }

    /// Builds a [`GameBoy`] instance.
    pub fn build(self) -> Result<GameBoy, CartridgeError> {
        GameBoy::build(self)
    }
}

/// The GameBoy is the main interface to the emulator.
///
/// Everything regarding emulation can be controlled from here.
pub struct GameBoy {
    cpu: Cpu,
    bus: Bus,
}

impl GameBoy {
    /// Initiate a builder object with a cartridge file.
    pub fn builder<RomP: AsRef<Path>>(rom_file: RomP) -> GameBoyBuilder {
        GameBoyBuilder {
            config: GameBoyConfig::default(),
            rom_file: rom_file.as_ref().to_path_buf(),
            boot_rom_file: None,
            sram_file: None,
            save_on_shutdown: true,
        }
    }

    fn build(builder: GameBoyBuilder) -> Result<Self, CartridgeError> {
        let file_path = builder.rom_file;
        let sram_file_path = builder.sram_file;
        let boot_rom_file_path = builder.boot_rom_file;
        let config = builder.config;
        let save_on_shutdown = builder.save_on_shutdown;

        let cartridge = Cartridge::from_file(file_path, sram_file_path, save_on_shutdown)?;

        let (bus, cpu) = if let Some(boot_rom_file) = boot_rom_file_path {
            let mut boot_rom_file = File::open(boot_rom_file)?;
            let mut data = vec![0; config.boot_rom_len()];

            // make sure the boot_rom is the exact same size
            assert_eq!(
                boot_rom_file.metadata()?.len(),
                data.len() as u64,
                "boot_rom file size is not correct"
            );

            boot_rom_file.read_exact(&mut data)?;

            (
                Bus::new_with_boot_rom(cartridge, data, config),
                Cpu::new(config),
            )
        } else {
            let is_cartridge_color = cartridge.is_cartridge_color();
            (
                Bus::new_without_boot_rom(cartridge, config),
                Cpu::new_without_boot_rom(config, is_cartridge_color),
            )
        };

        Ok(Self { bus, cpu })
    }

    /// Clocks the Gameboy clock for the duration of one PPU frame.
    ///
    /// This is good for timing emulation, you can call this function once
    /// and then render it.
    pub fn clock_for_frame(&mut self) {
        const PPU_CYCLES_PER_FRAME: u32 = 456 * 154;
        let mut cycles = 0u32;
        while cycles < PPU_CYCLES_PER_FRAME {
            self.cpu.next_instruction(&mut self.bus);
            cycles += self.bus.elapsed_ppu_cycles();
        }
    }

    /// Return the game title string extracted from the cartridge.
    pub fn game_title(&self) -> &str {
        self.bus.cartridge().game_title()
    }

    /// The cartridge file path.
    pub fn file_path(&self) -> &Path {
        self.bus.cartridge().file_path()
    }

    /// Return the pixels buffer of the PPU at the current state.
    ///
    /// The format of the pixel buffer is RGB, i.e. 3 bytes per pixel.
    pub fn screen_buffer(&self) -> &[u8] {
        self.bus.screen_buffer()
    }

    /// Return the audio buffer of the APU at the current state.
    ///
    /// We use `&mut` as it will also reset the buffers after using them
    pub fn audio_buffers(&mut self) -> AudioBuffers {
        self.bus.audio_buffers()
    }

    /// Change the state of the joypad button to `pressed`.
    pub fn press_joypad(&mut self, button: JoypadButton) {
        self.bus.press_joypad(button);
    }

    /// Change the state of the joypad button to `released`.
    pub fn release_joypad(&mut self, button: JoypadButton) {
        self.bus.release_joypad(button);
    }

    // TODO: Not sure if using RefCell is the best option here
    /// Connect a serial device to the Gameboy.
    ///
    /// Currently the gameboy can only be `master`, so the other device
    /// must be implemented as `slave`.
    pub fn connect_device(&mut self, device: Rc<RefCell<dyn SerialDevice>>) {
        self.bus.connect_device(device);
    }

    /// Disconnects the serial device if any is connected, else, nothing is done
    pub fn disconnect_device(&mut self) {
        self.bus.disconnect_device();
    }

    /// Saves the whole current state of the emulator.
    pub fn save_state<W: Write>(&self, mut writer: W) -> Result<(), SaveError> {
        SAVE_STATE_MAGIC.save(&mut writer)?;
        SAVE_STATE_VERSION.save(&mut writer)?;
        let cartridge_hash: &[u8; 32] = self.bus.cartridge().hash();
        cartridge_hash.save(&mut writer)?;

        let mut writer = zstd::Encoder::new(&mut writer, SAVE_STATE_ZSTD_COMPRESSION_LEVEL)?;

        self.cpu.save(&mut writer)?;
        self.bus.save(&mut writer)?;

        let _writer = writer.finish()?;

        Ok(())
    }

    /// Loads the whole state of the emulator, if an error happened in the middle
    /// the emulator will keep functioning like normal, as it stores a backup recovery state before
    /// loading the new state.
    pub fn load_state<R: Read + Seek>(&mut self, mut reader: R) -> Result<(), SaveError> {
        // save state, so that if an error occured we will restore it back.
        let mut recovery_save_state = Vec::new();
        self.cpu
            .save(&mut recovery_save_state)
            .expect("recovery save cpu");
        self.bus
            .save(&mut recovery_save_state)
            .expect("recovery save bus");

        let mut load_routine = || {
            let mut magic = [0u8; 4];
            let mut version = 0usize;
            let mut hash = [0u8; 32];

            magic.load(&mut reader)?;
            if &magic != SAVE_STATE_MAGIC {
                return Err(SaveError::InvalidSaveStateHeader);
            }

            // since there might be some possibility to migrate from different
            // versions, we will not check here.
            version.load(&mut reader)?;

            hash.load(&mut reader)?;
            if &hash != self.bus.cartridge().hash() {
                return Err(SaveError::InvalidCartridgeHash);
            }

            {
                // use a box on read because there are two types of readers
                // that we might use, compressed or not compressed based on the version
                // of the save_state file
                let mut second_stage_reader: Box<dyn Read>;

                if version == 1 && SAVE_STATE_VERSION == 2 {
                    // no need to use compression
                    second_stage_reader = Box::new(&mut reader);
                } else if version != SAVE_STATE_VERSION {
                    return Err(SaveError::UnmatchedSaveErrorVersion(version));
                } else {
                    second_stage_reader = Box::new(zstd::Decoder::new(&mut reader)?);
                }

                self.cpu.load(&mut second_stage_reader)?;
                self.bus.load(&mut second_stage_reader)?;
            }

            // make sure there is no more data
            let stream_current_pos = reader.stream_position()?;
            reader.seek(SeekFrom::End(0))?;
            let stream_last_pos = reader.stream_position()?;

            let (remaining_data_len, overflow) =
                stream_last_pos.overflowing_sub(stream_current_pos);
            assert!(!overflow);

            if remaining_data_len > 0 {
                // return seek
                reader.seek(SeekFrom::Start(stream_current_pos))?;

                Err(SaveError::SaveStateError(save_state::Error::TrailingData(
                    remaining_data_len,
                )))
            } else {
                Ok(())
            }
        };

        if let Err(err) = load_routine() {
            let mut cursor = Cursor::new(&recovery_save_state);

            self.cpu.load(&mut cursor).expect("recovery load cpu");
            self.bus.load(&mut cursor).expect("recovery load bus");

            Err(err)
        } else {
            Ok(())
        }
    }
}
