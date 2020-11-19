use super::sprite::Sprite;
use bitflags::bitflags;

bitflags! {
    struct LcdControl: u8 {
        const DISPLAY_ENABLE          = 1 << 7;
        const WINDOW_TILEMAP          = 1 << 6;
        const WINDOW_ENABLE           = 1 << 5;
        const BG_WINDOW_PATTERN_TABLE = 1 << 4;
        const BG_TILEMAP              = 1 << 3;
        const SPRITE_SIZE             = 1 << 2;
        const SPRITE_ENABLE           = 1 << 1;
        const BG_WINDOW_PRIORITY      = 1 << 0;
    }
}

bitflags! {
    struct LcdStatus: u8 {
        const LYC_LY_INTERRUPT        = 1 << 6;
        const MODE_2_OAM_INTERRUPT    = 1 << 5;
        const MODE_1_VBLANK_INTERRUPT = 1 << 4;
        const MODE_0_HBLANK_INTERRUPT = 1 << 3;
        const COINCIDENCE_FLAG        = 1 << 2;
        const MODE_FLAG               = 0b11;
    }
}

pub struct Ppu {
    lcd_control: LcdControl,
    lcdc_status: LcdStatus,
    scroll_y: u8,
    scroll_x: u8,
    lcdc_y: u8,
    lyc: u8,
    bg_palette: u8,
    sprite_palette: [u8; 2],
    windows_y: u8,
    windows_x: u8,

    vram: [u8; 0x2000],
    oam: [Sprite; 0x40],
}

impl Ppu {
    pub fn read_vram(&self, addr: u16) -> u8 {
        self.vram[addr as usize & 0x1FFF]
    }

    pub fn write_vram(&mut self, addr: u16, data: u8) {
        self.vram[addr as usize & 0x1FFF] = data;
    }

    pub fn read_oam(&self, addr: u16) -> u8 {
        let addr = addr & 0xFF;
        self.oam[addr as usize / 4].get_at_offset(addr as u8 % 4)
    }

    pub fn write_oam(&mut self, addr: u16, data: u8) {
        let addr = addr & 0xFF;
        self.oam[addr as usize / 4].set_at_offset(addr as u8 % 4, data);
    }

    pub fn read_register(&mut self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.lcd_control.bits(),
            0xFF41 => self.lcdc_status.bits(),
            0xFF42 => self.scroll_y,
            0xFF43 => self.scroll_x,
            0xFF44 => self.lcdc_y,
            0xFF45 => self.lyc,
            0xFF47 => self.bg_palette,
            0xFF48 => self.sprite_palette[0],
            0xFF49 => self.sprite_palette[1],
            0xFF4A => self.windows_y,
            0xFF4B => self.windows_x,
            _ => unreachable!(),
        }
    }

    pub fn write_register(&mut self, addr: u16, data: u8) {
        match addr {
            0xFF40 => self
                .lcd_control
                .clone_from(&LcdControl::from_bits_truncate(data)),
            0xFF41 => self
                .lcdc_status
                .clone_from(&LcdStatus::from_bits_truncate(data & 0x78)),
            0xFF42 => self.scroll_y = data,
            0xFF43 => self.scroll_x = data,
            0xFF44 => {
                // not writable??
            }
            0xFF45 => self.lyc = data,
            0xFF47 => self.bg_palette = data,
            0xFF48 => self.sprite_palette[0] = data,
            0xFF49 => self.sprite_palette[1] = data,
            0xFF4A => self.windows_y = data,
            0xFF4B => self.windows_x = data,
            _ => unreachable!(),
        }
    }
}
