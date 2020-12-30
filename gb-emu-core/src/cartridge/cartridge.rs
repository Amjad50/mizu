use super::error::{CartridgeError, SramError};
use super::mappers;
use super::mappers::{Mapper, MapperType, MappingResult};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug)]
enum GameBoyType {
    NonColor,
    Color,
}

#[derive(Debug)]
struct CartridgeType {
    mapper_type: MapperType,
    ram: bool,
    battery: bool,
}

impl CartridgeType {
    fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(Self {
                mapper_type: MapperType::NoMapper,
                ram: false,
                battery: false,
            }),
            1 => Some(Self {
                mapper_type: MapperType::Mbc1 { multicart: false },
                ram: false,
                battery: false,
            }),
            2 => Some(Self {
                mapper_type: MapperType::Mbc1 { multicart: false },
                ram: true,
                battery: false,
            }),
            3 => Some(Self {
                mapper_type: MapperType::Mbc1 { multicart: false },
                ram: true,
                battery: true,
            }),
            5 => Some(Self {
                mapper_type: MapperType::Mbc2,
                ram: false,
                battery: false,
            }),
            6 => Some(Self {
                mapper_type: MapperType::Mbc2,
                ram: true,
                battery: true,
            }),
            8 => Some(Self {
                mapper_type: MapperType::NoMapper,
                ram: true,
                battery: false,
            }),
            9 => Some(Self {
                mapper_type: MapperType::NoMapper,
                ram: true,
                battery: true,
            }),
            0xB => Some(Self {
                mapper_type: MapperType::Mmm01,
                ram: false,
                battery: false,
            }),
            0xC => Some(Self {
                mapper_type: MapperType::Mmm01,
                ram: true,
                battery: false,
            }),
            0xD => Some(Self {
                mapper_type: MapperType::Mmm01,
                ram: true,
                battery: true,
            }),
            0xF => Some(Self {
                mapper_type: MapperType::Mbc3 { timer: true },
                ram: false,
                battery: true,
            }),
            0x10 => Some(Self {
                mapper_type: MapperType::Mbc3 { timer: true },
                ram: true,
                battery: true,
            }),
            0x11 => Some(Self {
                mapper_type: MapperType::Mbc3 { timer: false },
                ram: false,
                battery: false,
            }),
            0x12 => Some(Self {
                mapper_type: MapperType::Mbc3 { timer: false },
                ram: true,
                battery: false,
            }),
            0x13 => Some(Self {
                mapper_type: MapperType::Mbc3 { timer: false },
                ram: true,
                battery: true,
            }),
            0x19 => Some(Self {
                mapper_type: MapperType::Mbc5 { rumble: false },
                ram: false,
                battery: false,
            }),
            0x1A => Some(Self {
                mapper_type: MapperType::Mbc5 { rumble: false },
                ram: true,
                battery: false,
            }),
            0x1B => Some(Self {
                mapper_type: MapperType::Mbc5 { rumble: false },
                ram: true,
                battery: true,
            }),
            0x1C => Some(Self {
                mapper_type: MapperType::Mbc5 { rumble: true },
                ram: false,
                battery: false,
            }),
            0x1D => Some(Self {
                mapper_type: MapperType::Mbc5 { rumble: true },
                ram: true,
                battery: false,
            }),
            0x1E => Some(Self {
                mapper_type: MapperType::Mbc5 { rumble: true },
                ram: true,
                battery: true,
            }),
            0x20 => Some(Self {
                mapper_type: MapperType::Mbc6,
                ram: true,
                battery: true,
            }),
            0x22 => Some(Self {
                mapper_type: MapperType::Mbc7,
                ram: true,
                battery: true,
            }),
            _ => None,
        }
    }

    /// Checks if the cartridge is MBC1 multicart and update the `mapper` value
    fn update_mbc1_multicart(&mut self, data: &[u8]) {
        if let MapperType::Mbc1 { ref mut multicart } = self.mapper_type {
            // Must be 8 MegaBits
            if data.len() == 0x100000 {
                for i in 0..=3 {
                    let bank_start = i << 18;

                    if &data[(bank_start + 0x104)..=(bank_start + 0x133)] != NINTENDO_LOGO_DATA {
                        // Not multicart
                        return;
                    }
                }
                // If all checks pass
                *multicart = true;
            }
        }
    }

    fn get_mapper(&self) -> Option<Box<dyn Mapper>> {
        let mapper: Box<dyn Mapper> = match self.mapper_type {
            MapperType::NoMapper => Box::new(mappers::NoMapper::default()),
            MapperType::Mbc1 { multicart } => Box::new(mappers::Mbc1::new(multicart)),
            MapperType::Mbc2 => Box::new(mappers::Mbc2::default()),
            MapperType::Mbc3 { timer } => Box::new(mappers::Mbc3::new(timer)),
            MapperType::Mbc5 { rumble } => Box::new(mappers::Mbc5::new(rumble)),
            _ => return None,
        };

        Some(mapper)
    }
}

