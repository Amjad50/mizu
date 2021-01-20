use super::colors::Color;

pub const LCD_WIDTH: usize = 160;
pub const LCD_HEIGHT: usize = 144;

pub struct Lcd {
    x: u8,
    buf: [[u8; LCD_WIDTH * LCD_HEIGHT * 3]; 2],
    selected_buffer: usize,
    raw_buf: [u8; LCD_WIDTH * LCD_HEIGHT * 3],
}

impl Default for Lcd {
    fn default() -> Self {
        Self {
            x: 0,
            buf: [[0xFF; LCD_WIDTH * LCD_HEIGHT * 3]; 2],
            selected_buffer: 0,
            raw_buf: [0x1F; LCD_WIDTH * LCD_HEIGHT * 3],
        }
    }
}

impl Lcd {
    pub fn push(&mut self, color: Color, y: u8) {
        let index = (y as usize * LCD_WIDTH + self.x as usize) * 3;

        let r = color.r as u16;
        let g = color.g as u16;
        let b = color.b as u16;

        let rr = r * 26 + g * 4 + b * 2;
        let gg = g * 24 + b * 8;
        let bb = r * 6 + g * 4 + b * 22;

        let rr = rr.min(960) >> 2;
        let gg = gg.min(960) >> 2;
        let bb = bb.min(960) >> 2;

        let i = self.next_buffer_index();
        self.buf[i][index + 0] = rr as u8;
        self.buf[i][index + 1] = gg as u8;
        self.buf[i][index + 2] = bb as u8;

        // used for testing
        self.raw_buf[index + 0] = color.r & 0x1F;
        self.raw_buf[index + 1] = color.g & 0x1F;
        self.raw_buf[index + 2] = color.b & 0x1F;

        self.x += 1;
    }

    pub fn x(&self) -> u8 {
        self.x
    }

    pub fn next_line(&mut self) {
        self.x = 0;
    }

    pub fn switch_buffers(&mut self) {
        self.selected_buffer = self.next_buffer_index();
    }

    pub fn screen_buffer(&self) -> &[u8] {
        &self.buf[self.selected_buffer as usize]
    }

    #[cfg(test)]
    pub fn raw_screen_buffer(&self) -> &[u8] {
        &self.raw_buf
    }

    pub fn clear(&mut self) {
        for buf in self.buf.iter_mut() {
            for (byte, raw_byte) in buf.iter_mut().zip(self.raw_buf.iter_mut()) {
                // fill with white
                *byte = 0xFF;
                *raw_byte = 0x1F;
            }
        }
    }
}

impl Lcd {
    fn next_buffer_index(&self) -> usize {
        self.selected_buffer ^ 1
    }
}
