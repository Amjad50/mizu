use fixed_vec_deque::FixedVecDeque;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PixelType {
    Background,
    Sprite,
}

#[derive(Clone, Copy, Debug)]
pub struct FifoPixel {
    ty: PixelType,
    color: u8,
    palette: u8,
}

impl Default for FifoPixel {
    fn default() -> Self {
        Self {
            ty: PixelType::Background,
            color: 0,
            palette: 0,
        }
    }
}

impl FifoPixel {
    pub fn color(&self) -> u8 {
        self.color
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
    pub fn pop(&mut self) -> FifoPixel {
        *self.pixels.pop_front().unwrap()
    }

    pub fn push_bg(&mut self, colors: [u8; 8]) {
        for &color in colors.iter() {
            *self.pixels.push_back() = FifoPixel {
                ty: PixelType::Background,
                color,
                palette: 0xFF,
            };
        }
    }

    pub fn mix_sprite(&mut self, colors: [u8; 8], palette: u8, background_priority: bool) {
        assert!(self.len() >= 8);

        for (pixel, &sprite_color) in self.pixels.iter_mut().take(8).zip(colors.iter()) {
            if pixel.ty == PixelType::Background && !background_priority && sprite_color != 0 {
                pixel.color = sprite_color;
                pixel.palette = palette;
                pixel.ty = PixelType::Sprite;
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
