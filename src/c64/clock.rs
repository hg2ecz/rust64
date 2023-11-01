// timing clock structure
use std::time::Instant;

pub struct Clock {
    last_time: Instant,
    clock_period: f64,
    clock: u64,
    debug_error: [u64; 20],
    debug_sleep_sum: u64,
    debug_print: u64,
}

const DEBUG_CLK: u64 = (1.5 * 985248.0) as u64;

impl Clock {
    pub fn new(freq: f64) -> Self {
        Clock {
            last_time: Instant::now(),
            clock_period: 1.0 / freq,
            clock: 0,
            debug_error: [0; 20],
            debug_sleep_sum: 0,
            debug_print: 0,
        }
    }

    pub fn tick(&mut self) -> bool {
        let new_clock = (self.last_time.elapsed().as_secs_f64() / self.clock_period) as u64;
        if new_clock != self.clock {
            // Clock Debug
            let diff = new_clock - self.clock - 1;
            match diff {
                0..=19 => self.debug_error[diff as usize] += 1,
                _ => self.debug_error[19] += 1,
            }
            self.debug_sleep_sum += diff;
            if self.clock >= self.debug_print + DEBUG_CLK {
                self.debug_print += DEBUG_CLK;
                println!(
                    "Clock diag: {:?} error_sum: {} ({:.2}%))",
                    self.debug_error,
                    self.debug_sleep_sum,
                    100.0 * self.debug_sleep_sum as f64 / (self.debug_error[0] + self.debug_sleep_sum) as f64
                );
                for d in &mut self.debug_error {
                    *d = 0;
                }
                self.debug_sleep_sum = 0;
            }
            // Store new clock & return
            self.clock = new_clock;
            true
        } else {
            false
        }
    }
}
