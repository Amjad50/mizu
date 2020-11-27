use super::cartridge::{Cartridge, CartridgeError};
use super::cpu::Cpu;
use super::memory::Bus;
use super::JoypadButton;

use std::path::Path;

pub struct GameBoy {
    cpu: Cpu,
    bus: Bus,
}

impl GameBoy {
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Self, CartridgeError> {
        let cartridge = Cartridge::from_file(file_path)?;

        Ok(Self {
            bus: Bus::new(cartridge),
            cpu: Cpu::new(),
        })
    }

    pub fn clock(&mut self) {
        // this will clock the Bus as well as many times as it needs
        self.cpu.next_instruction(&mut self.bus);
    }

    pub fn screen_buffer(&self) -> Vec<u8> {
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
}
