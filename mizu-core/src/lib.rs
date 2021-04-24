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
use std::io::{Read, Write};
use std::path::Path;
use std::rc::Rc;

use save_state::Savable;

pub use joypad::JoypadButton;
pub use printer::Printer;
pub use save_error::SaveError;

use cartridge::{Cartridge, CartridgeError};
use cpu::Cpu;
use memory::Bus;
use serial::SerialDevice;

const SAVE_STATE_MAGIC: &[u8; 4] = b"MST\xee";
const SAVE_STATE_VERSION: usize = 1;

#[derive(Debug, Default, Clone, Copy, Savable)]
pub struct GameboyConfig {
    /// Should the gameboy run in DMG mode? default is in CGB mode
    pub is_dmg: bool,
}

impl GameboyConfig {
    pub fn boot_rom_len(&self) -> usize {
        if self.is_dmg {
            0x100
        } else {
            0x900
        }
    }
}

pub struct GameBoy {
    cpu: Cpu,
    bus: Bus,
}

impl GameBoy {
    pub fn new<P: AsRef<Path>>(
        file_path: P,
        boot_rom_file: Option<P>,
        config: GameboyConfig,
    ) -> Result<Self, CartridgeError> {
        let cartridge = Cartridge::from_file(file_path)?;

        let (bus, cpu) = if let Some(boot_rom_file) = boot_rom_file {
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

    /// Synced to PPU
    ///
    /// Not sure if this is an accurate apporach, but it looks good, as the
    /// number of PPU cycles per frame is fixed, counting for the number
    /// of ppu cycles is better than waiting for Vblank, as if the lcd
    /// is off, Vblank is not coming
    pub fn clock_for_frame(&mut self) {
        const PPU_CYCLES_PER_FRAME: u32 = 456 * 154;
        let mut cycles = 0u32;
        while cycles < PPU_CYCLES_PER_FRAME {
            self.cpu.next_instruction(&mut self.bus);
            cycles += self.bus.elapsed_ppu_cycles() as u32;
        }
    }

    pub fn game_title(&self) -> &str {
        &self.bus.cartridge().game_title()
    }

    pub fn file_path(&self) -> &Path {
        &self.bus.cartridge().file_path()
    }

    pub fn screen_buffer(&self) -> &[u8] {
        self.bus.screen_buffer()
    }

    pub fn audio_buffer(&mut self) -> Vec<f32> {
        self.bus.audio_buffer()
    }

    pub fn press_joypad(&mut self, button: JoypadButton) {
        self.bus.press_joypad(button);
    }

    pub fn release_joypad(&mut self, button: JoypadButton) {
        self.bus.release_joypad(button);
    }

    // TODO: Not sure if using RefCell is the best option here
    pub fn connect_device(&mut self, device: Rc<RefCell<dyn SerialDevice>>) {
        self.bus.connect_device(device);
    }

    /// Disconnects the serial device if any is connected, else, nothing is done
    pub fn disconnect_device(&mut self) {
        self.bus.disconnect_device();
    }

    pub fn save_state<W: Write>(&self, mut writer: W) -> Result<(), SaveError> {
        SAVE_STATE_MAGIC.save(&mut writer)?;
        SAVE_STATE_VERSION.save(&mut writer)?;
        let cartridge_hash: &[u8; 32] = self.bus.cartridge().hash();
        cartridge_hash.save(&mut writer)?;

        self.cpu.save(&mut writer)?;
        self.bus.save(&mut writer)?;
        Ok(())
    }

    pub fn load_state<R: Read>(&mut self, mut reader: R) -> Result<(), SaveError> {
        let mut magic = [0u8; 4];
        let mut version = 0usize;
        let mut hash = [0u8; 32];

        magic.load(&mut reader)?;
        if &magic != SAVE_STATE_MAGIC {
            return Err(SaveError::InvalidSaveStateHeader);
        }

        version.load(&mut reader)?;
        if version != SAVE_STATE_VERSION {
            return Err(SaveError::UnmatchedSaveErrorVersion(version));
        }

        hash.load(&mut reader)?;
        if &hash != self.bus.cartridge().hash() {
            return Err(SaveError::InvalidCartridgeHash);
        }

        self.cpu.load(&mut reader)?;
        self.bus.load(&mut reader)?;

        Ok(())
    }
}
