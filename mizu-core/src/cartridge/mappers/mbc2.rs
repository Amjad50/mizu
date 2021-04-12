use super::{Mapper, MappingResult};
use serde::{Deserialize, Serialize};

type Mbc2Ram = [u8; 512];

mod mbc2_ram_serde {
    use super::Mbc2Ram;
    use serde::{
        de::{Error, Visitor},
        Deserializer, Serializer,
    };
    use std::convert::TryInto;

    pub fn deserialize<'de, D>(d: D) -> Result<Mbc2Ram, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BytesVisitor;

        impl<'a> Visitor<'a> for BytesVisitor {
            type Value = Mbc2Ram;

            fn expecting(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(fmt, "512 `u8` array")
            }

            fn visit_bytes<E>(self, visitor: &[u8]) -> Result<Self::Value, E>
            where
                E: Error,
            {
                visitor
                    .try_into()
                    .or(Err(Error::invalid_length(visitor.len(), &self)))
            }
        }

        d.deserialize_bytes(BytesVisitor)
    }

    pub fn serialize<S>(t: &[u8], s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_bytes(t)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Mbc2 {
    rom_banks: u8,

    /// the bank number to use in the memory [0x4000..=0x7FFF]
    rom_bank_4000: u8,

    /// internal 512x4bit ram
    #[serde(with = "mbc2_ram_serde")]
    ram: Mbc2Ram,

    ram_enable: bool,
}

impl Default for Mbc2 {
    fn default() -> Self {
        Self {
            rom_banks: 0,
            rom_bank_4000: 1,
            ram: [0; 512],
            ram_enable: false,
        }
    }
}

#[typetag::serde]
impl Mapper for Mbc2 {
    fn init(&mut self, rom_banks: u16, _ram_size: usize) {
        assert!(rom_banks <= 16);
        self.rom_banks = rom_banks as u8;
    }

    fn map_read_rom0(&self, addr: u16) -> usize {
        // same address for the first rom bank
        addr as usize
    }

    fn map_read_romx(&self, addr: u16) -> usize {
        let addr = addr & 0x3FFF;

        let bank = self.rom_bank_4000 % self.rom_banks;

        bank as usize * 0x4000 + addr as usize
    }

    fn map_ram_read(&mut self, addr: u16) -> MappingResult {
        if self.ram_enable {
            MappingResult::Value(0xF0 | self.ram[addr as usize & 0x1FF])
        } else {
            MappingResult::NotMapped
        }
    }

    fn map_ram_write(&mut self, addr: u16, data: u8) -> MappingResult {
        if self.ram_enable {
            self.ram[addr as usize & 0x1FF] = data & 0xF;
        }

        MappingResult::NotMapped
    }

    fn write_bank_controller_register(&mut self, addr: u16, data: u8) {
        if addr <= 0x3FFF {
            if addr & 0x100 == 0 {
                self.ram_enable = data & 0xF == 0xA;
            } else {
                self.rom_bank_4000 = data & 0xF;
                if self.rom_bank_4000 == 0 {
                    self.rom_bank_4000 = 1;
                }
            }
        }
    }

    fn save_battery_size(&self) -> usize {
        512
    }

    fn save_battery(&self) -> Vec<u8> {
        self.ram.into()
    }

    fn load_battery(&mut self, data: &[u8]) {
        assert!(data.len() == 512);

        self.ram.copy_from_slice(data);
    }
}
