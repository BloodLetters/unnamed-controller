pub mod scheduler;

pub use scheduler::{EventId, EventScheduler, SimTime};

/// Basic placeholder for clock and delta-time scheduling logic.
pub struct Clock {
    pub ticks: u64,
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock {
    /// Constructs a clock starting at tick zero.
    pub fn new() -> Self {
        Self { ticks: 0 }
    }

    /// Advances the clock by one tick.
    pub fn tick(&mut self) {
        self.ticks += 1;
    }
}
