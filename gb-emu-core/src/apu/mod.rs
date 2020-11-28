mod apu;
mod envelope;
mod pulse_channel;

pub use apu::Apu;

trait ApuChannel {
    fn output(&mut self) -> u8;
    fn muted(&self) -> bool;
}

struct LengthCountedChannel<C: ApuChannel> {
    max_length: u8,
    length: u8,
    current_counter: u8,
    counter_decrease_enable: bool,
    muted: bool,
    channel: C,
}

impl<C: ApuChannel> LengthCountedChannel<C> {
    pub fn new(channel: C, max_length: u8) -> Self {
        Self {
            max_length,
            length: 0,
            current_counter: 0,
            counter_decrease_enable: false,
            muted: false,
            channel,
        }
    }
    pub fn channel(&self) -> &C {
        &self.channel
    }

    pub fn channel_mut(&mut self) -> &mut C {
        &mut self.channel
    }

    pub fn write_sound_length(&mut self, data: u8) {
        self.length = self.max_length - data;
        self.current_counter = self.length;
    }

    pub fn write_length_enable(&mut self, data: bool) {
        self.counter_decrease_enable = data;
        self.current_counter = self.length;
    }

    pub fn read_length_enable(&self) -> bool {
        self.counter_decrease_enable
    }

    pub fn restart_channel(&mut self) {
        self.muted = false;
        self.current_counter = self.length;
    }

    pub fn clock_length_counter(&mut self) {
        if self.counter_decrease_enable {
            if self.current_counter == 0 {
                self.muted = true;
            } else {
                self.current_counter -= 1;
                if self.current_counter == 0 {
                    self.muted = true;
                    self.counter_decrease_enable = false;
                }
            }
        }
    }
}

impl<C: ApuChannel> ApuChannel for LengthCountedChannel<C> {
    fn output(&mut self) -> u8 {
        if self.muted {
            0
        } else {
            self.channel.output()
        }
    }

    fn muted(&self) -> bool {
        self.muted || self.channel.muted()
    }
}

struct Dac<C: ApuChannel> {
    capacitor: f32,
    channel: C,
}

impl<C: ApuChannel> Dac<C> {
    pub fn new(channel: C) -> Self {
        Self {
            capacitor: 0.,
            channel,
        }
    }

    pub fn dac_output(&mut self) -> f32 {
        if self.channel.muted() {
            0.
        } else {
            // divide by 8 because we will multiply by master volume
            let dac_in = self.channel.output() as f32 / 15.;
            let dac_out = dac_in - self.capacitor;

            self.capacitor = dac_in - dac_out * 0.926;

            dac_out
        }
    }
}

impl<C: ApuChannel> std::ops::Deref for Dac<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.channel
    }
}

impl<C: ApuChannel> std::ops::DerefMut for Dac<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.channel
    }
}
