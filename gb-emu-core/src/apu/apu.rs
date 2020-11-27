use super::pulse_channel::PulseChannel;
use super::{ApuChannel, LengthCountedChannel};

pub struct Apu {
    pulse1: LengthCountedChannel<PulseChannel>,

    sample_counter: f64,
    buffer: Vec<f32>,
}

impl Default for Apu {
    fn default() -> Self {
        Self {
            buffer: Vec::new(),
            sample_counter: 0.,
            pulse1: LengthCountedChannel::new(PulseChannel::default()),
        }
    }
}

impl Apu {
    pub fn read_register(&mut self, addr: u16) -> u8 {
        match addr {
            0xFF10 => self.pulse1.channel_mut().read_sweep_register(),
            0xFF11 => 0x3F | (self.pulse1.channel_mut().read_pattern_duty() << 6),
            0xFF12 => self.pulse1.channel().envelope().read_envelope_register(),
            0xFF13 => 0xFF,
            0xFF14 => 0xBF | ((self.pulse1.read_length_enable() as u8) << 6),
            _ => 0xFF,
        }
    }

    pub fn write_register(&mut self, addr: u16, data: u8) {
        match addr {
            0xFF10 => self.pulse1.channel_mut().write_sweep_register(data),
            0xFF11 => {
                self.pulse1.channel_mut().write_pattern_duty(data >> 5);
                self.pulse1.write_sound_length(data & 0x3F);
            }
            0xFF12 => self
                .pulse1
                .channel_mut()
                .envelope_mut()
                .write_envelope_register(data),
            0xFF13 => {
                let freq = (self.pulse1.channel().frequency() & 0xFF00) | data as u16;
                self.pulse1.channel_mut().write_frequency(freq);
            }
            0xFF14 => {
                let freq =
                    (self.pulse1.channel().frequency() & 0xFF) | (((data as u16) & 0x7) << 8);
                self.pulse1.channel_mut().write_frequency(freq);

                self.pulse1.write_length_enable((data >> 6) & 1 == 1);

                if data & 0x80 != 0 {
                    // restart
                }
            }
            _ => {}
        }
    }

    pub fn get_buffer(&mut self) -> Vec<f32> {
        std::mem::replace(&mut self.buffer, Vec::new())
    }

    pub fn clock(&mut self) {
        const SAMPLE_RATE: f64 = 22050.;
        const SAMPLE_EVERY_N_CLOCKS: f64 = (((16384 * 256) / 4) as f64) / SAMPLE_RATE;

        self.sample_counter += 1.;
        if self.sample_counter >= SAMPLE_EVERY_N_CLOCKS {
            let sample = self.pulse1.output();

            self.buffer.push(sample as f32 / 0xF as f32);
        }

        self.pulse1.channel_mut().clock();
    }
}
