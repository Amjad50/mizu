use super::noise_channel::NoiseChannel;
use super::pulse_channel::PulseChannel;
use super::wave_channel::WaveChannel;
use super::{ApuChannel, Dac, LengthCountedChannel};
use bitflags::bitflags;

bitflags! {
    struct ChannelsControl: u8 {
        const VIN_LEFT  = 1 << 7;
        const VOL_LEFT  = 7 << 4;
        const VIN_RIGHT = 1 << 3;
        const VOL_RIGHT = 7;
    }
}

impl ChannelsControl {
    fn vol_left(&self) -> u8 {
        (self.bits() >> 4) & 7
    }

    fn vol_right(&self) -> u8 {
        self.bits() & 7
    }
}

bitflags! {
    struct ChannelsSelection :u8 {
        const NOISE_LEFT   = 1 << 7;
        const WAVE_LEFT    = 1 << 6;
        const PULSE2_LEFT  = 1 << 5;
        const PULSE1_LEFT  = 1 << 4;
        const NOISE_RIGHT  = 1 << 3;
        const WAVE_RIGHT   = 1 << 2;
        const PULSE2_RIGHT = 1 << 1;
        const PULSE1_RIGHT = 1 << 0;
    }
}

pub struct Apu {
    pulse1: Dac<LengthCountedChannel<PulseChannel>>,
    pulse2: Dac<LengthCountedChannel<PulseChannel>>,
    wave: Dac<LengthCountedChannel<WaveChannel>>,
    noise: Dac<LengthCountedChannel<NoiseChannel>>,

    channels_control: ChannelsControl,
    channels_selection: ChannelsSelection,

    sample_counter: f64,
    buffer: Vec<f32>,

    cycle: u16,
}

impl Default for Apu {
    fn default() -> Self {
        Self {
            channels_control: ChannelsControl::from_bits_truncate(0),
            channels_selection: ChannelsSelection::from_bits_truncate(0),
            buffer: Vec::new(),
            sample_counter: 0.,
            pulse1: Dac::new(LengthCountedChannel::new(PulseChannel::default(), 64)),
            pulse2: Dac::new(LengthCountedChannel::new(PulseChannel::default(), 64)),
            wave: Dac::new(LengthCountedChannel::new(WaveChannel::default(), 256)),
            noise: Dac::new(LengthCountedChannel::new(NoiseChannel::default(), 64)),
            cycle: 0,
        }
    }
}

impl Apu {
    pub fn read_register(&mut self, addr: u16) -> u8 {
        match addr {
            0xFF10 => 0x80 | self.pulse1.channel_mut().read_sweep_register(),
            0xFF11 => 0x3F | (self.pulse1.channel_mut().read_pattern_duty() << 6),
            0xFF12 => self.pulse1.channel().envelope().read_envelope_register(),
            0xFF13 => 0xFF,
            0xFF14 => 0xBF | ((self.pulse1.read_length_enable() as u8) << 6),

            0xFF15 => 0xFF,
            0xFF16 => 0x3F | (self.pulse2.channel_mut().read_pattern_duty() << 6),
            0xFF17 => self.pulse2.channel().envelope().read_envelope_register(),
            0xFF18 => 0xFF,
            0xFF19 => 0xBF | ((self.pulse2.read_length_enable() as u8) << 6),

            0xFF1A => 0x7F | ((self.wave.channel().read_channel_enable() as u8) << 7),
            0xFF1B => 0xFF,
            0xFF1C => 0x9F | ((self.wave.channel().read_volume()) << 5),
            0xFF1D => 0xFF,
            0xFF1E => 0xBF | ((self.wave.read_length_enable() as u8) << 6),

            0xFF1F => 0xFF,
            0xFF20 => 0xE0 | self.noise.read_sound_length(),
            0xFF21 => self.noise.channel().envelope().read_envelope_register(),
            0xFF22 => self.noise.channel().read_noise_register(),
            0xFF23 => 0xBF | ((self.noise.read_length_enable() as u8) << 6),

            0xFF24 => self.channels_control.bits(),
            0xFF25 => self.channels_selection.bits(),
            0xFF26 => {
                // for now no available way to shutdown the apu
                0x80 | 0x70
                    | ((self.noise.enabled() as u8) << 3)
                    | ((self.wave.enabled() as u8) << 2)
                    | ((self.pulse2.enabled() as u8) << 1)
                    | self.pulse1.enabled() as u8
            }

            0xFF27..=0xFF2F => 0xFF,

            0xFF30..=0xFF3F => self.wave.channel().read_buffer((addr & 0xF) as u8),
            _ => 0xFF,
        }
    }

