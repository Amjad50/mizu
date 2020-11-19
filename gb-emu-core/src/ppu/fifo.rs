use fixed_vec_deque::FixedVecDeque;

#[derive(Clone, Copy)]
pub enum FifoPixel {
    Background {
        color: u8,
    },
    Sprite {
        color: u8,
        palette: u8,
        background_priority: bool,
    },
    Empty,
}

impl Default for FifoPixel {
    fn default() -> Self {
        Self::Empty
    }
}

impl FifoPixel {
    pub fn color(&self) -> u8 {
        match self {
            Self::Background { color } | Self::Sprite { color, .. } => *color,
            Self::Empty => unreachable!("Should not request color of empty pixel"),
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
    pub fn pop(&mut self) -> FifoPixel {
        *self.pixels.pop_front().unwrap()
    }

    pub fn push_bg(&mut self, colors: [u8; 8]) {
        for &color in colors.iter() {
            *self.pixels.push_back() = FifoPixel::Background { color };
        }
    }

    pub fn len(&self) -> usize {
        self.pixels.len()
    }

    pub fn clear(&mut self) {
        self.pixels.clear();
    }
}
