use super::ApuChannel;

const VOLUME_SHIFT_TABLE: [u8; 4] = [4, 0, 1, 2];

#[derive(Default)]
pub struct WaveChannel {
    volume: u8,
    volume_shift: u8,
    frequency: u16,

    buffer: [u8; 32],
    buffer_position: u8,

    frequency_timer: u16,

    channel_enable: bool,
}

impl WaveChannel {
    pub fn write_volume(&mut self, vol: u8) {
        self.volume = vol;
        self.volume_shift = VOLUME_SHIFT_TABLE[vol as usize & 3];
    }

    pub fn read_volume(&self) -> u8 {
        self.volume
    }

    pub fn frequency(&self) -> u16 {
        self.frequency
    }

    pub fn write_frequency(&mut self, data: u16) {
        self.frequency = data;
    }

    pub fn write_buffer(&mut self, offset: u8, data: u8) {
        self.buffer[offset as usize & 0x1F] = data;
    }

    pub fn read_buffer(&self, offset: u8) -> u8 {
        self.buffer[offset as usize & 0x1F]
    }

    pub fn clock(&mut self) {
        if self.frequency_timer == 0 {
            self.clock_position();

            // reload timer
            self.frequency_timer = 0x7FF - self.frequency;
        } else {
            self.frequency_timer -= 1;
        }
    }

    pub fn reset_buffer_index(&mut self) {
        self.buffer_position = 0;
    }
}

impl WaveChannel {
    fn clock_position(&mut self) {
        self.buffer_position = (self.buffer_position + 1) % 32;
    }
}

impl ApuChannel for WaveChannel {
    fn output(&mut self) -> u8 {
        let byte = self.buffer[self.buffer_position as usize / 2];
        // the shift will be 4 if buffer_position is even, and 0 if its odd
        let shift = 4 * ((self.buffer_position & 1) ^ 1);
        let byte = (byte >> shift) & 0xF;

        byte >> self.volume_shift
    }

    fn muted(&self) -> bool {
        false
    }

    fn set_enable(&mut self, enabled: bool) {
        self.channel_enable = enabled;
    }

    fn enabled(&self) -> bool {
        self.channel_enable
    }

    fn trigger(&mut self) {
        self.buffer_position = 0;
    }
}
