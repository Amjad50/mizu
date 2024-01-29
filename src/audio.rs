use std::error::Error;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{HeapProducer, HeapRb};
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
    buffer_producer: HeapProducer<f32>,
    resampler: Option<FftFixedInOut<f32>>,
    pre_resampled_buffer: Vec<f32>,
    pre_resampled_split_buffers: [Vec<f32>; 2],
    resample_process_buffers: [Vec<f32>; 2],
    resampled_buffer: Vec<f32>,
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
                        sample_rate.0 as usize / 60,
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
        let buffer = HeapRb::new(output_sample_rate.0 as usize / 2);
        let (buffer_producer, mut buffer_consumer) = buffer.split();

        let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for sample in data {
                // /5.5 to reduce the volume of the sample
                *sample = buffer_consumer.pop().unwrap_or(0.) / 5.5;
            }
        };

        let output_stream = output_device
            .build_output_stream(&config, output_data_fn, Self::err_fn, None)
            .expect("failed to build an output audio stream");

        Ok(Self {
            buffer_producer,
            output_stream,
            pre_resampled_buffer: Vec::new(),
            pre_resampled_split_buffers: [Vec::new(), Vec::new()],
            resample_process_buffers: [Vec::new(), Vec::new()],
            resampled_buffer: Vec::new(),
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
        fn read_frames(inbuffer: &[f32], n_frames: usize, outputs: &mut [Vec<f32>]) {
            for output in outputs.iter_mut() {
                output.clear();
                output.reserve(n_frames);
            }
            let mut value: f32;
            let mut inbuffer_iter = inbuffer.iter();
            for _ in 0..n_frames {
                for output in outputs.iter_mut() {
                    value = *inbuffer_iter.next().unwrap();
                    output.push(value);
                }
            }
        }

        /// Helper to merge channels into a single vector
        /// the number of channels is the size of `waves` slice
        fn write_frames(waves: &[Vec<f32>], outbuffer: &mut Vec<f32>) {
            let nbr = waves[0].len();
            for frame in 0..nbr {
                for wave in waves.iter() {
                    outbuffer.push(wave[frame]);
                }
            }
        }

        if let Some(resampler) = &mut self.resampler {
            self.pre_resampled_buffer.extend_from_slice(data);
            // finish all the frames, as sometimes after appending many data
            // we might get 2 loops worth of unprocessed audio
            loop {
                let frames = resampler.input_frames_next();

                if self.pre_resampled_buffer.len() < frames * 2 {
                    return;
                }

                // only read the needed frames
                read_frames(
                    &self.pre_resampled_buffer,
                    frames,
                    &mut self.pre_resampled_split_buffers,
                );

                self.resample_process_buffers[0].clear();
                self.resample_process_buffers[0].clear();

                let output_frames = resampler.output_frames_next();
                self.resample_process_buffers[0].reserve(output_frames);
                self.resample_process_buffers[1].reserve(output_frames);

                resampler
                    .process_into_buffer(
                        &self.pre_resampled_split_buffers,
                        &mut self.resample_process_buffers,
                        None,
                    )
                    .unwrap();

                if self.resampled_buffer.len() < output_frames * 2 {
                    self.resampled_buffer
                        .reserve(output_frames * 2 - self.resampled_buffer.len());
                }
                self.resampled_buffer.clear();
                write_frames(&self.resample_process_buffers, &mut self.resampled_buffer);

                self.buffer_producer.push_slice(&self.resampled_buffer);

                self.pre_resampled_buffer = self.pre_resampled_buffer.split_off(frames * 2);
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
