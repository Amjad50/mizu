use super::{Mapper, MappingResult, ONE_SECOND_MAPPER_CLOCKS};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;
use std::time::{SystemTime, UNIX_EPOCH};

fn system_time_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|e| e.duration())
        .as_secs()
}

struct RtcRegister {
    /// A full second is ONE_SECOND_MAPPER_CLOCKS, which is synced to the bus
    sub_second: u32,

    seconds: u8,
    minutes: u8,
    hours: u8,
    days: u16,

    halt: bool,
    day_counter_carry: bool,

    last_latched_time: u64,
    latched: bool,

    current_time_secs: u64,
}

impl Default for RtcRegister {
    fn default() -> Self {
        let system_time = system_time_now();
        Self {
            seconds: 0,
            minutes: 0,
            hours: 0,
            days: 0,
            halt: false,
            day_counter_carry: false,
            last_latched_time: system_time,
            latched: false,

            sub_second: 0,
            current_time_secs: system_time,
        }
    }
}

impl RtcRegister {
    fn read_register(&mut self, index: u8) -> u8 {
        if !self.latched {
            self.update_registers();
        }

        match index {
            0 => self.seconds,
            1 => self.minutes,
            2 => self.hours,
            3 => (self.days & 0xFF) as u8,
            4 => {
                ((self.day_counter_carry as u8) << 7)
                    | ((self.halt as u8) << 6)
                    | ((self.days >> 8) & 1) as u8
            }
            _ => unreachable!(),
        }
    }

    fn write_register(&mut self, index: u8, data: u8) {
        let old_halt = self.halt;

        match index {
            0 => {
                self.seconds = data & 0x3F;
                self.sub_second = 0;
            }
            1 => self.minutes = data & 0x3F,
            2 => self.hours = data & 0x1F,
            3 => {
                self.days &= 0x100;
                self.days |= data as u16;
            }
            4 => {
                self.days &= 0xFF;
                self.days |= ((data & 1) as u16) << 8;
                self.halt = (data >> 6) & 1 == 1;
                self.day_counter_carry = (data >> 7) & 1 == 1;
            }
            _ => unreachable!(),
        }

        if old_halt && !self.halt {
            self.last_latched_time = self.current_time_secs;
        }
    }

    fn set_latch(&mut self, value: bool) {
        self.latched = value;
        if !self.latched {
            self.update_registers();
        }
    }

    fn update_registers(&mut self) {
        if self.halt {
            return;
        }

        let new_time = self.current_time_secs;

        if let Some(diff) = new_time.checked_sub(self.last_latched_time) {
            if diff != 0 {
                let seconds = diff % 60;
                let minutes = (diff / 60) % 60;
                let hours = (diff / 60 / 60) % 24;
                let days = diff / 60 / 60 / 24;

                self.add_to_registers(seconds as u8, minutes as u8, hours as u8, days);
                self.last_latched_time = new_time;
            }
        } else {
            // error in subtracting time (maybe time changed to the past?)
            self.last_latched_time = new_time;
        }
    }

    fn add_to_registers(&mut self, seconds: u8, mut minutes: u8, mut hours: u8, mut days: u64) {
        self.seconds = (self.seconds + seconds) & 0x3F;
        if self.seconds == 60 {
            minutes += 1;
            self.seconds = 0;
        }

        self.minutes = (self.minutes + minutes) & 0x3F;
        if self.minutes == 60 {
            hours += 1;
            self.minutes = 0;
        }

        self.hours = (self.hours + hours) & 0x1F;
        if self.hours == 24 {
            days += 1;
            self.hours = 0;
        }

        self.days = (self.days as u64).wrapping_add(days) as u16;
        self.day_counter_carry = self.days > 0x1FF;
        self.days &= 0x1FF;
    }

    fn save_battery_size(&self) -> usize {
        std::mem::size_of::<u8>() * 3 + std::mem::size_of::<u16>() + std::mem::size_of::<u64>()
    }

    fn save_battery(&self) -> Vec<u8> {
        let result = Vec::with_capacity(self.save_battery_size());
        let mut cur = Cursor::new(result);

        cur.write_u8(self.seconds).unwrap();
        cur.write_u8(self.minutes).unwrap();
        cur.write_u8(self.hours).unwrap();
        cur.write_u16::<LittleEndian>(self.days).unwrap();
        cur.write_u64::<LittleEndian>(self.last_latched_time)
            .unwrap();

        let result = cur.into_inner();
        assert_eq!(result.len(), self.save_battery_size());

        result
    }

