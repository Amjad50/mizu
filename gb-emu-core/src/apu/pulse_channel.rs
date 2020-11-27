use super::envelope::EnvelopGenerator;
use super::ApuChannel;

const DUTY_CYCLE_SEQUENCES: [[u8; 8]; 4] = [
    [1, 0, 0, 0, 0, 0, 0, 0],
    [1, 1, 0, 0, 0, 0, 0, 0],
    [1, 1, 1, 1, 0, 0, 0, 0],
    [0, 0, 1, 1, 1, 1, 1, 1],
];

pub struct PulseChannel {
    sweep_time: u8,
    is_sweep_decrese: bool,
    sweep_shift_n: u8,
    sequencer_data: [u8; 8],
    sequencer_position: usize,
    duty: u8,
    envelope: EnvelopGenerator,
    frequency: u16,

    current_timer: u16,
}

impl Default for PulseChannel {
    fn default() -> Self {
        Self {
            sweep_time: 0,
            is_sweep_decrese: false,
            sweep_shift_n: 0,
            duty: 0,
            sequencer_data: DUTY_CYCLE_SEQUENCES[0],
            sequencer_position: 0,
            envelope: EnvelopGenerator::default(),
            frequency: 0,
            current_timer: 0,
        }
    }
}

impl PulseChannel {
    pub fn write_sweep_register(&mut self, data: u8) {
        self.sweep_time = (data >> 4) & 7;
        self.is_sweep_decrese = (data >> 3) & 1 == 1;
        self.sweep_shift_n = data & 7;
    }

    pub fn read_sweep_register(&self) -> u8 {
        ((self.sweep_time & 7) << 4)
            | ((self.is_sweep_decrese as u8) << 3)
            | (self.sweep_shift_n & 7)
    }

    pub fn write_pattern_duty(&mut self, data: u8) {
        // TODO: find if we need to reset the sequencer or not (NES do not need)
        self.sequencer_data = DUTY_CYCLE_SEQUENCES[data as usize & 3];
        self.duty = data & 3;
    }

    pub fn read_pattern_duty(&self) -> u8 {
        self.duty & 3
    }

    pub fn frequency(&self) -> u16 {
        self.frequency
    }

    pub fn write_frequency(&mut self, data: u16) {
        self.frequency = data;
    }

    pub fn envelope(&self) -> &EnvelopGenerator {
        &self.envelope
    }

    pub fn envelope_mut(&mut self) -> &mut EnvelopGenerator {
        &mut self.envelope
    }

    pub fn clock(&mut self) {
        if self.current_timer == 0 {
            self.clock_sequencer();

            // reload timer
            self.current_timer = 0x7FF - self.frequency;
        } else {
            self.current_timer -= 1;
        }
    }
}

impl PulseChannel {
    fn clock_sequencer(&mut self) {
        self.sequencer_position = (self.sequencer_position + 1) % 8;
    }
}

impl ApuChannel for PulseChannel {
    fn output(&mut self) -> u8 {
        (self.sequencer_data[self.sequencer_position]) * self.envelope.current_volume()
    }
}
