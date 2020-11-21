use super::interrupts::Interrupts;
use crate::cartridge::Cartridge;
use crate::cpu::CpuBusProvider;
use crate::joypad::{Joypad, JoypadButton};
use crate::ppu::Ppu;
use crate::timer::Timer;

struct Ram {
    // DMG mode only, Color can switch the second bank
    data: [u8; 0x2000],
}

impl Default for Ram {
    fn default() -> Self {
        Self { data: [0; 0x2000] }
    }
}

/// read/write_ramx does the same as ram0, but should be changed
/// when supporting GBC
impl Ram {
    fn read_ram0(&self, addr: u16) -> u8 {
        self.data[addr as usize & 0xFFF]
    }

    fn read_ramx(&self, addr: u16) -> u8 {
        self.data[addr as usize & 0x1FFF]
    }

    fn write_ram0(&mut self, addr: u16, data: u8) {
        self.data[addr as usize & 0xFFF] = data;
    }
    fn write_ramx(&mut self, addr: u16, data: u8) {
        self.data[addr as usize & 0x1FFF] = data;
    }
}

pub struct Bus {
    cartridge: Cartridge,
    ppu: Ppu,
    ram: Ram,
    interrupts: Interrupts,
    timer: Timer,
    joypad: Joypad,
    hram: [u8; 127],
}

impl Bus {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            cartridge,
            ppu: Ppu::default(),
            ram: Ram::default(),
            interrupts: Interrupts::default(),
            timer: Timer::default(),
            joypad: Joypad::default(),
            hram: [0; 127],
        }
    }

    pub fn screen_buffer(&self) -> Vec<u8> {
        self.ppu.screen_buffer()
    }

    pub fn press_joypad(&mut self, button: JoypadButton) {
        self.joypad.press_joypad(button);
    }

    pub fn release_joypad(&mut self, button: JoypadButton) {
        self.joypad.release_joypad(button);
    }
}

impl Bus {
    fn on_cpu_machine_cycle(&mut self) {
        // clock the ppu four times
        for _ in 0..4 {
            self.ppu.clock(&mut self.interrupts);
        }
        self.timer.clock_divider(&mut self.interrupts);
        self.joypad.update_interrupts(&mut self.interrupts);
    }
}

impl CpuBusProvider for Bus {
    // each time the cpu reads, clock the ppu
    fn read(&mut self, addr: u16) -> u8 {
        self.on_cpu_machine_cycle();

        match addr {
            0x0000..=0x3FFF => self.cartridge.read_rom0(addr), // rom0
            0x4000..=0x7FFF => self.cartridge.read_romx(addr), // romx
            0x8000..=0x9FFF => self.ppu.read_vram(addr),       // ppu vram
            0xA000..=0xBFFF => self.cartridge.read_ram(addr),  // sram
            0xC000..=0xCFFF => self.ram.read_ram0(addr),       // wram0
            0xD000..=0xDFFF => self.ram.read_ramx(addr),       // wramx
            // 0xE000..=0xFDFF => 0xFF,                           // echo
            0xFE00..=0xFE9F => self.ppu.read_oam(addr), // ppu oam
            // 0xFEA0..=0xFEFF => 0xFF,                           // unused
            0xFF00 => self.joypad.read_joypad(),
            0xFF04..=0xFF07 => self.timer.read_register(addr), // divider and timer
            0xFF0F => self.interrupts.read_interrupt_flags(),
            0xFF40..=0xFF45 | 0xFF47..=0xFF4B => self.ppu.read_register(addr), // ppu io registers
            // 0xFF46 => 0xFF,                                    // dma start
            // 0xFF4C..=0xFF7F => 0xFF,                           // io registers
            0xFF80..=0xFFFE => self.hram[addr as usize & 0x7F], // hram
            0xFFFF => self.interrupts.read_interrupt_enable(),
            _ => {
                println!("Tried reading unmapped address {:04X}", addr);
                0xFF
            }
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x7FFF => self.cartridge.write_to_bank_controller(addr, data), // rom0
            0x8000..=0x9FFF => self.ppu.write_vram(addr, data),                     // ppu vram
            0xA000..=0xBFFF => self.cartridge.write_ram(addr, data),                // sram
            0xC000..=0xCFFF => self.ram.write_ram0(addr, data),                     // wram0
            0xD000..=0xDFFF => self.ram.write_ramx(addr, data),                     // wramx
            // 0xE000..=0xFDFF => {}                                                   // echo
            0xFE00..=0xFE9F => self.ppu.write_oam(addr, data), // ppu oam
            //0xFEA0..=0xFEFF => {}                                                   // unused
            0xFF00 => self.joypad.write_joypad(data),
            0xFF04..=0xFF07 => self.timer.write_register(addr, data), // divider and timer
            0xFF0F => self.interrupts.write_interrupt_flags(data),
            0xFF40..=0xFF45 | 0xFF47..=0xFF4B => self.ppu.write_register(addr, data), // ppu io registers
            // 0xFF46 => {}                                                              // dma start
            // 0xFF4C..=0xFF7F => {} // io registers
            0xFF80..=0xFFFE => self.hram[addr as usize & 0x7F] = data, // hram
            0xFFFF => self.interrupts.write_interrupt_enable(data),
            _ => {
                println!(
                    "Tried writing to unmapped address {:04X}, data = {:02X}",
                    addr, data
                );
            }
        }
    }

    fn get_interrupts(&mut self) -> Option<u8> {
        self.interrupts.get_highest_interrupt_addr_and_ack()
    }

    fn check_interrupts(&self) -> bool {
        self.interrupts.is_interrupts_available()
    }
}
