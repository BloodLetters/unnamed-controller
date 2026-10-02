use std::sync::atomic::{AtomicUsize, Ordering};

/// Lock-free single-producer single-consumer circular buffer for audio sample streams.
pub struct AudioRingBuffer {
    buffer: Vec<f32>,
    capacity: usize,
    write_head: AtomicUsize,
    read_head: AtomicUsize,
}

impl AudioRingBuffer {
    /// Creates an audio ring buffer with specified maximum sample capacity.
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        Self {
            buffer: vec![0.0; capacity],
            capacity,
            write_head: AtomicUsize::new(0),
            read_head: AtomicUsize::new(0),
        }
    }

    /// Pushes a single audio sample into the circular buffer.
    pub fn push(&mut self, sample: f32) -> bool {
        let w = self.write_head.load(Ordering::Relaxed);
        let r = self.read_head.load(Ordering::Relaxed);
        let next_w = (w + 1) % self.capacity;
        if next_w == r {
            return false;
        }
        self.buffer[w] = sample;
        self.write_head.store(next_w, Ordering::Release);
        true
    }

    /// Pushes a slice of audio samples, discarding oldest if buffer fills.
    pub fn write_slice(&mut self, samples: &[f32]) -> usize {
        let mut written = 0;
        for &sample in samples {
            if self.push(sample) {
                written += 1;
            } else {
                break;
            }
        }
        written
    }

    /// Reads samples into the destination slice, padding with 0.0 on underflow.
    pub fn read_slice(&mut self, dest: &mut [f32]) -> usize {
        let mut read_count = 0;
        let mut r = self.read_head.load(Ordering::Relaxed);
        let w = self.write_head.load(Ordering::Acquire);

        for out in dest.iter_mut() {
            if r != w {
                *out = self.buffer[r];
                r = (r + 1) % self.capacity;
                read_count += 1;
            } else {
                *out = 0.0;
            }
        }
        self.read_head.store(r, Ordering::Release);
        read_count
    }

    /// Returns the number of available samples ready to be read.
    pub fn available_read(&self) -> usize {
        let w = self.write_head.load(Ordering::Acquire);
        let r = self.read_head.load(Ordering::Relaxed);
        if w >= r {
            w - r
        } else {
            self.capacity - (r - w)
        }
    }

    /// Clears all pending samples from the ring buffer.
    pub fn clear(&mut self) {
        self.write_head.store(0, Ordering::Relaxed);
        self.read_head.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_write_and_read() {
        let mut rb = AudioRingBuffer::new(8);
        assert_eq!(rb.available_read(), 0);

        let written = rb.write_slice(&[0.1, 0.2, 0.3]);
        assert_eq!(written, 3);
        assert_eq!(rb.available_read(), 3);

        let mut out = [0.0; 4];
        let read = rb.read_slice(&mut out);
        assert_eq!(read, 3);
        assert_eq!(out[0], 0.1);
        assert_eq!(out[1], 0.2);
        assert_eq!(out[2], 0.3);
        assert_eq!(out[3], 0.0);
        assert_eq!(rb.available_read(), 0);
    }
}
