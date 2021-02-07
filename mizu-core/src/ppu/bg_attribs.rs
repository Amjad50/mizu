use bitflags::bitflags;

bitflags! {
    #[derive(Default)]
    pub struct BgAttribute: u8 {
        const PRIORITY = 1 << 7;
        const VER_FLIP = 1 << 6;
        const HOR_FLIP = 1 << 5;
        const UNUSED   = 1 << 4;
        const BANK     = 1 << 3;
        const PALETTE  = 0b111;
    }
}

impl BgAttribute {
    pub fn new(byte: u8) -> Self {
        Self::from_bits_truncate(byte)
    }

    pub fn bank(&self) -> u8 {
        self.contains(Self::BANK) as u8
    }

    pub fn palette(&self) -> u8 {
        self.bits() & Self::PALETTE.bits
    }

    pub fn is_horizontal_flip(&self) -> bool {
        self.contains(Self::HOR_FLIP)
    }

    pub fn is_vertical_flip(&self) -> bool {
        self.contains(Self::VER_FLIP)
    }

    pub fn priority(&self) -> bool {
        self.contains(Self::PRIORITY)
    }
}
