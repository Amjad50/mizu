mod interrupts;

pub use interrupts::{InterruptManager, InterruptType};

use crate::apu::Apu;
use crate::cartridge::Cartridge;
use crate::cpu::CpuBusProvider;
use crate::joypad::{Joypad, JoypadButton};
use crate::ppu::Ppu;
use crate::serial::Serial;
use crate::timer::Timer;
use interrupts::Interrupts;

struct BootRom {
    enabled: bool,
    data: [u8; 0x900],
}

impl Default for BootRom {
    fn default() -> Self {
        Self {
            enabled: false,
            data: [0; 0x900],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Speed {
    Normal,
    Double,
}

impl Default for Speed {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Default)]
struct SpeedController {
    preparing_switch: bool,
    current_speed: Speed,
}

impl SpeedController {
    fn read_key1(&self) -> u8 {
        0x7E | ((self.current_speed as u8) << 7) | self.preparing_switch as u8
    }

    fn write_key1(&mut self, data: u8) {
        self.preparing_switch = data & 1 != 0;
    }

    fn preparing_switch(&self) -> bool {
        self.preparing_switch
    }

    fn current_speed(&self) -> Speed {
        self.current_speed
    }

    fn commit_speed_switch(&mut self) {
        assert!(self.preparing_switch);
        self.current_speed = match self.current_speed {
            Speed::Normal => Speed::Double,
            Speed::Double => Speed::Normal,
        };
        self.preparing_switch = false;
    }
}

#[derive(Clone, Copy)]
enum BusType {
    // VRAM
    Video,
    // Cartridge ROM and SRAM, and WRAM
    External,
}

#[derive(Default)]
struct HDMA {
    source_addr: u16,
    dest_addr: u16,
    length: u8,
    /// `true` if transfere during hblank only
    hblank_dma: bool,
    master_dma_active: bool,
    hblank_dma_active: bool,
    cached_ppu_hblank: bool,
}

impl HDMA {
    fn write_register(&mut self, addr: u16, data: u8) {
        match addr {
            0xFF51 => {
                // high src
                self.source_addr &= 0xFF;
                self.source_addr |= (data as u16) << 8;
            }
            0xFF52 => {
                // low src
                self.source_addr &= 0xFF00;
                // the lower 4 bits are ignored
                self.source_addr |= (data & 0xF0) as u16;
            }
            0xFF53 => {
                // high dest
                self.dest_addr &= 0xFF;
                // the top 3 bits are ignored and forced to 0x8 to be
                // in VRAM at all time
                self.dest_addr |= (((data & 0x1F) | 0x80) as u16) << 8;
            }
            0xFF54 => {
                // low dest
                self.dest_addr &= 0xFF00;
                // the lower 4 bits are ignored
                self.dest_addr |= (data & 0xF0) as u16;
            }
            0xFF55 => {
                // control
                self.length = data & 0x7F;
                if self.master_dma_active {
                    // make sure we are in hblank only
                    assert!(self.hblank_dma);

                    self.master_dma_active = data & 0x80 != 0;

                    // TODO: if new_flag is true, it should restart transfere.
                    //  check if source should start from the beginning or
                    //  current value
                    self.source_addr &= 0xFFF0;
                    self.dest_addr &= 0xFFF0;
                } else {
                    self.master_dma_active = true;
                    self.cached_ppu_hblank = false;
                    self.hblank_dma = data & 0x80 != 0;
                }
            }
            _ => unreachable!(),
        }
    }

    fn read_register(&mut self, addr: u16) -> u8 {
        match addr {
            0xFF51..=0xFF54 => 0xFF,
            0xFF55 => (((!self.master_dma_active) as u8) << 7) | self.length, // control
            _ => unreachable!(),
        }
    }

    fn get_next_src_address(&mut self) -> u16 {
        let result = self.source_addr;
        self.source_addr += 1;
        result
    }

    fn transfer_clock(&mut self, ppu: &mut Ppu, values: &[u8]) {
        for value in values {
            ppu.write_vram(self.dest_addr, *value);
            self.dest_addr += 1;

            if self.dest_addr & 0xF == 0 {
                self.hblank_dma_active = false;
                self.length = self.length.wrapping_sub(1);

                if self.length == 0xFF {
                    self.master_dma_active = false;
                }
            }
        }
    }

    fn is_transferreing(&mut self, ppu: &Ppu) -> bool {
        let new_ppu_hblank_mode = ppu.get_current_mode() == 0;

        if self.hblank_dma && !self.hblank_dma_active {
            if !self.cached_ppu_hblank && new_ppu_hblank_mode {
                self.hblank_dma_active = true;
            }
        }
        self.cached_ppu_hblank = new_ppu_hblank_mode;

        self.master_dma_active && (!self.hblank_dma || self.hblank_dma_active)
    }
}

#[derive(Default)]
struct DMA {
    conflicting_bus: Option<BusType>,
    current_value: u8,
    address: u16,
    in_transfer: bool,
    starting_delay: u8,
}

impl DMA {
    fn start_dma(&mut self, mut high_byte: u8) {
        // addresses changed from internal bus into external bus
        if high_byte == 0xFE || high_byte == 0xFF {
            high_byte &= 0xDF;
        }

        self.address = (high_byte as u16) << 8;

        // 8 T-cycles here for delay instead of 4, this is to ensure correct
        // DMA timing
        self.starting_delay = 2;
        self.in_transfer = true;
    }

    fn read(&self) -> u8 {
        (self.address >> 8) as u8
    }

    fn transfer_clock(&mut self, ppu: &mut Ppu, value: u8) {
        if self.starting_delay > 0 {
            self.starting_delay -= 1;

            // block after 1 M-cycle delay
            if self.starting_delay == 0 {
                let high_byte = (self.address >> 8) as u8;

                self.conflicting_bus = Some(if (0x80..=0x9F).contains(&high_byte) {
                    BusType::Video
                } else {
                    BusType::External
                });
            }
        } else {
            self.current_value = value;

            ppu.write_oam(0xFE00 | (self.address & 0xFF), value);

            self.address += 1;
            if self.address & 0xFF == 0xA0 {
                self.in_transfer = false;
                self.conflicting_bus = None;
            }
        }
    }
}

struct Wram {
    data: [u8; 0x8000],
    bank: u8,
}

impl Default for Wram {
    fn default() -> Self {
        Self {
            data: [0; 0x8000],
            bank: 1,
        }
    }
}

impl Wram {
    fn read_wram0(&self, addr: u16) -> u8 {
        self.data[addr as usize & 0xFFF]
    }

    fn read_wramx(&self, addr: u16) -> u8 {
        self.data[(0x1000 * self.bank as usize) + (addr as usize & 0xFFF)]
    }

    fn write_wram0(&mut self, addr: u16, data: u8) {
        self.data[addr as usize & 0xFFF] = data;
    }

    fn write_wramx(&mut self, addr: u16, data: u8) {
        self.data[(0x1000 * self.bank as usize) + (addr as usize & 0xFFF)] = data;
    }

    fn set_wram_bank(&mut self, data: u8) {
        self.bank = data & 7;
        // bank cannot be 0
        if self.bank == 0 {
            self.bank = 1;
        }
    }

    fn get_wram_bank(&self) -> u8 {
        0xF8 | self.bank
    }
}

pub struct Bus {
    cartridge: Cartridge,
    ppu: Ppu,
    wram: Wram,
    interrupts: Interrupts,
    timer: Timer,
    joypad: Joypad,
    serial: Serial,
    dma: DMA,
    hdma: HDMA,
    apu: Apu,
    hram: [u8; 127],
    boot_rom: BootRom,
    speed_controller: SpeedController,

    /// Used to track how many ppu cycles have elapsed
    /// when the frontend gets the elapsed value, its reset to 0
    elapsed_ppu_cycles: u32,
}

impl Bus {
    pub fn new_without_boot_rom(cartridge: Cartridge) -> Self {
        let cgb_mode = cartridge.is_cartridge_color();

        Self {
            cartridge,
            ppu: Ppu::new_skip_boot_rom(cgb_mode),
            wram: Wram::default(),
            interrupts: Interrupts::default(),
            timer: Timer::new_skip_boot_rom(),
            joypad: Joypad::default(),
            serial: Serial::default(),
            dma: DMA::default(),
            hdma: HDMA::default(),
            apu: Apu::new_skip_boot_rom(),
            hram: [0; 127],
            boot_rom: BootRom::default(),
            speed_controller: SpeedController::default(),

            elapsed_ppu_cycles: 0,
        }
    }

    pub fn new_with_boot_rom(cartridge: Cartridge, boot_rom_data: [u8; 0x900]) -> Self {
        let mut s = Self::new_without_boot_rom(cartridge);
        s.timer = Timer::default();
        s.ppu = Ppu::default();
        s.apu = Apu::default();
        s.boot_rom.data = boot_rom_data;
        s.boot_rom.enabled = true;
        s
    }

    pub fn screen_buffer(&self) -> &[u8] {
        self.ppu.screen_buffer()
    }

    pub fn audio_buffer(&mut self) -> Vec<f32> {
        self.apu.get_buffer()
    }

    pub fn press_joypad(&mut self, button: JoypadButton) {
        self.joypad.press_joypad(button);
    }

    pub fn release_joypad(&mut self, button: JoypadButton) {
        self.joypad.release_joypad(button);
    }

    pub fn elapsed_ppu_cycles(&mut self) -> u32 {
        std::mem::replace(&mut self.elapsed_ppu_cycles, 0)
    }
}

impl Bus {
    fn on_cpu_machine_cycle(&mut self) {
        let double_speed = self.speed_controller.current_speed() == Speed::Double;

        // will be 2 in normal speed and 1 in double speed
        let cpu_clocks_added = ((!double_speed) as u8) + 1;

        // will be 4 in normal speed and 2 in double speed
        let t_clocks = cpu_clocks_added * 2;
        self.elapsed_ppu_cycles += t_clocks as u32;

        // APU stays at the same speed even if CPU is in double speed
        self.ppu.clock(&mut self.interrupts, t_clocks);

        // APU stays at the same speed even if CPU is in double speed
        self.apu.clock(t_clocks);

        // HDMA stays at the same speed even if CPU is in double speed
        if self.hdma.is_transferreing(&self.ppu) {
            // this can be filled with 1 values in double mode and 2 in normal
            // mode
            let mut values = [0; 2];
            let addr = self.hdma.get_next_src_address();
            values[0] = self.read_not_ticked(addr, None);
            let mut values_len = 1;
            if !double_speed {
                values_len = 2;
                let addr = self.hdma.get_next_src_address();
                values[1] = self.read_not_ticked(addr, None);
            }

            self.hdma
                .transfer_clock(&mut self.ppu, &values[..values_len]);
        }

        // timer, DMA, and serial follow the CPU in speed and operates at double speed
        // if CPU is in double speed
        self.timer.clock_divider(&mut self.interrupts);
        self.joypad.update_interrupts(&mut self.interrupts);
        self.serial.clock(&mut self.interrupts);

        if self.dma.in_transfer {
            let value = self.read_not_ticked(self.dma.address, None);
            self.dma.transfer_clock(&mut self.ppu, value);
        }
    }

    fn read_not_ticked(&mut self, addr: u16, block_for_dma: Option<BusType>) -> u8 {
        let dma_value = if block_for_dma.is_some() {
            self.dma.current_value
        } else {
            0xFF
        };

        match (addr, block_for_dma) {
            (0x0000..=0x00FF, _) | (0x0200..=0x08FF, _) if self.boot_rom.enabled => {
                self.boot_rom.data[addr as usize]
            } // boot rom
            (0x0000..=0x7FFF, Some(BusType::External)) => dma_value, // external bus DMA conflict
            (0x0000..=0x3FFF, _) => self.cartridge.read_rom0(addr),  // rom0
            (0x4000..=0x7FFF, _) => self.cartridge.read_romx(addr),  // romx
            (0x8000..=0x9FFF, Some(BusType::Video)) => dma_value,    // video bus DMA conflict
            (0x8000..=0x9FFF, _) => self.ppu.read_vram(addr),        // ppu vram
            (0xA000..=0xDFFF, Some(BusType::External)) => dma_value, // external bus DMA conflict
            (0xA000..=0xBFFF, _) => self.cartridge.read_ram(addr),   // sram
            (0xC000..=0xCFFF, _) => self.wram.read_wram0(addr),      // wram0
            (0xD000..=0xDFFF, _) => self.wram.read_wramx(addr),      // wramx
            (0xE000..=0xFDFF, _) => self.read_not_ticked(0xC000 | (addr & 0x1FFF), block_for_dma), // echo
            (0xFE00..=0xFE9F, None) => self.ppu.read_oam(addr), // ppu oam
            (0xFEA0..=0xFEFF, _) => 0,                          // unused
            (0xFF00, _) => self.joypad.read_joypad(),           // joypad
            (0xFF01..=0xFF02, _) => self.serial.read_register(addr), // serial
            (0xFF04..=0xFF07, _) => self.timer.read_register(addr), // divider and timer
            (0xFF0F, _) => self.interrupts.read_interrupt_flags(), // interrupts flags
            (0xFF10..=0xFF3F, _) => self.apu.read_register(addr), // apu
            (0xFF40..=0xFF45, _) | (0xFF47..=0xFF4B, _) => self.ppu.read_register(addr), // ppu io registers
            (0xFF46, _) => self.dma.read(), // dma start
            (0xFF4D, _) => self.speed_controller.read_key1(), // speed
            (0xFF4F, _) => self.ppu.get_vram_bank(), // vram bank
            (0xFF50, _) => 0xFF,            // boot rom stop
            (0xFF51..=0xFF55, _) => self.hdma.read_register(addr), // hdma
            (0xFF56, _) => {
                todo!("RP port");
            }
            (0xFF68..=0xFF69, _) | (0xFF6A..=0xFF6B, _) => {
                self.ppu.read_color_register(addr) // ppu colors
            }
            (0xFF6C, _) => self.ppu.read_sprite_priority_mode(),
            (0xFF70, _) => self.wram.get_wram_bank(), // wram bank
            (0xFF80..=0xFFFE, _) => self.hram[addr as usize & 0x7F], // hram
            (0xFFFF, _) => self.interrupts.read_interrupt_enable(), //interrupts enable
            _ => 0xFF,
        }
    }

    fn write_not_ticked(&mut self, addr: u16, data: u8, block_for_dma: Option<BusType>) {
        match (addr, block_for_dma) {
            (0x0000..=0x7FFF, Some(BusType::External)) => {} // ignore writes
            (0x0000..=0x7FFF, _) => self.cartridge.write_to_bank_controller(addr, data), // rom0
            (0x8000..=0x9FFF, Some(BusType::Video)) => {}    // ignore writes
            (0x8000..=0x9FFF, _) => self.ppu.write_vram(addr, data), // ppu vram
            (0xA000..=0xDFFF, Some(BusType::External)) => {} // ignore writes
            (0xA000..=0xBFFF, _) => self.cartridge.write_ram(addr, data), // sram
            (0xC000..=0xCFFF, _) => self.wram.write_wram0(addr, data), // wram0
            (0xD000..=0xDFFF, _) => self.wram.write_wramx(addr, data), // wramx
            (0xE000..=0xFDFF, _) => {
                self.write_not_ticked(0xC000 | (addr & 0x1FFF), data, block_for_dma)
            } // echo
            (0xFE00..=0xFE9F, None) => self.ppu.write_oam(addr, data), // ppu oam
            (0xFEA0..=0xFEFF, _) => {}                       // unused
            (0xFF00, _) => self.joypad.write_joypad(data),   // joypad
            (0xFF01..=0xFF02, _) => self.serial.write_register(addr, data), // serial
            (0xFF04..=0xFF07, _) => self.timer.write_register(addr, data), // divider and timer
            (0xFF0F, _) => self.interrupts.write_interrupt_flags(data), // interrupts flags
            (0xFF10..=0xFF3F, _) => self.apu.write_register(addr, data), // apu
            (0xFF40..=0xFF45, _) | (0xFF47..=0xFF4B, _) => self.ppu.write_register(addr, data), // ppu io registers
            (0xFF46, _) => self.dma.start_dma(data), // dma start
            (0xFF4D, _) => self.speed_controller.write_key1(data), // speed
            (0xFF4F, _) => self.ppu.set_vram_bank(data), // vram bank
            (0xFF50, _) => {
                self.boot_rom.enabled = false;
                self.ppu
                    .update_cgb_mode(self.cartridge.is_cartridge_color());
            } // boot rom stop
            (0xFF51..=0xFF55, _) => self.hdma.write_register(addr, data), // hdma
            (0xFF56, _) => {
                todo!("RP port");
            }
            (0xFF68..=0xFF69, _) | (0xFF6A..=0xFF6B, _) => {
                self.ppu.write_color_register(addr, data); // ppu colors
            }
            (0xFF6C, _) => self.ppu.write_sprite_priority_mode(data),
            (0xFF70, _) => self.wram.set_wram_bank(data), // wram bank
            (0xFF80..=0xFFFE, _) => self.hram[addr as usize & 0x7F] = data, // hram
            (0xFFFF, _) => self.interrupts.write_interrupt_enable(data), // interrupts enable
            _ => {}
        }
    }
}

impl CpuBusProvider for Bus {
    /// each time the cpu reads, clock the components on the bus
    fn read(&mut self, addr: u16) -> u8 {
        let result = self.read_not_ticked(addr, self.dma.conflicting_bus);
        self.on_cpu_machine_cycle();
        result
    }

    /// each time the cpu writes, clock the components on the bus
    fn write(&mut self, addr: u16, data: u8) {
        self.write_not_ticked(addr, data, self.dma.conflicting_bus);
        self.on_cpu_machine_cycle();
    }

    // gets the interrupt type and remove it
    fn take_next_interrupt(&mut self) -> Option<InterruptType> {
        let int = self.interrupts.get_highest_interrupt();
        if let Some(int) = int {
            self.interrupts.acknowledge_interrupt(int);
        }
        int
    }

    fn peek_next_interrupt(&mut self) -> Option<InterruptType> {
        self.interrupts.get_highest_interrupt()
    }

    fn check_interrupts(&self) -> bool {
        self.interrupts.is_interrupts_available()
    }

    fn is_hdma_running(&mut self) -> bool {
        self.hdma.is_transferreing(&self.ppu)
    }

    fn is_speed_switch_prepared(&mut self) -> bool {
        self.speed_controller.preparing_switch()
    }

    fn commit_speed_switch(&mut self) {
        self.speed_controller.commit_speed_switch();
    }
}
