use fixed_vec_deque::FixedVecDeque;

#[derive(Clone, Copy, PartialEq)]
pub enum PaletteType {
    Background,
    Sprite(u8),
}

#[derive(Clone, Copy)]
struct FifoPixel {
    color: u8,
    palette: PaletteType,
}

impl Default for FifoPixel {
    fn default() -> Self {
        Self {
            color: 0,
            palette: PaletteType::Background,
        }
    }
}

pub struct Fifo {
    pixels: FixedVecDeque<[FifoPixel; 16]>,
}

impl Default for Fifo {
    fn default() -> Self {
        Self {
            pixels: FixedVecDeque::new(),
        }
    }
}

impl Fifo {
    pub fn pop(&mut self) -> (u8, PaletteType) {
        let pixel = *self.pixels.pop_front().unwrap();

        (pixel.color, pixel.palette)
    }

    pub fn push_bg(&mut self, colors: [u8; 8]) {
        for &color in colors.iter() {
            *self.pixels.push_back() = FifoPixel {
                palette: PaletteType::Background,
                color,
            };
        }
    }

    pub fn mix_sprite(&mut self, colors: [u8; 8], palette: u8, background_priority: bool) {
        assert!(self.len() >= 8);

        for (pixel, &sprite_color) in self.pixels.iter_mut().take(8).zip(colors.iter()) {
            if pixel.palette == PaletteType::Background {
                if (!background_priority && sprite_color != 0)
                    || (pixel.color == 0 && sprite_color != 0)
                {
                    pixel.color = sprite_color;
                    pixel.palette = PaletteType::Sprite(palette);
                }
            } else if pixel.color == 0 {
                pixel.color = sprite_color;
                pixel.palette = PaletteType::Sprite(palette);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.pixels.len()
    }

    pub fn clear(&mut self) {
        self.pixels.clear();
    }
}
