use super::cartridge::{Cartridge, CartridgeError};
use super::cpu::Cpu;
use super::memory::Bus;

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
}
