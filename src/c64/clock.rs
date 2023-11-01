// timing clock structure
use std::time::Instant;

pub struct Clock {
    last_time: Instant,
    clock_period: f64,
}

impl Clock {
    pub fn new(freq: f64) -> Self {
        Clock {
            last_time: Instant::now(),
            clock_period: 1.0 / freq,
        }
    }

    pub fn tick(&mut self) -> bool {
        if self.last_time.elapsed().as_secs_f64() >= self.clock_period {
            self.last_time = Instant::now();
            true
        } else {
            false
        }
    }
}
