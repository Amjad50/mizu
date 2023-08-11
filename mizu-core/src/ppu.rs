#[macro_use]
mod colors;
mod bg_attribs;
mod fifo;
mod lcd;
mod sprite;

use bitflags::bitflags;
use save_state::Savable;

use crate::memory::{InterruptManager, InterruptType};
use crate::GameBoyConfig;

use bg_attribs::BgAttribute;
use colors::{Color, ColorPalette, ColorPalettesCollection};
use fifo::{BgFifo, SpriteFifo, SpritePriorityMode};
use lcd::Lcd;
use sprite::{SelectedSprite, Sprite};

bitflags! {
    #[derive(Savable)]
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
    #[derive(Savable)]
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

#[derive(Default, Savable)]
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

#[derive(Savable)]
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
    dmg_bg_palette: u8,
    dmg_sprite_palettes: [u8; 2],
    windows_y: u8,
    windows_x: u8,

    vram: [u8; 0x4000],
    vram_bank: u8,
    oam: [Sprite; 40],
    // the sprites that got selected
    selected_oam: [SelectedSprite; 10],
    selected_oam_size: u8,

    cgb_bg_palettes: ColorPalettesCollection,
    cgb_sprite_palettes: ColorPalettesCollection,

    fine_scroll_x_discard: u8,
    fetcher: Fetcher,
    is_drawing_window: bool,
    window_y_counter: u8,

    bg_fifo: BgFifo,
    sprite_fifo: SpriteFifo,

    lcd: Lcd,

    cycle: u16,
    scanline: u8,

    mode_3_end_cycle: u16,

    /// track if the next frame is LCD still turning on
    lcd_turned_on: bool,

    sprite_priority_mode: SpritePriorityMode,

    is_cgb_mode: bool,

    config: GameBoyConfig,
}

