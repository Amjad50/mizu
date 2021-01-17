macro_rules! color {
    ($r:expr, $g:expr, $b:expr) => {
        Color {
            r: $r,
            g: $g,
            b: $b,
        }
    };
}

pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn from_raw(mut color: u16) -> Self {
        let r = (color & 0x1F) as u8;
        color >>= 5;
        let g = (color & 0x1F) as u8;
        color >>= 5;
        let b = (color & 0x1F) as u8;

        Self { r, g, b }
    }

    pub fn to_raw(&self) -> u16 {
        let r = (self.r & 0x1F) as u16;
        let g = (self.g & 0x1F) as u16;
        let b = (self.b & 0x1F) as u16;

        r | (g << 5) | (b << 10)
    }
}

#[derive(Clone, Copy)]
pub struct ColorPalette {
    data: [u16; 4],
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self { data: [0; 4] }
    }
}

impl ColorPalette {
    pub fn new(colors: [Color; 4]) -> Self {
        let mut raw_colors = [0; 4];
        for (i, color) in colors.iter().enumerate() {
            raw_colors[i] = color.to_raw();
        }
        Self { data: raw_colors }
    }

    pub fn get_color(&self, color_index: u8) -> Color {
        let color = self.data[color_index as usize & 3];
        Color::from_raw(color)
    }
}

impl ColorPalette {
    fn set_color_data(&mut self, index: u8, data: u8) {
        let color_ref = &mut self.data[index as usize / 2];

        if index & 1 != 0 {
            *color_ref &= 0xFF;
            *color_ref |= (data as u16) << 8;
        } else {
            *color_ref &= 0xFF00;
            *color_ref |= data as u16;
        }
    }

    fn get_color_data(&self, index: u8) -> u8 {
        let index = index & 7;

        let color = self.data[index as usize / 2];

        if index & 1 != 0 {
            (color >> 8) as u8
        } else {
            color as u8
        }
    }
}

pub struct ColorPalettesCollection {
    index: u8,
    auto_increment: bool,
    palettes: [ColorPalette; 8],
}

impl Default for ColorPalettesCollection {
    fn default() -> Self {
        Self {
            index: 0,
            auto_increment: true,
            palettes: [ColorPalette::default(); 8],
        }
    }
}

impl ColorPalettesCollection {
    pub fn read_index(&self) -> u8 {
        ((self.auto_increment as u8) << 7) | 0x40 | self.index
    }

    pub fn write_index(&mut self, data: u8) {
        self.index = data & 0x3F;
        self.auto_increment = data & 0x80 != 0;
    }

    pub fn write_color_data(&mut self, data: u8) {
        let palette = &mut self.palettes[self.index as usize / 8];

        palette.set_color_data(self.index % 8, data);

        self.index = (self.index + self.auto_increment as u8) & 0x3F;
    }

    pub fn read_color_data(&self) -> u8 {
        let palette = &self.palettes[self.index as usize / 8];

        palette.get_color_data(self.index % 8)
    }

    pub fn get_palette(&self, index: u8) -> ColorPalette {
        self.palettes[index as usize & 7]
    }

    pub fn set_palette(&mut self, index: u8, palette: ColorPalette) {
        self.palettes[index as usize & 7] = palette;
    }
}
