mod apu;
mod envelope;
mod pulse_channel;

pub use apu::Apu;

trait ApuChannel {
    fn output(&mut self) -> u8;
}

struct LengthCountedChannel<C: ApuChannel> {
    length: u8,
    enable: bool,
    channel: C,
}

impl<C: ApuChannel> LengthCountedChannel<C> {
    pub fn new(channel: C) -> Self {
        Self {
            length: 0,
            enable: false,
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
        self.length = data;
    }

    pub fn write_length_enable(&mut self, data: bool) {
        self.enable = data;
    }

    pub fn read_length_enable(&self) -> bool {
        self.enable
    }
}

impl<C: ApuChannel> ApuChannel for LengthCountedChannel<C> {
    fn output(&mut self) -> u8 {
        if self.length == 0 && self.enable {
            0
        } else {
            self.channel.output()
        }
    }
}
