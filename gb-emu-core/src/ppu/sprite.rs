use bitflags::bitflags;

bitflags! {
    struct SpriteFlags: u8 {
        const PRIORITY = 1 << 7;
        const X_FLIP   = 1 << 6;
        const Y_FLIP   = 1 << 5;
        const PALLETE  = 1 << 4;
    }
}

pub struct Sprite {
    y: u8,
    x: u8,
    pattern: u8,
    flags: SpriteFlags,
}

impl Sprite {
    pub fn get_at_offset(&self, offset: u8) -> u8 {
        match offset {
            0 => self.y,
            1 => self.x,
            2 => self.pattern,
            3 => self.flags.bits(),
            _ => unreachable!(),
        }
    }

    pub fn set_at_offset(&mut self, offset: u8, data: u8) {
        match offset {
            0 => self.y = data,
            1 => self.x = data,
            2 => self.pattern = data,
            3 => self
                .flags
                .clone_from(&SpriteFlags::from_bits_truncate(data)),
            _ => unreachable!(),
        }
    }
}