impl Ppu {
    pub fn new(config: GameBoyConfig) -> Self {
        let mut cgb_bg_palettes = ColorPalettesCollection::default();
        let mut cgb_sprite_palettes = ColorPalettesCollection::default();

        if config.is_dmg {
            cgb_bg_palettes.set_palette(
                0,
                ColorPalette::new([
                    color!(31, 31, 31),
                    color!(21, 21, 21),
                    color!(10, 10, 10),
                    color!(0, 0, 0),
                ]),
            );

            cgb_sprite_palettes.set_palette(
                0,
                ColorPalette::new([
                    color!(31, 31, 31),
                    color!(21, 21, 21),
                    color!(10, 10, 10),
                    color!(0, 0, 0),
                ]),
            );

            cgb_sprite_palettes.set_palette(
                1,
                ColorPalette::new([
                    color!(31, 31, 31),
                    color!(21, 21, 21),
                    color!(10, 10, 10),
                    color!(0, 0, 0),
                ]),
            );
        }

        let sprite_priority_mode = if config.is_dmg {
            SpritePriorityMode::ByCoord
        } else {
            SpritePriorityMode::ByIndex
        };

        Self {
            lcd_control: LcdControl::from_bits_truncate(0),
            // COINCIDENCE_FLAG flag set because LYC and LY are 0 at the beginning
            lcd_status: LcdStatus::from_bits_truncate(4),
            scroll_y: 0,
            scroll_x: 0,
            ly: 0,
            lyc: 0,
            stat_interrupt_line: false,
            dmg_bg_palette: 0xFC,
            dmg_sprite_palettes: [0xFF; 2],
            windows_y: 0,
            windows_x: 0,
            vram: [0; 0x4000],
            vram_bank: 0,
            oam: [Sprite::default(); 40],
            selected_oam: [SelectedSprite::default(); 10],
            selected_oam_size: 0,
            cgb_bg_palettes,
            cgb_sprite_palettes,
            fine_scroll_x_discard: 0,
            fetcher: Fetcher::default(),
            is_drawing_window: false,
            window_y_counter: 0,
            bg_fifo: BgFifo::default(),
            sprite_fifo: SpriteFifo::new(sprite_priority_mode),
            lcd: Lcd::default(),
            cycle: 4,
            scanline: 0,
            mode_3_end_cycle: 0,
            lcd_turned_on: false,
            // CGB by default, the bootrom of the CGB will change
            // it if it detected the rom is DMG
            sprite_priority_mode,
            is_cgb_mode: !config.is_dmg,

            config,
        }
    }
    /// create a ppu instance that match the one the ppu would have when the
    /// boot_rom finishes execution
    pub fn new_skip_boot_rom(mut cgb_mode: bool, config: GameBoyConfig) -> Self {
        let mut s = Self::new(config);
        // set I/O registers to the value which would have if boot_rom ran
        s.write_lcd_control(0x91);
        s.write_scroll_y(0x00);
        s.write_scroll_x(0x00);
        s.write_lyc(0x00);
        s.write_dmg_bg_palette(0xFC);
        s.write_dmg_sprite_palettes(0, 0xFF);
        s.write_dmg_sprite_palettes(1, 0xFF);
        s.write_window_y(0x00);
        s.write_window_x(0x00);

        if config.is_dmg {
            cgb_mode = false;
        }

        // palettes for DMG only
        if !cgb_mode {
            s.cgb_bg_palettes.set_palette(
                0,
                ColorPalette::new([
                    color!(31, 31, 31),
                    color!(21, 21, 21),
                    color!(10, 10, 10),
                    color!(0, 0, 0),
                ]),
            );

            s.cgb_sprite_palettes.set_palette(
                0,
                ColorPalette::new([
                    color!(31, 31, 31),
                    color!(21, 21, 21),
                    color!(10, 10, 10),
                    color!(0, 0, 0),
                ]),
            );

            s.cgb_sprite_palettes.set_palette(
                1,
                ColorPalette::new([
                    color!(31, 31, 31),
                    color!(21, 21, 21),
                    color!(10, 10, 10),
                    color!(0, 0, 0),
                ]),
            );
            s.sprite_priority_mode = SpritePriorityMode::ByCoord;
        }

        s.sprite_fifo
            .update_sprite_priority_mode(s.sprite_priority_mode);

        s.is_cgb_mode = cgb_mode;

        s.scanline = 153;
        s.cycle = 400;
        s.ly = 0;
        s.lcd_status.current_mode_set(1);

        s
    }

    pub fn read_vram(&self, addr: u16) -> u8 {
        self.read_vram_banked(self.vram_bank, addr)
    }

    pub fn write_vram(&mut self, addr: u16, data: u8) {
        // here since this is the only place vram is written to, no need
        // to make another function `write_vram_banked`
        let offset = addr as usize & 0x1FFF;
        let bank_start = self.vram_bank as usize * 0x2000;
        self.vram[bank_start + offset] = data;
    }

    pub fn read_oam(&self, addr: u16) -> u8 {
        if !self.is_oam_locked() {
            self.read_oam_no_lock(addr)
        } else {
            0xFF
        }
    }

    pub fn write_oam(&mut self, addr: u16, data: u8) {
        if !self.is_oam_locked() {
            self.write_oam_no_lock(addr, data);
        }
    }

    /// This is used for DMA only, as it can write when OAM is normally blocked
    pub fn write_oam_no_lock(&mut self, addr: u16, data: u8) {
        let addr = addr & 0xFF;
        self.oam[addr as usize / 4].set_at_offset(addr as u8 % 4, data);
    }

