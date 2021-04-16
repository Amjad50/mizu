use serde::{Deserialize, Serialize};

use super::colors::ColorPalette;
use super::sprite::SelectedSprite;
use fixed_vec_deque::FixedVecDeque;

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum SpritePriorityMode {
    ByIndex, // CGB
    ByCoord, // DMG
}

/// Background store the `bg_priority` of the `bg_attribs` for the pixel data
#[derive(Clone, Copy, Default, Serialize, Deserialize)]
pub struct BgFifoPixel {
    pub color: u8,
    pub palette: ColorPalette,
    pub bg_priority: bool,
}

/// Sprite store the index of the sprite, as in CGB priority is done by index
///  and not by coordinate
#[derive(Clone, Copy, Default, Serialize, Deserialize)]
pub struct SpriteFifoPixel {
    pub color: u8,
    pub palette: ColorPalette,
    pub dmg_palette: u8,
    pub index: u8,
    pub oam_bg_priority: bool,
}

pub struct BgFifo {
    pixels: FixedVecDeque<[BgFifoPixel; 16]>,
}

impl Default for BgFifo {
    fn default() -> Self {
        Self {
            pixels: FixedVecDeque::new(),
        }
    }
}

impl BgFifo {
    pub fn pop(&mut self) -> BgFifoPixel {
        *self.pixels.pop_front().unwrap()
    }

    pub fn push(&mut self, colors: [u8; 8], palette: ColorPalette, bg_priority: bool) {
        for &color in colors.iter() {
            *self.pixels.push_back() = BgFifoPixel {
                color,
                palette,
                bg_priority,
            };
        }
    }

    pub fn len(&self) -> usize {
        self.pixels.len()
    }

    pub fn clear(&mut self) {
        self.pixels.clear();
    }
}

pub struct SpriteFifo {
    pixels: FixedVecDeque<[SpriteFifoPixel; 8]>,
    sprite_priority_mode: SpritePriorityMode,
}

impl SpriteFifo {
    pub fn new(sprite_priority_mode: SpritePriorityMode) -> Self {
        Self {
            pixels: FixedVecDeque::new(),
            sprite_priority_mode,
        }
    }

    pub fn update_sprite_priority_mode(&mut self, sprite_priority_mode: SpritePriorityMode) {
        self.sprite_priority_mode = sprite_priority_mode;
    }

    pub fn pop(&mut self) -> Option<SpriteFifoPixel> {
        self.pixels.pop_front().map(|x| *x)
    }

    pub fn push(&mut self, colors: [u8; 8], sprite: &SelectedSprite, palette: ColorPalette) {
        let dmg_palette = sprite.sprite().dmg_palette();
        let index = sprite.index();
        let oam_bg_priority = sprite.sprite().bg_priority();

        // If there are still elements in the fifo, then mix the two sprites
        // together, meaning, check priorities and replace sprite pixel if needed
        let mut to_mix = self.len();

        for (i, &new_color) in colors.iter().enumerate() {
            let new_sprite_pixel = SpriteFifoPixel {
                color: new_color,
                palette,
                dmg_palette,
                index,
                oam_bg_priority,
            };

            // replace or mix
            if to_mix > 0 {
                to_mix -= 1;

                let old_pixel = &mut self.pixels[i];

                if ((self.sprite_priority_mode == SpritePriorityMode::ByIndex
                    && index < old_pixel.index)
                    || old_pixel.color == 0)
                    && new_color != 0
                {
                    *old_pixel = new_sprite_pixel;
                }
            } else {
                *self.pixels.push_back() = new_sprite_pixel;
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
