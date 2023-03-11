use std::error::Error;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{Producer, RingBuffer};
use rubato::{FftFixedInOut, Resampler};

#[derive(Debug)]
pub enum AudioPlayerError {
    DualChannelNotSupported,
}

impl Error for AudioPlayerError {}

impl std::fmt::Display for AudioPlayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DualChannelNotSupported => write!(f, "Dual channel not supported"),
        }
    }
}

pub struct AudioPlayer {
    buffer_producer: Producer<f32>,
    resampler: Option<FftFixedInOut<f32>>,
    resample_buffer: Vec<f32>,
    output_stream: cpal::Stream,
}

impl AudioPlayer {
    pub fn new(sample_rate: u32) -> Result<Self, AudioPlayerError> {
        let host = cpal::default_host();
        let output_device = host
            .default_output_device()
            .expect("failed to get default output audio device");

        let sample_rate = cpal::SampleRate(sample_rate);

        let conf = output_device
            .supported_output_configs()
            .unwrap()
            .collect::<Vec<_>>();

        let mut found_conf = false;

        for c in &conf {
            // must have 2 channels and f32 format
            // (almost all? devices will have at least one configuration with these)
            if c.channels() == 2
                && c.sample_format() == cpal::SampleFormat::F32
                && c.min_sample_rate() <= sample_rate
                && c.max_sample_rate() >= sample_rate
            {
                found_conf = true;
                break;
            }
        }

        let (output_sample_rate, resampler) = if found_conf {
            (sample_rate, None)
        } else {
            let def_conf = output_device.default_output_config().unwrap();

            if def_conf.channels() != 2 || def_conf.sample_format() != cpal::SampleFormat::F32 {
                eprintln!("No supported configuration found for audio device, please open an issue in github `Amjad50/mizu`\n\
                      list of supported configurations: {:#?}", conf);
                return Err(AudioPlayerError::DualChannelNotSupported);
            }

            (
                def_conf.sample_rate(),
                Some(
                    FftFixedInOut::<f32>::new(
                        sample_rate.0 as usize,
                        def_conf.sample_rate().0 as usize,
                        // the number of samples for one video frame in 60 FPS
                        sample_rate.0 as usize * 60,
                        2,
                    )
                    .unwrap(),
                ),
            )
        };

        let config = cpal::StreamConfig {
            channels: 2,
            sample_rate: output_sample_rate,
            buffer_size: cpal::BufferSize::Default,
        };

        // Limiting the number of samples in the buffer is better to minimize
        // audio delay in emulation, this is because emulation speed
        // does not 100% match audio playing speed (44100Hz).
        // The buffer holds only audio for 1/4 second, which is good enough for delays,
        // It can be reduced more, but it might cause noise(?) for slower machines
        // or if any CPU intensive process started while the emulator is running
        let buffer = RingBuffer::new(output_sample_rate.0 as usize / 2);
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

        Ok(Self {
            buffer_producer,
            output_stream,
            resample_buffer: Vec::new(),
            resampler,
        })
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
        // helper method to split channels into separate vectors
        fn read_frames(inbuffer: &Vec<f32>, n_frames: usize, channels: usize) -> Vec<Vec<f32>> {
            let mut wfs = Vec::with_capacity(channels);
            for _chan in 0..channels {
                wfs.push(Vec::with_capacity(n_frames));
            }
            let mut value: f32;
            let mut inbuffer_iter = inbuffer.iter();
            for _ in 0..n_frames {
                for wf in wfs.iter_mut().take(channels) {
                    value = *inbuffer_iter.next().unwrap();
                    wf.push(value);
                }
            }
            wfs
        }

        /// Helper to merge channels into a single vector
        fn write_frames(waves: Vec<Vec<f32>>, outbuffer: &mut Vec<f32>, channels: usize) {
            let nbr = waves[0].len();
            for frame in 0..nbr {
                for chan in 0..channels {
                    let value = waves[chan][frame];
                    outbuffer.push(value);
                }
            }
        }

        if let Some(resampler) = &mut self.resampler {
            self.resample_buffer.extend_from_slice(data);
            // finish all the frames, as sometimes after appending many data
            // we might get 2 loops worth of unprocessed audio
            loop {
                let frames = resampler.input_frames_next();

                if self.resample_buffer.len() < frames * 2 {
                    return;
                }

                // only read the needed frames
                let input = read_frames(&mut self.resample_buffer, frames, 2);
                let output = resampler.process(&input, None).unwrap();

                let mut resampled = Vec::with_capacity(output[0].len() * 2);
                write_frames(output, &mut resampled, 2);

                self.buffer_producer.push_slice(&resampled);

                self.resample_buffer = self.resample_buffer.split_off(frames * 2);
            }
        } else {
            // no resampling
            self.buffer_producer.push_slice(data);
        }
    }
}

impl AudioPlayer {
    fn err_fn(err: cpal::StreamError) {
        eprintln!("an error occurred on audio stream: {}", err);
    }
}
