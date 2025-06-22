/// A utility for implementing exponential backoff retry logic.
#[derive(Debug, Clone)]
pub struct ExponentialBackoff {
    retries: u8,
    min_secs: u32,
    max_secs: u32,
    factor: u32,
    counter: u8,
    value_secs: u32,
}

impl Default for ExponentialBackoff {
    /// Creates a default `ExponentialBackoff` instance with:
    /// - `retries` set to 10
    /// - `min_secs` set to 1 second
    /// - `max_secs` set to 60 seconds
    /// - `factor` set to 2 (doubling the backoff time)
    fn default() -> Self {
        Self::new(10, 1, 60, 2)
    }
}

impl ExponentialBackoff {
    /// Creates a new `ExponentialBackoff` instance with the specified
    /// parameters.
    pub fn new(retries: u8, min_secs: u32, max_secs: u32, factor: u32) -> Self {
        Self { retries, min_secs, max_secs, factor, counter: 0, value_secs: min_secs }
    }

    /// Resets the backoff counter to zero.
    pub fn reset(&mut self) {
        self.counter = 0;
        self.value_secs = 0;
    }

    /// Returns the current backoff value in seconds.
    pub fn value_secs(&self) -> u32 {
        self.value_secs
    }

    /// Get iteration count
    pub fn get_iteration_count(&self) -> u8 {
        self.counter
    }
}

impl Iterator for ExponentialBackoff {
    type Item = u32;

    /// Returns the next backoff value, incrementing the counter and applying
    /// exponential backoff logic.
    fn next(&mut self) -> Option<Self::Item> {
        // Check if we have reached the maximum number of retries
        // If retries is set to 9, it means unlimited number of retries
        // If retries is > 0 and we have reached or exceeded the retry limit, stop
        // iteration.
        if self.retries > 0 && self.counter >= self.retries {
            return None;
        }

        // Store the current value to return.
        let value = self.value_secs;
        self.counter += 1;
        self.value_secs = match self.counter {
            // First iteration, use minimum delay
            0 => self.min_secs,
            // Applu exponential backoff for subsequent iterations
            _ => {
                let next_value = self.value_secs * self.factor;
                if next_value > self.max_secs {
                    self.max_secs
                } else {
                    next_value
                }
            },
        };
        // Return the backoff value for this iteration.
        // The caller should wait for value seconds before the next retry
        Some(value)
    }
}
