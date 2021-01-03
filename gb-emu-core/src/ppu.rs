mod bg_attribs;
mod colors;
mod fifo;
mod lcd;
mod sprite;

use crate::memory::{InterruptManager, InterruptType};
use bg_attribs::BgAttribute;
use bitflags::bitflags;
use colors::{Color, ColorPalette, ColorPalettesCollection};
use fifo::{Fifo, SpritePriorityMode};
use lcd::Lcd;
use sprite::Sprite;

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

    fn sprite_size(&self) -> u8 {
        if self.intersects(Self::SPRITE_SIZE) {
            16
        } else {
            8
        }
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

#[derive(Default)]
struct Fetcher {
    delay_counter: u8,
    data: Option<([u8; 8], BgAttribute)>,
    x: u8,
}

impl Fetcher {
    fn cycle(&mut self) -> bool {
        self.delay_counter = self.delay_counter.saturating_sub(1);
        if self.delay_counter == 0 {
            self.reset();
            true
        } else {
            false
        }
    }

    fn push(&mut self, data: [u8; 8], attribs: BgAttribute) {
        self.x += 1;
        self.data = Some((data, attribs));
    }

    fn pop(&mut self) -> Option<([u8; 8], BgAttribute)> {
        self.data.take()
    }

    fn reset(&mut self) {
        self.delay_counter = 8;
    }
}

pub struct Ppu {
    lcd_control: LcdControl,
    lcd_status: LcdStatus,
    scroll_y: u8,
    scroll_x: u8,

    /// representation of `scanline`, these are separated because in scanline 153
    /// `ly` vaue will be 0 and `lyc` is affected by this
    ly: u8,
    lyc: u8,
    stat_interrupt_line: bool,
    bg_palette: u8,
    sprite_palette: [u8; 2],
    windows_y: u8,
    windows_x: u8,

    vram: [u8; 0x4000],
    vram_bank: u8,
    oam: [Sprite; 40],
    // the sprites that got selected
    selected_oam: [(Sprite, u8); 10],
    selected_oam_size: u8,

    bg_palettes: ColorPalettesCollection,
    sprite_palettes: ColorPalettesCollection,

    fine_scroll_x_discard: u8,
    fetcher: Fetcher,
    is_drawing_window: bool,
    window_y_counter: u8,

    fifo: Fifo,

    lcd: Lcd,

    cycle: u16,
    scanline: u8,

    /// track if the next frame is LCD still turning on
    lcd_turned_on: bool,

    sprite_priority_mode: SpritePriorityMode,
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            lcd_control: LcdControl::from_bits_truncate(0),
            // COINCIDENCE_FLAG flag set because LYC and LY are 0 at the beginning
            lcd_status: LcdStatus::from_bits_truncate(4),
            scroll_y: 0,
            scroll_x: 0,
            ly: 0,
            lyc: 0,
            stat_interrupt_line: false,
            bg_palette: 0xFC,
            sprite_palette: [0xFF; 2],
            windows_y: 0,
            windows_x: 0,
            vram: [0; 0x4000],
            vram_bank: 0,
            oam: [Sprite::default(); 40],
            selected_oam: [(Sprite::default(), 0xFF); 10],
            selected_oam_size: 0,
            bg_palettes: ColorPalettesCollection::default(),
            sprite_palettes: ColorPalettesCollection::default(),
            fine_scroll_x_discard: 0,
            fetcher: Fetcher::default(),
            is_drawing_window: false,
            window_y_counter: 0,
            fifo: Fifo::default(),
            lcd: Lcd::default(),
            cycle: 4,
            scanline: 0,
            lcd_turned_on: false,
            // CGB by default, the bootrom of the CGB will change
            // it if it detected the rom is DMG
            sprite_priority_mode: SpritePriorityMode::ByIndex,
        }
    }
}

impl Ppu {
    /// create a ppu instance that match the one the ppu would have when the
    /// boot_rom finishes execution
    pub fn new_skip_boot_rom() -> Self {
        let mut s = Self::default();
        // set I/O registers to the value which would have if boot_rom ran
        s.write_register(0xFF40, 0x91);
        s.write_register(0xFF42, 0x00);
        s.write_register(0xFF43, 0x00);
        s.write_register(0xFF45, 0x00);
        s.write_register(0xFF47, 0xFC);
        s.write_register(0xFF48, 0xFF);
        s.write_register(0xFF49, 0xFF);
        s.write_register(0xFF4A, 0x00);
        s.write_register(0xFF4B, 0x00);

        s.scanline = 153;
        s.cycle = 400;
        s.ly = 0;
        s.lcd_status.current_mode_set(1);

        s
    }

