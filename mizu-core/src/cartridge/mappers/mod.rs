mod mbc1;
mod mbc2;
mod mbc3;
mod mbc5;
mod no_mapper;

pub(super) use mbc1::Mbc1;
pub(super) use mbc2::Mbc2;
pub(super) use mbc3::Mbc3;
pub(super) use mbc5::Mbc5;
pub(super) use no_mapper::NoMapper;

use save_state::SaveError;
use serde::{Deserialize, Serialize};

/// The number of clocks needed from the bus to complete one second
pub const ONE_SECOND_MAPPER_CLOCKS: u32 = 4194304 / 2;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MapperType {
    NoMapper,
    Mbc1 { multicart: bool },
    Mbc2,
    Mbc3 { timer: bool },
    Mbc5 { rumble: bool },
    Mmm01,
    Mbc6,
    Mbc7,
}

pub enum MappingResult {
    Addr(usize),
    Value(u8),
    NotMapped,
}

pub trait Mapper {
    fn init(&mut self, rom_banks: u16, ram_size: usize);

    fn map_read_rom0(&self, addr: u16) -> usize;

    fn map_read_romx(&self, addr: u16) -> usize;

    fn map_ram_read(&mut self, addr: u16) -> MappingResult;

    fn map_ram_write(&mut self, addr: u16, data: u8) -> MappingResult;

    fn write_bank_controller_register(&mut self, _addr: u16, _data: u8) {
        // ignored
    }

    fn save_battery_size(&self) -> usize {
        0
    }

    fn save_battery(&self) -> Vec<u8> {
        Vec::new()
    }

    fn load_battery(&mut self, _data: &[u8]) {
        // ignored
    }

    /// Fixed-timed updates from the bus, the main purpose is to be used to
    /// sync the MBC3 RTC clock to emulation in case emulation speed changed
    ///
    /// the number of clocks for one second is (ONE_SECOND_MAPPER_CLOCKS) 4194304 / 2
    fn clock(&mut self) {
        // ignore
    }

    // this only works because the types of the mapper have constant save size,
    // the problem is that we cannot have methods that contain generic like
    // `Savable::save` and `Savable::load` in a trait object, like in our case
    // with the mapper field in `Cartridge`
    //
    // TODO: find a better solution
    fn save_state_size(&self) -> Result<u64, SaveError>;
    fn save_state(&self) -> Result<Vec<u8>, SaveError>;
    fn load_state(&mut self, data: &[u8]) -> Result<(), SaveError>;
}
