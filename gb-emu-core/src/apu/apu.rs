use super::pulse_channel::PulseChannel;
use super::{ApuChannel, LengthCountedChannel};
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
    pulse1: LengthCountedChannel<PulseChannel>,

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
            pulse1: LengthCountedChannel::new(PulseChannel::default(), 64),
            cycle: 0,
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

            0xFF24 => self.channels_control.bits(),
            0xFF25 => self.channels_selection.bits(),
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
                    self.pulse1.restart_channel();
                }
            }

            0xFF24 => self
                .channels_control
                .clone_from(&ChannelsControl::from_bits_truncate(data)),
            0xFF25 => self
                .channels_selection
                .clone_from(&ChannelsSelection::from_bits_truncate(data)),
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

        // TODO: change frame counter stuff
        match self.cycle {
            4096 | 12288 => {
                // quarter frame
                self.pulse1.clock_length_counter();
            }
            8192 => {
                // half frame
                self.pulse1.clock_length_counter();
            }
            16384 => {
                // full frame
                self.pulse1.clock_length_counter();
                self.pulse1.channel_mut().envelope_mut().clock();
                self.cycle = 0;
            }
            _ => {}
        }
    }
}

impl Apu {
    fn get_outputs(&mut self) -> (f32, f32) {
        let mut right = 0.;
        let mut left = 0.;

        let pulse1 = if self.pulse1.muted() {
            0.
        } else {
            self.pulse1.output() as f32 / 15.
        };

        if self
            .channels_selection
            .contains(ChannelsSelection::PULSE1_LEFT)
        {
            left += pulse1;
        }

        if self
            .channels_selection
            .contains(ChannelsSelection::PULSE1_RIGHT)
        {
            right += pulse1;
        }

        (right, left)
    }
}