    pub fn read_vram(&self, addr: u16) -> u8 {
        self.vram[(self.vram_bank as usize * 0x2000) + (addr as usize & 0x1FFF)]
    }

    pub fn write_vram(&mut self, addr: u16, data: u8) {
        self.vram[(self.vram_bank as usize * 0x2000) + (addr as usize & 0x1FFF)] = data;
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
            0xFF41 => 0x80 | self.lcd_status.bits(),
            0xFF42 => self.scroll_y,
            0xFF43 => self.scroll_x,
            0xFF44 => self.ly,
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
            0xFF40 => {
                let old_disply_enable = self.lcd_control.display_enable();

                self.lcd_control
                    .clone_from(&LcdControl::from_bits_truncate(data));

                if !self.lcd_control.display_enable() && old_disply_enable {
                    if self.scanline < 144 {
                        println!(
                            "[WARN] Tried to turn off display outside VBLANK, hardware may get corrupted"
                        );
                    }

                    self.ly = 0;
                    self.cycle = 4;
                    self.scanline = 0;
                    self.lcd_status.current_mode_set(0);
                    self.lcd.clear();

                    // to function as soon as lcd is turned on
                    self.lcd_turned_on = true;
                }
            }
            0xFF41 => {
                self.lcd_status.clone_from(&LcdStatus::from_bits_truncate(
                    (self.lcd_status.bits() & !0x78) | (data & 0x78),
                ));
            }
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

    pub fn get_vram_bank(&self) -> u8 {
        self.vram_bank
    }

    pub fn set_vram_bank(&mut self, data: u8) {
        self.vram_bank = data & 1;
    }

    pub fn read_color_register(&mut self, addr: u16) -> u8 {
        match addr {
            0xFF68 => self.bg_palettes.read_index(),
            0xFF69 => self.bg_palettes.read_color_data(),
            0xFF6A => self.sprite_palettes.read_index(),
            0xFF6B => self.sprite_palettes.read_color_data(),
            _ => unreachable!(),
        }
    }

    pub fn write_color_register(&mut self, addr: u16, data: u8) {
        match addr {
            0xFF68 => self.bg_palettes.write_index(data),
            0xFF69 => self.bg_palettes.write_color_data(data),
            0xFF6A => self.sprite_palettes.write_index(data),
            0xFF6B => self.sprite_palettes.write_color_data(data),
            _ => unreachable!(),
        }
    }

    pub fn write_sprite_priority_mode(&mut self, data: u8) {
        self.sprite_priority_mode = if data & 1 == 0 {
            SpritePriorityMode::ByIndex
        } else {
            SpritePriorityMode::ByCoord
        };
    }

    pub fn read_sprite_priority_mode(&self) -> u8 {
        if let SpritePriorityMode::ByIndex = self.sprite_priority_mode {
            1
        } else {
            0
        }
    }

    pub fn screen_buffer(&self) -> &[u8] {
        self.lcd.screen_buffer()
    }

    // clocks the PPU 4 times in a row
    pub fn clock_4_times<I: InterruptManager>(&mut self, interrupt_manager: &mut I) {
        let mut new_stat_int_happened = false;

        if !self.lcd_control.display_enable() {
            return;
        }

        // change modes depending on cycle
        match (self.scanline, self.cycle) {
            (0, 4) => {
                // if the lcd is not just turning on, then switch to mode 2,
                // when the lcd is turning on it will start here, but will keep
                // mode 0
                if !self.lcd_turned_on {
                    // change to mode 2 from mode 1
                    self.lcd_status.current_mode_set(2);
                }
            }
            (1..=143, 0) => {
                // change to mode 2 from mode 0
                self.lcd_status.current_mode_set(2);
            }
            (0..=143, 80) => {
                // change to mode 3 from mode 2
                self.fine_scroll_x_discard = self.scroll_x & 0x7;
                self.fetcher.reset();
                self.lcd_status.current_mode_set(3);
            }
            (144, 4) => {
                // change to mode 1 from mode 0
                self.lcd_status.current_mode_set(1);
                self.enter_vblank();

                interrupt_manager.request_interrupt(InterruptType::Vblank);
            }
            _ => {}
        }

        match self.lcd_status.current_mode() {
            0 => {
                new_stat_int_happened =
                    new_stat_int_happened || self.lcd_status.mode_0_hblank_interrupt();
            }
            // TODO: check if apply to 144 or to 144-153
            1 if self.cycle == 4 && self.scanline == 144 => {
                // special: also mode 2 interrupt if enabled
                new_stat_int_happened = new_stat_int_happened
                    || self.lcd_status.mode_1_vblank_interrupt()
                    || self.lcd_status.mode_2_oam_interrupt();
            }
            1 => {
                new_stat_int_happened =
                    new_stat_int_happened || self.lcd_status.mode_1_vblank_interrupt();
            }
            2 if self.cycle == 0 => {
                new_stat_int_happened =
                    new_stat_int_happened || self.lcd_status.mode_2_oam_interrupt();
            }
            2 if self.cycle == 4 => {
                self.load_selected_sprites_oam();

                // execluded from the spcial case where mode2 interrupt happen
                // at cycle 0, here it happens at cycle 4
                if self.scanline == 0 {
                    new_stat_int_happened =
                        new_stat_int_happened || self.lcd_status.mode_2_oam_interrupt();
                }
            }
            3 => {
                for _ in 0..4 {
                    if self.draw() {
                        // change mode to 0 from 3
                        self.lcd_status.current_mode_set(0);
                        self.enter_hblank();
                        break;
                    }
                }
            }
            _ => {}
        }

        let new_coincidence = self.ly == self.lyc;
        self.lcd_status.coincidence_flag_set(new_coincidence);

        new_stat_int_happened =
            new_stat_int_happened || (new_coincidence && self.lcd_status.lyc_ly_interrupt());

        if new_stat_int_happened && !self.stat_interrupt_line {
            interrupt_manager.request_interrupt(InterruptType::LcdStat);
        }

        self.stat_interrupt_line = new_stat_int_happened;

        if self.scanline == 153 && self.cycle == 4 {
            self.ly = 0;
        }

        // increment cycle
        self.cycle += 4;
        if self.cycle == 456 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline == 154 {
                self.scanline = 0;
                self.lcd.next_line();
            }
            self.ly = self.scanline;
        }

        self.lcd_turned_on = false;
    }
}

