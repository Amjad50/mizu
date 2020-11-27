#[derive(Default)]
pub struct EnvelopGenerator {
    volume: u8,
    sweep_increase: bool,
    sweep_count: u8,
}

impl EnvelopGenerator {
    pub fn write_envelope_register(&mut self, data: u8) {
        // TODO: is initial volume different?
        self.volume = data >> 4;
        self.sweep_increase = (data >> 3) & 1 == 1;
        self.sweep_count = data & 7;
    }

    pub fn read_envelope_register(&self) -> u8 {
        ((self.volume & 0xF) << 4) | ((self.sweep_increase as u8) << 3) | (self.sweep_count & 7)
    }

    pub fn current_volume(&self) -> u8 {
        self.volume
    }
}
