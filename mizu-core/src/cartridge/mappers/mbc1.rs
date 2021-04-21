use super::{Mapper, MappingResult};
use save_state::Savable;

#[derive(Savable)]
pub struct Mbc1 {
    is_2k_ram: bool,
    ram_banks: u8,
    rom_banks: u16,

    /// true for rom, false for ram
    mode: bool,
    two_bit_bank2: u8,

    ram_enable: bool,

    rom_bank1: u8,

    multicart: bool,
}

impl Mbc1 {
    pub fn new(multicart: bool) -> Self {
        Self {
            is_2k_ram: false,
            ram_banks: 0,
            rom_banks: 0,
            mode: false,
            two_bit_bank2: 0,
            ram_enable: false,
            rom_bank1: 1,
            multicart,
        }
    }
}

impl Mbc1 {
    #[inline]
    fn bank2_shift(&self) -> u8 {
        if self.multicart {
            4
        } else {
            5
        }
    }
}

impl Mapper for Mbc1 {
    fn init(&mut self, rom_banks: u16, ram_size: usize) {
        self.rom_banks = rom_banks;
        self.ram_banks = (ram_size / 0x2000) as u8;
        self.is_2k_ram = ram_size == 0x800;
    }

    fn map_read_rom0(&self, addr: u16) -> usize {
        let bank = if self.mode {
            (self.two_bit_bank2 << self.bank2_shift()) % self.rom_banks as u8
        } else {
            0
        } as usize;

        bank * 0x4000 + addr as usize
    }

    fn map_read_romx(&self, addr: u16) -> usize {
        let addr = addr & 0x3FFF;

        let bank = (self.rom_bank1 | (self.two_bit_bank2 << self.bank2_shift())) as usize;
        let bank = bank % self.rom_banks as usize;

        bank as usize * 0x4000 + addr as usize
    }

    fn map_ram_read(&mut self, addr: u16) -> MappingResult {
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
                let bank = if self.mode { self.two_bit_bank2 } else { 0 } % self.ram_banks;
                MappingResult::Addr(bank as usize * 0x2000 + addr as usize)
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

                // For some reason the & is done after, so when bank1
                // is supplied with 0x10 (16) it will become 0 and not 1
                if self.multicart {
                    data &= 0xF;
                }

                self.rom_bank1 = data;
            }
            0x4000..=0x5FFF => {
                let data = data & 0x3;
                self.two_bit_bank2 = data;
            }
            0x6000..=0x7FFF => {
                self.mode = data & 1 == 1;
            }
            _ => {}
        }
    }

    fn save_state_size(&self) -> Result<u64, save_state::SaveError> {
        self.save_size()
    }

    fn save_state(&self) -> Result<Vec<u8>, save_state::SaveError> {
        save_state::save_object(self)
    }

    fn load_state(&mut self, data: &[u8]) -> Result<(), save_state::SaveError> {
        save_state::load_object(self, data)
    }
}
