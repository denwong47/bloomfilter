//! The condition to shift a filter in the rolling window.
//!

/// Default duration before shifting the filter.
///
/// This is set to 3600 seconds, or 1 hour.
pub const DEFAULT_SHIFT_DURATION: std::time::Duration = std::time::Duration::from_secs(3600);

/// Default number of insertions before shifting the filter.
///
/// This is set to 4096, which is valid for both 4 and 8 hash counts.
pub const DEFAULT_SHIFT_INSERTIONS: usize = 1 << 12;

pub trait ShiftCondition: Default {
    /// Whether the provided [`BloomFilter`] should be shifted.
    /// Typically this filter should be
    fn should_shift(&self) -> bool;

    /// Self mutation upon shift.
    fn do_shift(&mut self);

    /// Self mutation upon insertion.
    fn increment(&mut self);

    /// Increment itself, and return whether the filter should be shifted.
    fn should_shift_after_increment(&mut self) -> bool {
        self.increment();
        self.should_shift()
    }
}

/// Shift condition that shifts the filter after a certain duration.
#[derive(Debug, Clone)]
pub struct ShiftByDuration {
    pub duration: std::time::Duration,
    last_shift: std::time::Instant,
}

impl ShiftByDuration {
    /// Create a new shift condition that shifts the filter after a certain
    /// duration.
    pub fn new(duration: std::time::Duration) -> Self {
        Self {
            duration,
            last_shift: std::time::Instant::now(),
        }
    }
}

impl Default for ShiftByDuration {
    /// Create a new shift condition that shifts the filter after the default
    /// duration.
    fn default() -> Self {
        Self::new(DEFAULT_SHIFT_DURATION)
    }
}

impl ShiftCondition for ShiftByDuration {
    /// Whether the provided [`BloomFilter`] should be shifted.
    fn should_shift(&self) -> bool {
        self.last_shift.elapsed() > self.duration
    }

    /// Self mutation upon shift.
    fn do_shift(&mut self) {
        self.last_shift = std::time::Instant::now();
    }

    /// Self mutation upon insertion.
    fn increment(&mut self) {
        // Do nothing.
    }
}

/// Shift condition that shifts the filter after a certain number of insertions.
#[derive(Debug, Clone)]
pub struct ShiftByInsertions {
    pub insertions: usize,
    insertion_count: usize,
}

impl ShiftByInsertions {
    /// Create a new shift condition that shifts the filter after a certain
    /// number of insertions.
    pub fn new(limit: usize) -> Self {
        Self {
            insertions: limit,
            insertion_count: 0,
        }
    }
}

impl Default for ShiftByInsertions {
    /// Create a new shift condition that shifts the filter after the default
    /// number of insertions.
    fn default() -> Self {
        Self {
            insertions: DEFAULT_SHIFT_INSERTIONS,
            insertion_count: 0,
        }
    }
}

impl ShiftCondition for ShiftByInsertions {
    /// Whether the provided [`BloomFilter`] should be shifted.
    fn should_shift(&self) -> bool {
        // Since we increment before checking, we already need to shift if
        // the count is equal to the limit.
        self.insertion_count >= self.insertions
    }

    /// Self mutation upon shift.
    fn do_shift(&mut self) {
        self.insertion_count = 0;
    }

    /// Self mutation upon insertion.
    fn increment(&mut self) {
        self.insertion_count += 1;
    }
}
