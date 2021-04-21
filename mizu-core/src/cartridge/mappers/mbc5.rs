use super::{Mapper, MappingResult};
use save_state::Savable;

#[derive(Default, Savable)]
pub struct Mbc5 {
    rom_banks: u16,
    is_2k_ram: bool,
    ram_banks: u8,

    ram_enable: bool,
    ram_bank: u8,
    rom_bank: u16,

    // TODO: use this idk how
    _rumble: bool,
}

impl Mbc5 {
    pub fn new(rumble: bool) -> Self {
        Self {
            _rumble: rumble,
            rom_bank: 1,
            ..Self::default()
        }
    }
}

impl Mapper for Mbc5 {
    fn init(&mut self, rom_banks: u16, ram_size: usize) {
        assert!(rom_banks <= 512);
        self.rom_banks = rom_banks;
        self.ram_banks = (ram_size / 0x2000) as u8;
        self.is_2k_ram = ram_size == 0x800;
    }

    fn map_read_rom0(&self, addr: u16) -> usize {
        addr as usize
    }

    fn map_read_romx(&self, addr: u16) -> usize {
        let addr = addr & 0x3FFF;

        let bank = self.rom_bank % self.rom_banks;

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
                let bank = self.ram_bank % self.ram_banks;
                MappingResult::Addr(bank as usize * 0x2000 + addr as usize)
            }
        }
    }

    fn map_ram_write(&mut self, addr: u16, _data: u8) -> MappingResult {
        self.map_ram_read(addr)
    }

    fn write_bank_controller_register(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_enable = data == 0xA,
            0x2000..=0x2FFF => {
                self.rom_bank &= 0x100;
                self.rom_bank |= data as u16;
            }
            0x3000..=0x3FFF => {
                self.rom_bank &= 0xFF;
                self.rom_bank |= ((data & 1) as u16) << 8;
            }
            0x4000..=0x5FFF => {
                self.ram_bank = data & 0xF;
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
