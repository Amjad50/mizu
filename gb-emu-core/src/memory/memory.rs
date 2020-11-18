use crate::cartridge::{Cartridge, CartridgeError};
use crate::cpu::CpuBusProvider;

struct Ram {
    // DMG mode only, Color can switch the second bank
    data: [u8; 0x2000],
}

impl Default for Ram {
    fn default() -> Self {
        Self { data: [0; 0x2000] }
    }
}

struct Bus {
    cartridge: Cartridge,
    // ppu: Ppu,
    ram: Ram,
}

impl CpuBusProvider for Bus {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => self.cartridge.read_rom0(addr), // rom0
            0x4000..=0x7FFF => self.cartridge.read_romx(addr), // romx
            0x8000..=0x9FFF => 0xFF,                           // ppu vram
            0xA000..=0xBFFF => self.cartridge.read_ram(addr),  // sram
            0xC000..=0xCFFF => 0xFF,                           // wram0
            0xD000..=0xDFFF => 0xFF,                           // wramx
            0xE000..=0xFDFF => 0xFF,                           // echo
            0xFE00..=0xFE9F => 0xFF,                           // ppu oam
            0xFEA0..=0xFEFF => 0xFF,                           // unused
            0xFF00..=0xFF7F => 0xFF,                           // io registers
            0xFF80..=0xFFFE => 0xFF,                           // hram
            0xFFFF => 0xFF,                                    // ie register?
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x7FFF => self.cartridge.write_to_bank_controller(addr, data), // rom0
            0x8000..=0x9FFF => {}                                                   // ppu vram
            0xA000..=0xBFFF => self.cartridge.write_ram(addr, data),                // sram
            0xC000..=0xCFFF => {}                                                   // wram0
            0xD000..=0xDFFF => {}                                                   // wramx
            0xE000..=0xFDFF => {}                                                   // echo
            0xFE00..=0xFE9F => {}                                                   // ppu oam
            0xFEA0..=0xFEFF => {}                                                   // unused
            0xFF00..=0xFF7F => {}                                                   // io registers
            0xFF80..=0xFFFE => {}                                                   // hram
            0xFFFF => {}                                                            // ie register?
        }
    }

    fn check_interrupts(&mut self) -> bool {
        todo!()
    }

    fn ack_interrupt(&mut self) {
        todo!()
    }
}