pub struct Cartridge {
    file_path: Box<Path>,
    game_title: String,
    cartridge_type: CartridgeType,
    mapper: Box<dyn Mapper>,
    rom: Vec<u8>,
    ram: Vec<u8>,
}

impl Cartridge {
    pub fn from_file<P: AsRef<Path>>(file_path: P) -> Result<Self, CartridgeError> {
        let extension = file_path
            .as_ref()
            .extension()
            .ok_or(CartridgeError::ExtensionError)?;

        if extension != "gb" {
            return Err(CartridgeError::ExtensionError);
        }

        let mut file = File::open(file_path.as_ref())?;

        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        if data.len() < 0x8000 || data.len() % 0x4000 != 0 {
            eprintln!(
                "[WARN]: the cartridge contain invalid rom size {:X}",
                data.len()
            );
        }

        // extend the data, as some roms don't follow the rules :(
        if data.len() < 0x8000 {
            data.extend_from_slice(&vec![0; 0x8000 - data.len()]);
        }
        if data.len() % 0x4000 != 0 {
            data.extend_from_slice(&vec![0; 0x4000 - (data.len() % 0x4000)]);
        }

        if &data[0x104..=0x133] != NINTENDO_LOGO_DATA {
            return Err(CartridgeError::InvalidNintendoLogo);
        }

        let game_title = String::from_utf8(
            data[0x134..=0x142]
                .iter()
                .copied()
                .take_while(|e| e != &0)
                .collect::<Vec<u8>>(),
        )
        .map_err(|_| CartridgeError::InvalidGameTitle)?;

        let gameboy_type = if data[0x143] == 0x80 {
            GameBoyType::Color
        } else {
            GameBoyType::NonColor
        };

        println!("gameboy type {:?}", gameboy_type);

        let mut cartridge_type =
            CartridgeType::from_byte(data[0x147]).ok_or(CartridgeError::InvalidCartridgeType)?;

        // checks if its mbc1 multicart and update the mapper type
        cartridge_type.update_mbc1_multicart(&data);

        let num_rom_banks = data[0x148];
        if num_rom_banks > 8 {
            return Err(CartridgeError::InvalidRomSizeIndex(data[0x148]));
        }

        let rom_size = 0x8000 << num_rom_banks;

        if rom_size != data.len() {
            return Err(CartridgeError::InvalidRomSize(rom_size));
        }

        let ram_size = match data[0x149] {
            0 => 0,
            1 => 0x800,
            2 => 0x2000,
            3 => 0x8000,
            4 => 0x20000,
            5 => 0x10000,
            _ => {
                return Err(CartridgeError::InvalidRamSizeIndex(data[0x149]));
            }
        };

        if cartridge_type.ram && ram_size == 0 {
            // cartridge_type.ram = false;
            // return Err(CartridgeError::RamNotPresentError);
        } else if !cartridge_type.ram && ram_size != 0 {
            return Err(CartridgeError::NotNeededRamPresentError);
        }

        let mut ram = vec![0; ram_size];

        let mut checksum = 0u8;
        for &i in data[0x134..=0x14c].iter() {
            checksum = checksum.wrapping_sub(i).wrapping_sub(1);
        }

        if checksum != data[0x14d] {
            return Err(CartridgeError::InvalidChecksum {
                got: checksum,
                expected: data[0x14d],
            });
        }

        println!("LOG: {:?}", cartridge_type);

        let mut mapper =
            cartridge_type
                .get_mapper()
                .ok_or(CartridgeError::MapperNotImplemented(
                    cartridge_type.mapper_type,
                ))?;

        mapper.init((rom_size / 0x4000) as u16, ram_size);

        if cartridge_type.battery {
            if let Ok((saved_ram, extra)) =
                Self::load_sram_file(file_path.as_ref(), ram_size, mapper.save_battery_size())
            {
                ram = saved_ram;
                mapper.load_battery(&extra);
            }
        }

        Ok(Self {
            file_path: file_path.as_ref().to_path_buf().into_boxed_path(),
            game_title,
            cartridge_type,
            mapper,
            rom: data,
            ram,
        })
    }