impl Ppu {
    /// return true, if this is the last draw in the current scanline, and
    /// mode 0 is being activated
    fn draw(&mut self) -> bool {
        self.try_enter_window();

        if self.fetcher.cycle() {
            let (bg, attribs) = self.fetch_bg();
            self.fetcher.push(bg, attribs);
        }

        if self.fifo.len() <= 8 {
            if let Some((pixels, attribs)) = self.fetcher.pop() {
                self.fifo.push_bg(
                    pixels,
                    self.bg_palettes.get_palette(attribs.palette()),
                    attribs.priority(),
                );
            }
        }

        if self.fifo.len() > 8 {
            self.try_add_sprite();

            if self.fine_scroll_x_discard > 0 {
                self.fine_scroll_x_discard -= 1;
                self.fifo.pop();
            } else {
                let (color, palette) = self.fifo.pop();

                self.lcd.push(self.get_color(color, palette), self.scanline);

                if self.lcd.x() == 160 {
                    return true;
                }
            }
        }

        false
    }

    fn get_color(&self, color_index: u8, palette: ColorPalette) -> Color {
        palette.get_color(color_index)
    }

    fn get_bg_window_tile_and_y(&mut self) -> (u8, BgAttribute, u8) {
        let tile_x;
        let tile_y;
        let tile_map;

        if self.is_drawing_window {
            tile_x = self.fetcher.x;
            tile_y = self.window_y_counter;

            tile_map = self.lcd_control.window_tilemap();
        } else {
            tile_x = ((self.scroll_x / 8) + self.fetcher.x) & 0x1F;
            tile_y = self.scanline.wrapping_add(self.scroll_y);

            tile_map = self.lcd_control.bg_tilemap();
        }

        let tile_index = self.get_tile_index(tile_x, tile_y / 8);
        let tile = self.vram[(tile_map + tile_index) as usize];
        let tile_attribs = BgAttribute::new(self.vram[0x2000 + (tile_map + tile_index) as usize]);

        (tile, tile_attribs, tile_y)
    }

    fn get_tile_index(&self, tile_x: u8, tile_y: u8) -> u16 {
        tile_y as u16 * 32 + tile_x as u16
    }

    fn get_bg_pattern(&self, tile: u8, y: u8, bank: u8) -> [u8; 8] {
        let pattern_table = self.lcd_control.bg_window_pattern_table_base();

        let index = if self.lcd_control.bg_window_pattern_table_block_1() {
            let tile_index = (tile as i8 as i16 as u16).wrapping_mul(16);
            pattern_table.wrapping_add(tile_index)
        } else {
            pattern_table + (tile as u16) * 16
        };

        self.get_tile_pattern_from_index(index, y, bank)
    }