    /// In OAM bug on write:
    /// - The first word in the row is replaced with this bitwise expression:
    ///   `((a ^ c) & (b ^ c)) ^ c`, where a is the original value of that word,
    ///   b is the first word in the preceding row, and c is the third word
    ///   in the preceding row.
    /// - The last three words are copied from the last three words
    ///   in the preceding row.
    ///
    /// If two writes happen in the same M-cycle by inc/dec and a write,
    /// then only one is considered.
    pub fn oam_bug_write(&mut self) {
        let oam_row = self.check_get_oam_bug_row();

        if oam_row > 0 {
            let a = self.read_oam_word_no_lock(oam_row * 8);
            let b = self.read_oam_word_no_lock((oam_row - 1) * 8);
            let c = self.read_oam_word_no_lock((oam_row - 1) * 8 + 4);
            let result = ((a ^ c) & (b ^ c)) ^ c;

            self.write_oam_word_no_lock(oam_row * 8, result);
            for i in (2..=6).step_by(2) {
                let data = self.read_oam_word_no_lock((oam_row - 1) * 8 + i);
                self.write_oam_word_no_lock(oam_row * 8 + i, data);
            }
        }
    }

    /// In OAM bug on read:
    /// same as read `[oam_bug_write]`, with just a difference in the binary
    /// operation, `b | (a & c)` is used here
    pub fn oam_bug_read(&mut self) {
        let oam_row = self.check_get_oam_bug_row();

        if oam_row > 0 {
            let a = self.read_oam_word_no_lock(oam_row * 8);
            let b = self.read_oam_word_no_lock((oam_row - 1) * 8);
            let c = self.read_oam_word_no_lock((oam_row - 1) * 8 + 4);
            let result = b | (a & c);

            self.write_oam_word_no_lock(oam_row * 8, result);
            for i in (2..=6).step_by(2) {
                let data = self.read_oam_word_no_lock((oam_row - 1) * 8 + i);
                self.write_oam_word_no_lock(oam_row * 8 + i, data);
            }
        }
    }

    /// In some cases inc/dec and a read might happen in the same cycle.
    /// In this case, a strange behaviour occur:
    /// - This corruption will not happen if the accessed row is one of the
    ///   first four, as well as if it's the last row:
    ///     - The first word in the row preceding the currently accessed row
    ///       is replaced with the following bitwise expression:
    ///       `(b & (a | c | d)) | (a & c & d)` where a is the first word two
    ///       rows before the currently accessed row, b is the first word in
    ///       the preceding row (the word being corrupted), c is the first word
    ///       in the currently accessed row, and d is the third word in the
    ///       preceding row.
    ///     - The contents of the preceding row is copied (after the corruption
    ///       of the first word in it) both to the currently accessed row and to
    ///       two rows before the currently accessed row
    /// - Regardless of wether the previous corruption occurred or not,
    ///   a normal read corruption is then applied.
    pub fn oam_bug_read_write(&mut self) {
        let oam_row = self.check_get_oam_bug_row();

        if oam_row > 3 && oam_row < 19 {
            let a = self.read_oam_word_no_lock((oam_row - 2) * 8);
            let b = self.read_oam_word_no_lock((oam_row - 1) * 8);
            let c = self.read_oam_word_no_lock(oam_row * 8);
            let d = self.read_oam_word_no_lock((oam_row - 1) * 8 + 4);
            let result = (b & (a | c | d)) | (a & c & d);

            self.write_oam_word_no_lock((oam_row - 1) * 8, result);

            for i in (0..=6).step_by(2) {
                let data = self.read_oam_word_no_lock((oam_row - 1) * 8 + i);
                self.write_oam_word_no_lock(oam_row * 8 + i, data);
                self.write_oam_word_no_lock((oam_row - 2) * 8 + i, data);
            }
        }

        self.oam_bug_read();
    }

    pub fn read_lcd_control(&self) -> u8 {
        self.lcd_control.bits()
    }

