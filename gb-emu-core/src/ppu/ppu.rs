use super::fifo::Fifo;
use super::lcd::Lcd;
use super::sprite::Sprite;
use crate::memory::{InterruptManager, InterruptType};
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

/// The addresses to the VRAM are in 0x0000..0x2000 instead of the
/// original 0x8000..0xA000
impl LcdControl {
    fn display_enable(&self) -> bool {
        self.intersects(Self::DISPLAY_ENABLE)
    }

    fn window_tilemap(&self) -> u16 {
        if self.intersects(Self::WINDOW_TILEMAP) {
            0x1C00
        } else {
            0x1800
        }
    }

    fn window_enable(&self) -> bool {
        self.intersects(Self::WINDOW_ENABLE)
    }

    fn bg_window_pattern_table_base(&self) -> u16 {
        if self.intersects(Self::BG_WINDOW_PATTERN_TABLE) {
            0x0000
        } else {
            0x1000
        }
    }

    fn bg_window_pattern_table_block_1(&self) -> bool {
        !self.intersects(Self::BG_WINDOW_PATTERN_TABLE)
    }

    fn bg_tilemap(&self) -> u16 {
        if self.intersects(Self::BG_TILEMAP) {
            0x1C00
        } else {
            0x1800
        }
    }

    fn is_sprite_16(&self) -> bool {
        self.intersects(Self::SPRITE_SIZE)
    }

    fn sprite_enable(&self) -> bool {
        self.intersects(Self::SPRITE_ENABLE)
    }

    fn bg_window_priority(&self) -> bool {
        self.intersects(Self::BG_WINDOW_PRIORITY)
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

impl LcdStatus {
    fn lyc_ly_interrupt(&self) -> bool {
        self.intersects(Self::LYC_LY_INTERRUPT)
    }

    fn mode_2_oam_interrupt(&self) -> bool {
        self.intersects(Self::MODE_2_OAM_INTERRUPT)
    }

    fn mode_1_vblank_interrupt(&self) -> bool {
        self.intersects(Self::MODE_1_VBLANK_INTERRUPT)
    }

    fn mode_0_hblank_interrupt(&self) -> bool {
        self.intersects(Self::MODE_0_HBLANK_INTERRUPT)
    }

    fn coincidence_flag_set(&mut self, value: bool) {
        self.set(Self::COINCIDENCE_FLAG, value);
    }

    fn current_mode(&self) -> u8 {
        self.bits() & Self::MODE_FLAG.bits
    }

    fn current_mode_set(&mut self, data: u8) {
        self.clone_from(&Self::from_bits_truncate(
            (self.bits() & !0b11) | data & 0b11,
        ));
        assert!(self.current_mode() == data & 0b11);
    }
}

pub struct Ppu {
    lcd_control: LcdControl,
    lcd_status: LcdStatus,
    scroll_y: u8,
    scroll_x: u8,

    lyc: u8,
    bg_palette: u8,
    sprite_palette: [u8; 2],
    windows_y: u8,
    windows_x: u8,

    vram: [u8; 0x2000],
    oam: [Sprite; 40],
    // the sprites that got selected
    selected_oam: [Sprite; 10],
    selected_oam_size: u8,

    fetcher_x: u8,
    is_drawing_window: bool,
    window_y_counter: u8,

    bg_fifo: Fifo,
    sprite_fifo: Fifo,

    lcd: Lcd,

    cycle: u16,
    scanline: u8,
}

impl Default for Ppu {
    fn default() -> Self {
        let mut ppu = Self {
            lcd_control: LcdControl::from_bits_truncate(0),
            lcd_status: LcdStatus::from_bits_truncate(0),
            scroll_y: 0,
            scroll_x: 0,
            lyc: 0,
            bg_palette: 0,
            sprite_palette: [0; 2],
            windows_y: 0,
            windows_x: 0,
            vram: [0; 0x2000],
            oam: [Sprite::default(); 40],
            selected_oam: [Sprite::default(); 10],
            selected_oam_size: 0,
            fetcher_x: 0,
            is_drawing_window: false,
            window_y_counter: 0,
            bg_fifo: Fifo::default(),
            sprite_fifo: Fifo::default(),
            lcd: Lcd::default(),
            cycle: 0,
            scanline: 0,
        };
        ppu.reset();

        ppu
    }
}

impl Ppu {
    pub fn reset(&mut self) {
        // reset I/O registers
        self.write_register(0xff40, 0x91);
        self.write_register(0xff42, 0x00);
        self.write_register(0xff43, 0x00);
        self.write_register(0xff45, 0x00);
        self.write_register(0xff47, 0xFC);
        self.write_register(0xff48, 0xFF);
        self.write_register(0xff49, 0xFF);
        self.write_register(0xff4A, 0x00);
        self.write_register(0xff4B, 0x00);
    }

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
            0xFF41 => self.lcd_status.bits(),
            0xFF42 => self.scroll_y,
            0xFF43 => self.scroll_x,
            0xFF44 => self.scanline,
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
                .lcd_status
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

    pub fn screen_buffer(&self) -> Vec<u8> {
        self.lcd.screen_buffer()
    }

    pub fn clock<I: InterruptManager>(&mut self, interrupt_manager: &mut I) {
        // change modes depending on cycle
        match (self.scanline, self.cycle) {
            (0, 0) => {
                // change to mode 2 from mode 1
                self.lcd_status.current_mode_set(2);

                if self.lcd_status.mode_2_oam_interrupt() {
                    interrupt_manager.request_interrupt(InterruptType::LcdStat);
                }
            }
            (1..=143, 0) => {
                // change to mode 2 from mode 0
                self.lcd_status.current_mode_set(2);

                if self.lcd_status.mode_2_oam_interrupt() {
                    interrupt_manager.request_interrupt(InterruptType::LcdStat);
                }
            }
            (0..=143, 80) => {
                // change to mode 3 from mode 2
                self.lcd_status.current_mode_set(3);
            }
            (144, 0) => {
                // after drawing the screen reset the window y internal counter
                self.window_y_counter = 0;

                // change to mode 1 from mode 0
                self.lcd_status.current_mode_set(1);

                // FIXME: check if two interrupts are being fired
                interrupt_manager.request_interrupt(InterruptType::Vblank);
                if self.lcd_status.mode_1_vblank_interrupt() {
                    interrupt_manager.request_interrupt(InterruptType::LcdStat);
                }
            }
            _ => {}
        }

        if self.cycle == 0 {
            let flag = self.scanline == self.lyc;

            self.lcd_status.coincidence_flag_set(flag);

            if flag && self.lcd_status.lyc_ly_interrupt() {
                interrupt_manager.request_interrupt(InterruptType::LcdStat);
            }
        }

        match self.lcd_status.current_mode() {
            0 => {}
            1 => {}
            2 => {}
            3 => {
                if self.draw() && self.lcd_status.mode_0_hblank_interrupt() {
                    interrupt_manager.request_interrupt(InterruptType::LcdStat);
                }
            }
            _ => {}
        }

        // increment cycle
        self.cycle += 1;
        if self.cycle == 456 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline == 154 {
                self.scanline = 0;
                self.lcd.next_frame();
            }
        }
    }
}

impl Ppu {
    /// return true, if this is the last draw in the current scanline, and
    /// mode 0 is being activated
    fn draw(&mut self) -> bool {
        if self.lcd_control.window_enable()
            && !self.is_drawing_window
            && self.lcd.x() == self.windows_x.wrapping_sub(7)
            && self.scanline >= self.windows_y
        {
            // start window drawing
            self.bg_fifo.clear();
            self.fetcher_x = 0;
            self.is_drawing_window = true;
        }

        if self.bg_fifo.len() > 8 {
            self.lcd.push(self.bg_fifo.pop().color());
            if self.lcd.x() == 160 {
                self.lcd.next_line();
                // clear for the next line
                self.bg_fifo.clear();
                self.fetcher_x = 0;
                // change mode to 0 from 3
                self.lcd_status.current_mode_set(0);
                if self.is_drawing_window {
                    self.window_y_counter += 1;
                }
                self.is_drawing_window = false;
                return true;
            }
        }

        let tile = self.get_bg_window_tile();
        let bg_colors = self.get_bg_pattern(tile, self.scanline % 8);

        if self.bg_fifo.len() <= 8 {
            self.bg_fifo.push_bg(bg_colors);
            self.fetcher_x += 1;
        }

        false
    }

