use super::{Mapper, MappingResult};

pub struct Mbc1 {
    is_2k_ram: bool,
    ram_banks: u8,
    rom_banks: u8,
    /// true for rom, false for ram
    two_bit_mode_rom: bool,

    ram_enable: bool,

    rom_bank: u8,
    ram_bank: u8,
}

impl Default for Mbc1 {
    fn default() -> Self {
        Self {
            is_2k_ram: false,
            ram_banks: 0,
            rom_banks: 0,
            two_bit_mode_rom: true,
            ram_enable: false,
            rom_bank: 1,
            ram_bank: 0,
        }
    }
}

impl Mapper for Mbc1 {
    fn init(&mut self, rom_banks: u8, ram_size: usize) {
        self.rom_banks = rom_banks;
        self.ram_banks = (ram_size / 0x8000) as u8;
        self.is_2k_ram = ram_size == 0x800;
    }

    fn map_read_romx(&self, addr: u16) -> usize {
        let addr = addr & 0x3FFF;
        self.rom_bank as usize * 0x8000 + addr as usize
    }

    fn map_ram_read(&self, addr: u16) -> MappingResult {
        if !self.ram_enable {
            return MappingResult::NotMapped;
        }

        if self.is_2k_ram {
            MappingResult::Addr(addr as usize & 0x7FF)
        } else {
            if self.ram_banks == 0 {
                MappingResult::NotMapped
            } else {
                let addr = addr & 0x1FFF;
                MappingResult::Addr(self.ram_bank as usize * 0x2000 + addr as usize)
            }
        }
    }

    fn map_ram_write(&mut self, addr: u16, _data: u8) -> MappingResult {
        self.map_ram_read(addr)
    }

    fn write_bank_controller_register(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_enable = data & 0xf == 0xa,
            0x2000..=0x3FFF => {
                let mut data = data & 0x1F;
                if data == 0 {
                    data = 1;
                }
                self.rom_bank &= 0xE0;
                self.rom_bank |= data;
            }
            0x4000..=0x5FFF => {
                let data = data & 0x3;
                if self.two_bit_mode_rom {
                    self.rom_bank &= 0x1F;
                    self.rom_bank |= data << 5;
                } else {
                    self.ram_bank = data;
                }
            }
            0x6000..=0x7FFF => {
                if data & 1 == 0 {
                    // clear two bits of ram bank
                    self.ram_bank = 0;
                } else {
                    // clear two bits of rom bank
                    self.rom_bank &= 0x1F;
                }
            }
            _ => unreachable!(),
        }
    }
}