    pub fn write_lcd_control(&mut self, data: u8) {
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

    pub fn read_lcd_status(&self) -> u8 {
        0x80 | self.lcd_status.bits()
    }

    pub fn write_lcd_status(&mut self, data: u8) {
        self.lcd_status.clone_from(&LcdStatus::from_bits_truncate(
            (self.lcd_status.bits() & !0x78) | (data & 0x78),
        ));
    }

    pub fn read_scroll_y(&self) -> u8 {
        self.scroll_y
    }

    pub fn write_scroll_y(&mut self, data: u8) {
        self.scroll_y = data;
    }

    pub fn read_scroll_x(&self) -> u8 {
        self.scroll_x
    }

    pub fn write_scroll_x(&mut self, data: u8) {
        self.scroll_x = data;
    }

    pub fn read_ly(&self) -> u8 {
        self.ly
    }

    pub fn write_ly(&mut self, _data: u8) {}

    pub fn read_lyc(&self) -> u8 {
        self.lyc
    }

    pub fn write_lyc(&mut self, data: u8) {
        self.lyc = data;
    }

    pub fn read_dmg_bg_palette(&self) -> u8 {
        self.dmg_bg_palette
    }

    pub fn write_dmg_bg_palette(&mut self, data: u8) {
        self.dmg_bg_palette = data;
    }

    pub fn read_dmg_sprite_palettes(&self, index: u8) -> u8 {
        self.dmg_sprite_palettes[index as usize & 1]
    }

    pub fn write_dmg_sprite_palettes(&mut self, index: u8, data: u8) {
        self.dmg_sprite_palettes[index as usize & 1] = data;
    }

    pub fn read_window_y(&self) -> u8 {
        self.windows_y
    }

    pub fn write_window_y(&mut self, data: u8) {
        self.windows_y = data;
    }

    pub fn read_window_x(&self) -> u8 {
        self.windows_x
    }

    pub fn write_window_x(&mut self, data: u8) {
        self.windows_x = data;
    }

    pub fn read_vram_bank(&self) -> u8 {
        0xFE | self.vram_bank
    }

    pub fn write_vram_bank(&mut self, data: u8) {
        self.vram_bank = data & 1;
    }

    pub fn read_cgb_bg_palettes_index(&self) -> u8 {
        self.cgb_bg_palettes.read_index()
    }

    pub fn write_cgb_bg_palettes_index(&mut self, data: u8) {
        self.cgb_bg_palettes.write_index(data);
    }

    pub fn read_cgb_bg_palettes_data(&self) -> u8 {
        self.cgb_bg_palettes.read_color_data()
    }

    pub fn write_cgb_bg_palettes_data(&mut self, data: u8) {
        self.cgb_bg_palettes.write_color_data(data);
    }

    pub fn read_cgb_sprite_palettes_index(&self) -> u8 {
        self.cgb_sprite_palettes.read_index()
    }

    pub fn write_cgb_sprite_palettes_index(&mut self, data: u8) {
        self.cgb_sprite_palettes.write_index(data);
    }

    pub fn read_cgb_sprite_palettes_data(&self) -> u8 {
        self.cgb_sprite_palettes.read_color_data()
    }

    pub fn write_cgb_sprite_palettes_data(&mut self, data: u8) {
        self.cgb_sprite_palettes.write_color_data(data);
    }

    pub fn write_sprite_priority_mode(&mut self, data: u8) {
        self.sprite_priority_mode = if data & 1 == 0 {
            SpritePriorityMode::ByIndex
        } else {
            SpritePriorityMode::ByCoord
        };

        self.sprite_fifo
            .update_sprite_priority_mode(self.sprite_priority_mode);
    }

    pub fn read_sprite_priority_mode(&self) -> u8 {
        0xFE | if let SpritePriorityMode::ByIndex = self.sprite_priority_mode {
            0
        } else {
            1
        }
    }

    pub fn update_cgb_mode(&mut self, cgb_mode: bool) {
        self.is_cgb_mode = cgb_mode && !self.config.is_dmg;
    }

    pub fn get_current_mode(&self) -> u8 {
        self.lcd_status.current_mode()
    }

    pub fn screen_buffer(&self) -> &[u8] {
        self.lcd.screen_buffer()
    }

    pub fn enter_stop_mode(&mut self) {
        if self.config.is_dmg {
            self.lcd.clear();
        } else {
            // FIXME: the bus is not letting the ppu run during stop mode
            //  but in CGB, the ppu keeps running but because the vram
            //  is blocked it reads all black
            if self.get_current_mode() != 3 {
                // black
                self.lcd.fill(color!(0, 0, 0))
            }
        }
    }

    #[cfg(test)]
    pub fn raw_screen_buffer(&self) -> &[u8] {
        self.lcd.raw_screen_buffer()
    }

    pub fn clock<I: InterruptManager>(&mut self, interrupt_manager: &mut I, clocks: u8) {
        let mut new_stat_int_happened = false;

        if !self.lcd_control.display_enable() {
            return;
        }

        // change modes depending on cycle
        match (self.scanline, self.cycle) {
            (0, 0) => {
                // set to mode 2 on frame start
                // This will not be executed if the lcd is just turned on, since
                // it starts at cycle 4

                self.mode_3_end_cycle = 0;
                self.lcd_status.current_mode_set(2);
            }
            (0, 4) => {
                // if the lcd is not just turning on, then switch to mode 2,
                // when the lcd is turning on it will start here, but will keep
                // mode 0
                if !self.lcd_turned_on {
                    // change to mode 2 from mode 1
                    self.mode_3_end_cycle = 0;
                    self.lcd_status.current_mode_set(2);
                }
            }
            (1..=143, 0) => {
                // change to mode 2 from mode 0
                self.mode_3_end_cycle = 0;
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
                self.mode_3_end_cycle = 0;

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
            1 if self.cycle == 4 && self.scanline == 144 && self.config.is_dmg => {
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
                // FIXME: check mode 2 interrupt timing for DMG and CGB
                if !self.config.is_dmg {
                    new_stat_int_happened =
                        new_stat_int_happened || self.lcd_status.mode_2_oam_interrupt();
                }
            }
            2 if self.cycle == 4 => {
                self.load_selected_sprites_oam();

                // FIXME: check mode 2 interrupt timing for DMG and CGB
                if self.config.is_dmg {
                    new_stat_int_happened =
                        new_stat_int_happened || self.lcd_status.mode_2_oam_interrupt();
                }

                // execluded from the spcial case where mode2 interrupt happen
                // at cycle 0, here it happens at cycle 4
                if self.scanline == 0 {
                    new_stat_int_happened =
                        new_stat_int_happened || self.lcd_status.mode_2_oam_interrupt();
                }
            }
            3 => {
                for _ in 0..clocks {
                    if self.draw() {
                        // change mode to 0 from 3
                        self.lcd_status.current_mode_set(0);
                        self.mode_3_end_cycle = self.cycle;
                        self.enter_hblank();
                        break;
                    }
                }
            }
            _ => {}
        }

        // In CGB, the mode2 interrupt happens before the vblank interrupt
        if self.cycle == 0 && self.scanline == 144 && !self.config.is_dmg {
            // special: also mode 2 interrupt if enabled
            new_stat_int_happened = new_stat_int_happened
                || self.lcd_status.mode_1_vblank_interrupt()
                || self.lcd_status.mode_2_oam_interrupt();
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
        self.cycle += clocks as u16;
        if self.cycle >= 456 {
            self.cycle -= 456;
            self.scanline += 1;
            if self.scanline == 154 {
                self.lcd.switch_buffers();
                self.scanline = 0;
                self.lcd.next_line();
            }
            self.ly = self.scanline;
        }

        self.lcd_turned_on = false;
    }
}

impl Ppu {
    /// The OAM is locked during mode 2 (OAM Scan), mode 3 (Rendering)
    /// The lock is extended until 8 dots after the mode 3 is over
    fn is_oam_locked(&self) -> bool {
        // OAM is not locked in VBlank
        self.get_current_mode() != 1
            // OAM is locked in mode 2 and 3 with an extend of 8 clocks afterwards
            && ((2..=3).contains(&self.get_current_mode())
                || (self.mode_3_end_cycle != 0 && self.mode_3_end_cycle + 8 > self.cycle))
    }

    fn read_oam_no_lock(&self, addr: u16) -> u8 {
        let addr = addr & 0xFF;
        self.oam[addr as usize / 4].get_at_offset(addr as u8 % 4)
    }

    fn read_oam_word_no_lock(&self, offset: u8) -> u16 {
        assert!(offset < 0x9F);

        let addr = 0xFE00 | offset as u16;
        // Note: order of high and low does not matter
        // because these are used in oam_bug only which involve bitwise
        // operations and not normal arithmetic operations
        let low = self.read_oam_no_lock(addr) as u16;
        let high = self.read_oam_no_lock(addr + 1) as u16;

        (high << 8) | low
    }

    fn write_oam_word_no_lock(&mut self, offset: u8, data: u16) {
        assert!(offset < 0x9F);

        let addr = 0xFE00 | offset as u16;
        // Note: order does not matter, but it must follow `read_oam_word_no_lock`
        let low = data as u8;
        let high = (data >> 8) as u8;

        self.write_oam_no_lock(addr, low);
        self.write_oam_no_lock(addr + 1, high);
    }

    fn check_get_oam_bug_row(&self) -> u8 {
        if self.get_current_mode() == 2 {
            assert!(self.cycle > 0);

            let cycle = self.cycle - 4;

            (cycle / 4) as u8
        } else {
            0
        }
    }

    fn read_vram_banked(&self, bank: u8, addr: u16) -> u8 {
        let offset = addr as usize & 0x1FFF;
        let bank_start = bank as usize * 0x2000;
        self.vram[bank_start + offset]
    }

    /// return true, if this is the last draw in the current scanline, and
    /// mode 0 is being activated
    fn draw(&mut self) -> bool {
        self.try_enter_window();

        if self.fetcher.cycle() {
            let (bg, attribs) = self.fetch_bg();
            self.fetcher.push(bg, attribs);
        }

        if self.bg_fifo.len() <= 8 {
            if let Some((pixels, attribs)) = self.fetcher.pop() {
                self.bg_fifo.push(
                    pixels,
                    self.cgb_bg_palettes.get_palette(attribs.palette()),
                    attribs.priority(),
                );
            }
        }

        if self.bg_fifo.len() > 8 {
            if self.fine_scroll_x_discard > 0 {
                self.fine_scroll_x_discard -= 1;
                self.bg_fifo.pop();
                self.sprite_fifo.pop();
            } else {
                self.try_add_sprite();

                let color = self.get_next_color();
                self.lcd.push(color, self.scanline);

                if self.lcd.x() == 160 {
                    return true;
                }
            }
        }

        false
    }

    /// Mixes the two pixels (sprite and background) and outputs the correct color,
    /// mixing here does not mean using the two pixels and output something in the middle
    /// mixing just means check priorities and all stuff and pick which should be
    /// rendered, the other is just discarded
    fn get_next_color(&mut self) -> Color {
        let bg_pixel = self.bg_fifo.pop();
        let sprite_pixel = self.sprite_fifo.pop();

        // If we have a sprite, then mix, else just use the background
        let (mut color_index, palette, dmg_palette) = if let Some(sprite_pixel) = sprite_pixel {
            let master_priority = self.is_cgb_mode && !self.lcd_control.bg_window_priority();
            let bg_priority = bg_pixel.bg_priority;
            let oam_bg_priority = sprite_pixel.oam_bg_priority;

            if (master_priority || bg_pixel.color == 0 || (!bg_priority && !oam_bg_priority))
                && sprite_pixel.color != 0
            {
                // sprite wins
                (
                    sprite_pixel.color,
                    sprite_pixel.palette,
                    self.dmg_sprite_palettes[sprite_pixel.dmg_palette as usize],
                )
            } else {
                // background wins
                (bg_pixel.color, bg_pixel.palette, self.dmg_bg_palette)
            }
        } else {
            // there is no sprite pixel, so we just use the background pixel
            (bg_pixel.color, bg_pixel.palette, self.dmg_bg_palette)
        };

        if !self.is_cgb_mode {
            color_index = (dmg_palette >> (2 * color_index)) & 0b11;
        }

        palette.get_color(color_index)
    }

    /// Gets the tile number, BgAttribute for that tile, and its y position
    /// because the y position is different if we are drawing a window or
    /// normal background
    fn fetch_bg_tile_meta(&mut self) -> (u8, BgAttribute, u8) {
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
        let vram_index = tile_map + tile_index;
        let tile = self.read_vram_banked(0, vram_index);
        let tile_attribs = BgAttribute::new(self.read_vram_banked(1, vram_index));

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
        let low = self.read_vram_banked(bank, index + ((y as u16) * 2));
        let high = self.read_vram_banked(bank, index + ((y as u16) * 2 + 1));

        let mut result = [0; 8];

        for (i, result_item) in result.iter_mut().enumerate() {
            let bin_i = 7 - i;
            *result_item = ((high >> bin_i) & 1) << 1 | ((low >> bin_i) & 1);
        }

        result
    }

    fn fetch_bg(&mut self) -> ([u8; 8], BgAttribute) {
        if !self.is_cgb_mode && !self.lcd_control.bg_window_priority() {
            ([0; 8], BgAttribute::new(0))
        } else {
            let (tile, attribs, y) = self.fetch_bg_tile_meta();

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
    }

    fn load_selected_sprites_oam(&mut self) {
        let mut count = 0;
        for (i, &sprite) in self.oam.iter().enumerate() {
            // in range
            if self.scanline.wrapping_sub(sprite.screen_y()) < self.lcd_control.sprite_size() {
                self.selected_oam[count] = SelectedSprite::new(sprite, i as u8);
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
            for selected_sprite in self
                .selected_oam
                .iter()
                .take(self.selected_oam_size as usize)
            {
                let sprite = selected_sprite.sprite();

                // the x index of the sprite is of the left of the display
                let left_out_of_bounds = self.lcd.x() == 0 && sprite.x() < 8;

                if self.lcd.x() == sprite.screen_x() || left_out_of_bounds {
                    let mut y = self.scanline.wrapping_sub(sprite.screen_y());
                    if sprite.y_flipped() {
                        // sometimes `sprite_size` is smaller than `y`, which result in
                        //  overflowing sub, and it might be due to changing the
                        //  `sprite_size` in the middle of the scanline.
                        //
                        //  FIXME: Not sure if this works, but for now at least stay
                        //   in the range
                        y = (self.lcd_control.sprite_size() - 1).wrapping_sub(y)
                            % self.lcd_control.sprite_size();
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

                    // `cgb_sprite_palettes` 0, 1 will be used in DMG mode
                    // together with `dmg_sprite_palettes`
                    let palette_selector = if self.is_cgb_mode {
                        sprite.cgb_palette()
                    } else {
                        sprite.dmg_palette()
                    };

                    self.sprite_fifo.push(
                        colors,
                        selected_sprite,
                        self.cgb_sprite_palettes.get_palette(palette_selector),
                    );
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
            self.bg_fifo.clear();
            self.sprite_fifo.clear();
            self.fetcher.x = 0;
            self.is_drawing_window = true;
        }
    }

    /// Ending stuff for mode 3
    fn enter_hblank(&mut self) {
        self.lcd.next_line();
        // clear for the next line
        self.bg_fifo.clear();
        self.sprite_fifo.clear();
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