    fn get_sprite_pattern(&self, mut tile: u8, y: u8, bank: u8) -> [u8; 8] {
        if self.lcd_control.sprite_size() == 16 {
            tile &= 0xFE;
        }

        let index = tile as u16 * 16;

        self.get_tile_pattern_from_index(index, y, bank)
    }

    fn get_tile_pattern_from_index(&self, index: u16, y: u8, bank: u8) -> [u8; 8] {
        let index = index as usize;

        let low = self.vram[(bank as usize * 0x2000) + index + ((y as usize) * 2)];
        let high = self.vram[(bank as usize * 0x2000) + index + ((y as usize) * 2 + 1)];

        let mut result = [0; 8];

        for (i, result_item) in result.iter_mut().enumerate() {
            let bin_i = 7 - i;
            *result_item = ((high >> bin_i) & 1) << 1 | ((low >> bin_i) & 1);
        }

        result
    }

    fn fetch_bg(&mut self) -> ([u8; 8], BgAttribute) {
        let (tile, attribs, y) = self.get_bg_window_tile_and_y();

        let y = if attribs.is_vertical_flip() {
            7 - (y % 8)
        } else {
            y % 8
        };

        let mut pattern = self.get_bg_pattern(tile, y, attribs.bank());

        if attribs.is_horizontal_flip() {
            pattern.reverse();
        }

        (pattern, attribs)
    }

    fn load_selected_sprites_oam(&mut self) {
        let mut count = 0;
        for (i, &sprite) in self.oam.iter().enumerate() {
            // in range
            if self.scanline.wrapping_sub(sprite.screen_y()) < self.lcd_control.sprite_size() {
                self.selected_oam[count] = (sprite, i as u8);
                count += 1;

                if count == 10 {
                    break;
                }
            }
        }
        self.selected_oam_size = count as u8;
    }

    fn try_add_sprite(&mut self) {
        if self.lcd_control.sprite_enable() {
            for (sprite, index) in self
                .selected_oam
                .iter()
                .take(self.selected_oam_size as usize)
            {
                // the x index of the sprite is of the left of the display
                let left_out_of_bounds = self.lcd.x() == 0 && sprite.x() < 8;

                if self.lcd.x() == sprite.screen_x() || left_out_of_bounds {
                    let mut y = self.scanline.wrapping_sub(sprite.screen_y());
                    if sprite.y_flipped() {
                        y = (self.lcd_control.sprite_size() - 1) - y;
                    }

                    let mut colors = self.get_sprite_pattern(sprite.tile(), y, sprite.bank());

                    if sprite.x_flipped() {
                        colors.reverse();
                    }

                    if left_out_of_bounds {
                        let to_shift = 8u8.saturating_sub(sprite.x()) as usize;

                        colors.rotate_left(to_shift);
                        for i in &mut colors[8 - to_shift..] {
                            *i = 0;
                        }
                    }

                    // TODO: fix all these parameters
                    self.fifo.mix_sprite(
                        colors,
                        self.sprite_palettes.get_palette(sprite.cgb_palette()),
                        *index,
                        self.sprite_priority_mode,
                        sprite.bg_priority(),
                        !self.lcd_control.bg_window_priority(),
                    )
                }
            }
        }
    }

    fn try_enter_window(&mut self) {
        if self.lcd_control.window_enable()
            && !self.is_drawing_window
                // handle if window's x is less than 7
            && (self.lcd.x() == self.windows_x.wrapping_sub(7) || (self.lcd.x() == 0 && self.windows_x < 7))
            && self.scanline >= self.windows_y
        {
            // override the scroll_x if:
            // - the window_x is lower than 7; to discard the bits *from* the window
            // - there is already fine scroll; to reset the scrolling and for the window
            //   to stay in place
            if self.windows_x < 7 || self.fine_scroll_x_discard != 0 {
                self.fine_scroll_x_discard = 7 - self.windows_x;
            }
            // start window drawing
            self.fifo.clear();
            self.fetcher.x = 0;
            self.is_drawing_window = true;
        }
    }

    /// Ending stuff for mode 3
    fn enter_hblank(&mut self) {
        self.lcd.next_line();
        // clear for the next line
        self.fifo.clear();
        self.fetcher.x = 0;
        if self.is_drawing_window {
            self.window_y_counter += 1;
        }
        self.is_drawing_window = false;
    }

    fn enter_vblank(&mut self) {
        // after drawing the screen reset the window y internal counter
        self.window_y_counter = 0;
    }
}
