use super::colors::Color;

pub const LCD_WIDTH: usize = 160;
pub const LCD_HEIGHT: usize = 144;

pub struct Lcd {
    x: u8,
    buf: [u8; LCD_WIDTH * LCD_HEIGHT * 3],
}

impl Default for Lcd {
    fn default() -> Self {
        Self {
            x: 0,
            buf: [0xFF; LCD_WIDTH * LCD_HEIGHT * 3],
        }
    }
}

impl Lcd {
    pub fn push(&mut self, color: Color, y: u8) {
        let index = (y as usize * LCD_WIDTH + self.x as usize) * 3;

        self.buf[index + 0] = color.r;
        self.buf[index + 1] = color.g;
        self.buf[index + 2] = color.b;
        self.x += 1;
    }

    pub fn x(&self) -> u8 {
        self.x
    }

    pub fn next_line(&mut self) {
        self.x = 0;
    }

    pub fn screen_buffer(&self) -> &[u8] {
        &self.buf
    }

    pub fn clear(&mut self) {
        for i in &mut self.buf {
            // fill with white
            *i = 0xFF;
        }
    }
}