    /// 0x0000-0x3FFF
    pub fn read_rom0(&self, addr: u16) -> u8 {
        let addr = self.mapper.map_read_rom0(addr);

        self.rom[addr]
    }

    // TODO: implement mapper
    /// 0x4000-0x7FFF
    pub fn read_romx(&self, addr: u16) -> u8 {
        let addr = self.mapper.map_read_romx(addr);

        self.rom[addr]
    }

    /// 0x0000-0x7FFF
    pub fn write_to_bank_controller(&mut self, addr: u16, data: u8) {
        self.mapper.write_bank_controller_register(addr, data);
    }

    /// 0xA000-0xBFFF
    pub fn read_ram(&mut self, addr: u16) -> u8 {
        match self.mapper.map_ram_read(addr) {
            MappingResult::Addr(addr) => self.ram[addr],
            MappingResult::Value(value) => value,
            MappingResult::NotMapped => 0xFF,
        }
    }

    /// 0xA000-0xBFFF
    pub fn write_ram(&mut self, addr: u16, data: u8) {
        match self.mapper.map_ram_write(addr, data) {
            MappingResult::Addr(addr) => self.ram[addr] = data,
            MappingResult::NotMapped | MappingResult::Value(_) => {}
        }
    }
}

impl Cartridge {
    fn load_sram_file<P: AsRef<Path>>(
        path: P,
        sram_size: usize,
        extra_size: usize,
    ) -> Result<(Vec<u8>, Vec<u8>), SramError> {
        let path = path.as_ref().with_extension("gb.sav");
        println!("Loading SRAM file data from {:?}", path);

        let mut file = File::open(path)?;
        let mut result = vec![0; sram_size];
        let mut extra = vec![0; extra_size];

        file.read_exact(&mut result)
            .map_err(|_| SramError::SramFileSizeDoesNotMatch)?;

        file.read_exact(&mut extra)
            .map_err(|_| SramError::SramFileSizeDoesNotMatch)?;

        Ok((result, extra))
    }

    fn save_sram_file(&self) -> Result<(), SramError> {
        let path = self.file_path.with_extension("gb.sav");
        println!("Writing SRAM file data to {:?}", path);

        let mut file = File::create(&path)?;

        let size = file.write(&self.ram)?;

        if size != self.ram.len() {
            file.sync_all()?;
            // remove the file so it will not be loaded next time the game is run
            std::fs::remove_file(path).expect("Could not remove `gb.sav` file");
            return Err(SramError::FailedToSaveSramFile);
        }

        let extra = self.mapper.save_battery();

        let size = file.write(&extra)?;

        if size != extra.len() {
            file.sync_all()?;
            // remove the file so it will not be loaded next time the game is run
            std::fs::remove_file(path).expect("Could not remove `gb.sav` file");
            return Err(SramError::FailedToSaveSramFile);
        }

        Ok(())
    }
}

impl Drop for Cartridge {
    fn drop(&mut self) {
        if self.cartridge_type.battery {
            self.save_sram_file().unwrap();
        }
    }
}

const NINTENDO_LOGO_DATA: &[u8; 48] = &[
    0xce, 0xed, 0x66, 0x66, 0xcc, 0x0d, 0x00, 0x0b, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0c, 0x00, 0x0d,
    0x00, 0x08, 0x11, 0x1f, 0x88, 0x89, 0x00, 0x0e, 0xdc, 0xcc, 0x6e, 0xe6, 0xdd, 0xdd, 0xd9, 0x99,
    0xbb, 0xbb, 0x67, 0x63, 0x6e, 0x0e, 0xec, 0xcc, 0xdd, 0xdc, 0x99, 0x9f, 0xbb, 0xb9, 0x33, 0x3e,
];
