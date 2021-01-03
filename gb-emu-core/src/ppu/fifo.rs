use super::colors::ColorPalette;
use fixed_vec_deque::FixedVecDeque;

// Background store the `bg_priority` of the `bg_attribs` for the pixel data
// Sprite store the index of the sprite, as in CGB priority is done by index
//  and not by coordinate
#[derive(Clone, Copy)]
enum PixelType {
    Background(bool),
    Sprite(u8),
}

#[derive(Clone, Copy)]
struct FifoPixel {
    color: u8,
    palette: ColorPalette,
    pixel_type: PixelType,
}

impl Default for FifoPixel {
    fn default() -> Self {
        Self {
            color: 0,
            palette: ColorPalette::default(),
            pixel_type: PixelType::Background(false),
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
    pub fn pop(&mut self) -> (u8, ColorPalette) {
        let pixel = *self.pixels.pop_front().unwrap();

        (pixel.color, pixel.palette)
    }

    pub fn push_bg(&mut self, colors: [u8; 8], palette: ColorPalette, bg_priority: bool) {
        for &color in colors.iter() {
            *self.pixels.push_back() = FifoPixel {
                pixel_type: PixelType::Background(bg_priority),
                palette,
                color,
            };
        }
    }

    pub fn mix_sprite(
        &mut self,
        colors: [u8; 8],
        palette: ColorPalette,
        index: u8,
        oam_bg_priority: bool,
        master_priority: bool,
    ) {
        assert!(self.len() >= 8);

        for (pixel, &sprite_color) in self.pixels.iter_mut().take(8).zip(colors.iter()) {
            match pixel.pixel_type {
                PixelType::Background(bg_priority) => {
                    // TODO: fix this mess
                    if (master_priority
                        || ((!bg_priority || pixel.color == 0)
                            && (!oam_bg_priority || pixel.color == 0)))
                        && sprite_color != 0
                    {
                        pixel.color = sprite_color;
                        pixel.palette = palette;
                        pixel.pixel_type = PixelType::Sprite(index);
                    }
                }
                PixelType::Sprite(sprite_index) => {
                    if sprite_index > index || pixel.color == 0 {
                        pixel.color = sprite_color;
                        pixel.palette = palette;
                        pixel.pixel_type = PixelType::Sprite(index);
                    }
                }
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
