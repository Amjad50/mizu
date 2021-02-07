use crate::ppu::Ppu;

#[derive(Default)]
pub struct Hdma {
    source_addr: u16,
    dest_addr: u16,
    length: u8,
    /// `true` if transfere during hblank only
    hblank_dma: bool,
    master_dma_active: bool,
    hblank_dma_active: bool,
    cached_ppu_hblank: bool,
}

impl Hdma {
    pub fn write_register(&mut self, addr: u16, data: u8) {
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
                    self.hblank_dma_active = false;
                    self.cached_ppu_hblank = false;
                    self.hblank_dma = data & 0x80 != 0;
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn read_register(&mut self, addr: u16) -> u8 {
        match addr {
            0xFF51..=0xFF54 => 0xFF,
            0xFF55 => (((!self.master_dma_active) as u8) << 7) | self.length, // control
            _ => unreachable!(),
        }
    }

    pub fn get_next_src_address(&mut self) -> u16 {
        let result = self.source_addr;
        self.source_addr += 1;
        result
    }

    pub fn transfer_clock(&mut self, ppu: &mut Ppu, values: &[u8]) {
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

    pub fn is_transferreing(&mut self, ppu: &Ppu) -> bool {
        let new_ppu_hblank_mode = ppu.get_current_mode() == 0;

        // Hblank DMA will be activated for transfer only when:
        // - bit 8 of register `0xFF55` is 1 (`self.hblank_dma`).
        // - we entered hblank by `cached_ppu_hblank == false` and `new_ppu_hblank_mode == true`
        //   meaning that the mode change from mode 3 to mode 1 (hblank)
        if self.hblank_dma && !self.cached_ppu_hblank && new_ppu_hblank_mode {
            self.hblank_dma_active = true;
        }
        self.cached_ppu_hblank = new_ppu_hblank_mode;

        self.master_dma_active && (!self.hblank_dma || self.hblank_dma_active)
    }
}

#[derive(Clone, Copy)]
pub enum BusType {
    // VRAM
    Video,
    // Cartridge ROM and SRAM, and WRAM
    External,
}

#[derive(Default)]
pub struct OamDma {
    conflicting_bus: Option<BusType>,
    current_value: u8,
    address: u16,
    in_transfer: bool,
    starting_delay: u8,
}

impl OamDma {
    pub fn write_register(&mut self, mut high_byte: u8) {
        // addresses changed from internal bus into external bus
        // For addresses in the range 0xFE00 to 0xFFFF since they are internal,
        // the adresses below them are used which are 0xDE00 to 0xDFFF. These
        // addresses are part of the WRAM
        if high_byte == 0xFE || high_byte == 0xFF {
            high_byte &= 0xDF;
        }

        self.address = (high_byte as u16) << 8;

        // 8 T-cycles here for delay instead of 4, this is to ensure correct
        // DMA timing
        self.starting_delay = 2;
        self.in_transfer = true;
    }

    pub fn read_register(&self) -> u8 {
        (self.address >> 8) as u8
    }

    pub fn in_transfer(&self) -> bool {
        self.in_transfer
    }

    pub fn get_next_address(&self) -> u16 {
        self.address
    }

    pub fn current_value(&self) -> u8 {
        self.current_value
    }

    pub fn conflicting_bus(&self) -> Option<BusType> {
        self.conflicting_bus
    }

    pub fn transfer_clock(&mut self, ppu: &mut Ppu, value: u8) {
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

            // TODO: check and make sure that DMA can write to OAM even if its blocked
            ppu.write_oam_no_lock(0xFE00 | (self.address & 0xFF), value);

            self.address += 1;
            if self.address & 0xFF == 0xA0 {
                self.in_transfer = false;
                self.conflicting_bus = None;
            }
        }
    }
}
