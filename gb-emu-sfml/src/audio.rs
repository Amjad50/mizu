use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{Producer, RingBuffer};

pub struct AudioPlayer {
    buffer_producer: Producer<f32>,
    output_stream: cpal::Stream,
}

impl AudioPlayer {
    pub fn new(sample_rate: u32) -> Self {
        let host = cpal::default_host();
        let output_device = host
            .default_output_device()
            .expect("failed to get default output audio device");

        let config = cpal::StreamConfig {
            channels: 2,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        // Limiting the number of samples in the buffer is better to minimize
        // audio delay in emulation, this is because emulation speed
        // does not 100% match audio playing speed (44100Hz).
        // The buffer holds only audio for 1/4 second, which is good enough for delays,
        // It can be reduced more, but it might cause noise(?) for slower machines
        // or if any CPU intensive process started while the emulator is running
        let buffer = RingBuffer::new(sample_rate as usize / 2);
        let (buffer_producer, mut buffer_consumer) = buffer.split();

        let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for sample in data {
                // /5.5 to reduce the volume of the sample
                *sample = buffer_consumer.pop().unwrap_or(0.) / 5.5;
            }
        };

        let output_stream = output_device
            .build_output_stream(&config, output_data_fn, Self::err_fn)
            .expect("failed to build an output audio stream");

        Self {
            buffer_producer,
            output_stream,
        }
    }

    pub fn play(&self) {
        self.output_stream.play().unwrap();
    }

    /// Pause the player
    /// > not used for now, but maybe later
    #[allow(dead_code)]
    pub fn pause(&self) {
        self.output_stream.pause().unwrap();
    }

    pub fn queue(&mut self, data: &[f32]) {
        self.buffer_producer.push_slice(data);
    }
}

impl AudioPlayer {
    fn err_fn(err: cpal::StreamError) {
        eprintln!("an error occurred on audio stream: {}", err);
    }
}