    fn get_bg_window_tile(&mut self) -> u8 {
        let tile_x;
        let tile_y;
        let tile_map;

        if self.is_drawing_window {
            tile_x = self.fetcher_x;
            tile_y = self.window_y_counter;

            tile_map = self.lcd_control.window_tilemap();
        } else {
            tile_x = ((self.scroll_x / 8) + self.fetcher_x) & 0x1F;
            tile_y = self.scanline.wrapping_add(self.scroll_y);

            tile_map = self.lcd_control.bg_tilemap();
        }

        self.get_tile(tile_map, tile_x, tile_y / 8)
    }

    // ignore for now
    fn is_in_window(&self, x: u8, y: u8) -> bool {
        false
    }

    fn get_tile(&self, tile_map: u16, tile_x: u8, tile_y: u8) -> u8 {
        let index = tile_y as usize * 32 + tile_x as usize;

        self.vram[tile_map as usize + index]
    }

    fn get_bg_pattern(&self, tile: u8, y: u8) -> [u8; 8] {
        let pattern_table = self.lcd_control.bg_window_pattern_table_base();

        let index = if self.lcd_control.bg_window_pattern_table_block_1() {
            let tile_index = (tile as i8 as i16 as u16).wrapping_mul(16);
            pattern_table.wrapping_add(tile_index)
        } else {
            pattern_table + (tile as u16) * 16
        } as usize;

        let low = self.vram[index + (y as usize) * 2];
        let high = self.vram[index + (y as usize) * 2 + 1];

        let mut result = [0; 8];

        for i in 0..8 {
            let bin_i = 7 - i;
            result[i] = ((high >> bin_i) & 1) << 1 | ((low >> bin_i) & 1);
        }

        result
    }
}
