#[derive(Default)]
pub struct EnvelopGenerator {
    starting_volume: u8,
    current_volume: u8,
    sweep_increase: bool,
    period: u8,

    counter: u8,
}

impl EnvelopGenerator {
    pub fn write_envelope_register(&mut self, data: u8) {
        // TODO: is initial volume different?
        self.starting_volume = data >> 4;
        self.current_volume = self.starting_volume;
        self.sweep_increase = (data >> 3) & 1 == 1;
        self.period = data & 7;
        self.counter = self.period;
    }

    pub fn read_envelope_register(&self) -> u8 {
        ((self.starting_volume & 0xF) << 4) | ((self.sweep_increase as u8) << 3) | (self.period & 7)
    }

    pub fn current_volume(&self) -> u8 {
        self.starting_volume
    }

    pub fn clock(&mut self) {
        if self.period != 0 {
            if self.counter == 0 {
                self.counter = self.period;

                let change = if self.sweep_increase { 1 } else { -1 };
                let current_volume = self.current_volume as i8 + change;

                if current_volume >= 0 && current_volume <= 15 {
                    self.current_volume = current_volume as u8;
                }
            } else {
                self.counter -= 1;
            }
        }
    }

    pub fn trigger(&mut self) {
        self.counter = self.period;
        self.current_volume = self.starting_volume;
    }
}
