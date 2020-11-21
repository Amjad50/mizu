pub const LCD_WIDTH: usize = 160;
pub const LCD_HEIGHT: usize = 144;

pub struct Lcd {
    x: u8,
    buf: [u8; LCD_WIDTH * LCD_HEIGHT],
}

impl Default for Lcd {
    fn default() -> Self {
        Self {
            x: 0,
            buf: [0; 160 * 144],
        }
    }
}

impl Lcd {
    pub fn push(&mut self, pixel: u8, y: u8) {
        let index = y as usize * LCD_WIDTH + self.x as usize;

        self.buf[index] = (3 - (pixel & 3)) * 85;
        self.x += 1;
    }

    pub fn x(&self) -> u8 {
        self.x
    }

    pub fn next_line(&mut self) {
        self.x = 0;
    }

    pub fn screen_buffer(&self) -> Vec<u8> {
        self.buf.to_vec()
    }
}