    pub fn write_register(&mut self, addr: u16, data: u8) {
        match addr {
            0xFF10 => self.pulse1.channel_mut().write_sweep_register(data),
            0xFF11 => {
                self.pulse1.channel_mut().write_pattern_duty(data >> 6);
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
                    self.pulse1.trigger();
                }
            }

            0xFF15 => {}
            0xFF16 => {
                self.pulse2.channel_mut().write_pattern_duty(data >> 6);
                self.pulse2.write_sound_length(data & 0x3F);
            }
            0xFF17 => self
                .pulse2
                .channel_mut()
                .envelope_mut()
                .write_envelope_register(data),
            0xFF18 => {
                let freq = (self.pulse2.channel().frequency() & 0xFF00) | data as u16;
                self.pulse2.channel_mut().write_frequency(freq);
            }
            0xFF19 => {
                let freq =
                    (self.pulse2.channel().frequency() & 0xFF) | (((data as u16) & 0x7) << 8);
                self.pulse2.channel_mut().write_frequency(freq);

                self.pulse2.write_length_enable((data >> 6) & 1 == 1);

                if data & 0x80 != 0 {
                    // restart
                    self.pulse2.trigger();
                }
            }

            0xFF1A => {
                self.wave
                    .channel_mut()
                    .write_channel_enable(data & 0x80 != 0);
            }
            0xFF1B => {
                self.wave.write_sound_length(data);
            }
            0xFF1C => self.wave.channel_mut().write_volume((data >> 5) & 3),
            0xFF1D => {
                let freq = (self.wave.channel().frequency() & 0xFF00) | data as u16;
                self.wave.channel_mut().write_frequency(freq);
            }
            0xFF1E => {
                let freq = (self.wave.channel().frequency() & 0xFF) | (((data as u16) & 0x7) << 8);
                self.wave.channel_mut().write_frequency(freq);

                self.wave.write_length_enable((data >> 6) & 1 == 1);

                if data & 0x80 != 0 {
                    // restart
                    self.wave.trigger();
                }
            }

            0xFF1F => {}
            0xFF20 => self.noise.write_sound_length(data & 0x3F),
            0xFF21 => self
                .noise
                .channel_mut()
                .envelope_mut()
                .write_envelope_register(data),
            0xFF22 => self.noise.channel_mut().write_noise_register(data),
            0xFF23 => {
                self.noise.write_length_enable((data >> 6) & 1 == 1);

                if data & 0x80 != 0 {
                    // restart
                    self.noise.trigger();
                }
            }

            0xFF24 => self
                .channels_control
                .clone_from(&ChannelsControl::from_bits_truncate(data)),
            0xFF25 => self
                .channels_selection
                .clone_from(&ChannelsSelection::from_bits_truncate(data)),

            0xFF27..=0xFF2F => {
                // unused
            }

            0xFF30..=0xFF3F => {
                self.wave
                    .channel_mut()
                    .write_buffer((addr & 0xF) as u8, data);
            }
            _ => {}
        }
    }

    pub fn get_buffer(&mut self) -> Vec<f32> {
        std::mem::replace(&mut self.buffer, Vec::new())
    }

    pub fn clock(&mut self) {
        self.cycle += 1;

        const SAMPLE_RATE: f64 = 44100.;
        const SAMPLE_EVERY_N_CLOCKS: f64 = (((16384 * 256) / 4) as f64) / SAMPLE_RATE;

        self.sample_counter += 1.;
        if self.sample_counter >= SAMPLE_EVERY_N_CLOCKS {
            let (right_sample, left_sample) = self.get_outputs();

            // one for the right, one for the left
            self.buffer.push(right_sample);
            self.buffer.push(left_sample);

            self.sample_counter -= SAMPLE_EVERY_N_CLOCKS;
        }

        self.pulse1.channel_mut().clock();
        self.pulse2.channel_mut().clock();
        self.wave.channel_mut().clock();
        self.wave.channel_mut().clock();
        self.noise.channel_mut().clock();

        if self.cycle % 2048 == 0 {
            match self.cycle / 2048 {
                1 | 5 => {
                    self.pulse1.clock_length_counter();
                    self.pulse2.clock_length_counter();
                    self.wave.clock_length_counter();
                    self.noise.clock_length_counter();
                }
                3 | 7 => {
                    self.pulse1.channel_mut().clock_sweeper();
                    self.pulse1.clock_length_counter();
                    self.pulse2.clock_length_counter();
                    self.wave.clock_length_counter();
                    self.noise.clock_length_counter();
                }
                8 => {
                    self.pulse1.channel_mut().envelope_mut().clock();
                    self.pulse2.channel_mut().envelope_mut().clock();
                    self.noise.channel_mut().envelope_mut().clock();
                    self.cycle = 0;
                }
                _ => {}
            }
        }
    }
}

impl Apu {
    fn get_outputs(&mut self) -> (f32, f32) {
        let mut right = 0.;
        let mut left = 0.;

        let pulse1 = self.pulse1.dac_output() / 8.;
        let pulse2 = self.pulse2.dac_output() / 8.;
        let wave = self.wave.dac_output() / 8.;
        let noise = self.noise.dac_output() / 8.;

        if self
            .channels_selection
            .contains(ChannelsSelection::PULSE1_LEFT)
        {
            left += pulse1;
        }

        if self
            .channels_selection
            .contains(ChannelsSelection::PULSE2_LEFT)
        {
            left += pulse2;
        }

        if self
            .channels_selection
            .contains(ChannelsSelection::WAVE_LEFT)
        {
            left += wave;
        }

        if self
            .channels_selection
            .contains(ChannelsSelection::NOISE_LEFT)
        {
            left += noise;
        }

        if self
            .channels_selection
            .contains(ChannelsSelection::PULSE1_RIGHT)
        {
            right += pulse1;
        }

        if self
            .channels_selection
            .contains(ChannelsSelection::PULSE2_RIGHT)
        {
            right += pulse2;
        }

        if self
            .channels_selection
            .contains(ChannelsSelection::WAVE_RIGHT)
        {
            right += wave;
        }

        if self
            .channels_selection
            .contains(ChannelsSelection::NOISE_RIGHT)
        {
            right += noise;
        }

        let right_vol = self.channels_control.vol_right() as f32 + 1.;
        let left_vol = self.channels_control.vol_left() as f32 + 1.;

        (right * right_vol, left * left_vol)
    }
}