    fn load_battery(&mut self, data: &[u8]) {
        let mut cur = Cursor::new(data);

        self.seconds = cur.read_u8().unwrap();
        self.minutes = cur.read_u8().unwrap();
        self.hours = cur.read_u8().unwrap();
        self.days = cur.read_u16::<LittleEndian>().unwrap();
        self.last_latched_time = cur.read_u64::<LittleEndian>().unwrap();
        self.current_time_secs = self.last_latched_time;
    }

    fn clock_second_part(&mut self) {
        if !self.halt {
            self.sub_second += 1;

            if self.sub_second == ONE_SECOND_MAPPER_CLOCKS {
                self.sub_second = 0;
                self.current_time_secs += 1;
            }
        }
    }
}

#[derive(Default)]
pub struct Mbc3 {
    rom_banks: u16,
    is_2k_ram: bool,
    ram_banks: u8,
    rtc_present: bool,

    /// the bank number to use in the memory [0x4000..=0x7FFF]
    rom_bank_4000: u16,
    ram_bank: u8,

    current_rtc_register: u8,

    rtc_register: RtcRegister,

    ram_block_enable: bool,
    is_reading_ram: bool,
}

impl Mbc3 {
    pub fn new(timer: bool) -> Self {
        Self {
            rtc_present: timer,
            rom_bank_4000: 1,
            ram_block_enable: true,

            ..Self::default()
        }
    }
}

impl Mbc3 {
    fn map_ram(&self, addr: u16) -> MappingResult {
        if self.ram_banks == 0 {
            MappingResult::NotMapped
        } else {
            if self.is_2k_ram {
                MappingResult::Addr(addr as usize & 0x7FF)
            } else {
                let addr = addr & 0x1FFF;
                let bank = self.ram_bank % self.ram_banks;

                MappingResult::Addr(bank as usize * 0x2000 + addr as usize)
            }
        }
    }

    fn rtc_read(&mut self) -> u8 {
        self.rtc_register.read_register(self.current_rtc_register)
    }

    fn rtc_write(&mut self, data: u8) {
        self.rtc_register
            .write_register(self.current_rtc_register, data);
    }
}

impl Mapper for Mbc3 {
    fn init(&mut self, rom_banks: u16, ram_size: usize) {
        assert!(rom_banks <= 256);
        self.rom_banks = rom_banks;
        self.ram_banks = (ram_size / 0x2000) as u8;
        self.is_2k_ram = ram_size == 0x800;
    }

    fn map_read_rom0(&self, addr: u16) -> usize {
        addr as usize
    }

    fn map_read_romx(&self, addr: u16) -> usize {
        let addr = addr & 0x3FFF;

        let bank = self.rom_bank_4000 % self.rom_banks;

        bank as usize * 0x4000 + addr as usize
    }

    fn map_ram_read(&mut self, addr: u16) -> MappingResult {
        if self.ram_block_enable {
            if self.is_reading_ram {
                self.map_ram(addr)
            } else {
                if self.rtc_present {
                    MappingResult::Value(self.rtc_read())
                } else {
                    MappingResult::NotMapped
                }
            }
        } else {
            MappingResult::NotMapped
        }
    }

    fn map_ram_write(&mut self, addr: u16, data: u8) -> MappingResult {
        if self.ram_block_enable {
            if self.is_reading_ram {
                self.map_ram(addr)
            } else {
                if self.rtc_present {
                    self.rtc_write(data);
                }

                MappingResult::NotMapped
            }
        } else {
            MappingResult::NotMapped
        }
    }

    fn write_bank_controller_register(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_block_enable = data & 0xF == 0xA,
            0x2000..=0x3FFF => {
                self.rom_bank_4000 = data as u16;
                if self.rom_bank_4000 == 0 {
                    self.rom_bank_4000 = 1;
                }
            }
            0x4000..=0x5FFF => {
                let data = data & 0xF;
                // FIXME: what happens if the value written is outside 0x0..=0xC?
                assert!(data <= 3 || (0x8..=0xC).contains(&data));

                self.is_reading_ram = data <= 3;

                if self.is_reading_ram {
                    self.ram_bank = data & 3;
                } else {
                    self.current_rtc_register = data - 0x8;
                }
            }
            0x6000..=0x7FFF => {
                if self.rtc_present {
                    self.rtc_register.set_latch(data & 1 == 1);
                }
            }
            _ => {}
        }
    }

    fn save_battery_size(&self) -> usize {
        if self.rtc_present {
            self.rtc_register.save_battery_size()
        } else {
            0
        }
    }

    fn save_battery(&self) -> Vec<u8> {
        if self.rtc_present {
            self.rtc_register.save_battery()
        } else {
            Vec::new()
        }
    }

    fn load_battery(&mut self, data: &[u8]) {
        if self.rtc_present {
            self.rtc_register.load_battery(data)
        }
    }

    fn clock(&mut self) {
        self.rtc_register.clock_second_part();
    }
}
