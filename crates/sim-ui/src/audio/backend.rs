use sim_core::audio::AudioRingBuffer;
use std::sync::{Arc, Mutex};

/// Cross-platform audio output device manager.
pub struct AudioOutputDevice {
    pub ring_buffer: Arc<Mutex<AudioRingBuffer>>,
    #[cfg(not(target_arch = "wasm32"))]
    _stream: Option<cpal::Stream>,
    pub is_active: bool,
    pub sample_rate: u32,
}

impl AudioOutputDevice {
    /// Creates and initializes the audio output streaming backend.
    pub fn new(capacity: usize) -> Self {
        let ring_buffer = Arc::new(Mutex::new(AudioRingBuffer::new(capacity)));

        #[cfg(not(target_arch = "wasm32"))]
        {
            let (stream, sample_rate) = init_cpal_stream(Arc::clone(&ring_buffer));
            let is_active = stream.is_some();
            Self {
                ring_buffer,
                _stream: stream,
                is_active,
                sample_rate,
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            Self {
                ring_buffer,
                is_active: false,
                sample_rate: 44100,
            }
        }
    }

    /// Pushes a slice of audio samples to the audio device ring buffer.
    pub fn write_samples(&self, samples: &[f32]) {
        if let Ok(mut rb) = self.ring_buffer.lock() {
            rb.write_slice(samples);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn init_cpal_stream(ring_buffer: Arc<Mutex<AudioRingBuffer>>) -> (Option<cpal::Stream>, u32) {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

    let host = cpal::default_host();
    let device = match host.default_output_device() {
        Some(d) => d,
        None => return (None, 44100),
    };

    let config = match device.default_output_config() {
        Ok(c) => c,
        Err(_) => return (None, 44100),
    };

    let sample_rate = config.sample_rate().0;
    let channels = config.channels() as usize;
    let stream_config: cpal::StreamConfig = config.into();

    let err_fn = |err| {
        eprintln!("Audio output error: {}", err);
    };

    let rb_clone = Arc::clone(&ring_buffer);
    let stream_res = device.build_output_stream(
        &stream_config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let mut temp = vec![0.0; data.len() / channels.max(1)];
            if let Ok(mut rb) = rb_clone.lock() {
                rb.read_slice(&mut temp);
            }
            for (frame_idx, &sample) in temp.iter().enumerate() {
                for ch in 0..channels {
                    let out_idx = frame_idx * channels + ch;
                    if out_idx < data.len() {
                        data[out_idx] = sample;
                    }
                }
            }
        },
        err_fn,
        None,
    );

    match stream_res {
        Ok(s) => {
            if s.play().is_ok() {
                (Some(s), sample_rate)
            } else {
                (None, sample_rate)
            }
        }
        Err(_) => (None, sample_rate),
    }
}
