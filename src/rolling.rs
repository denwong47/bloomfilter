//! Rolling window of Bloom Filters implementation.
//!

use crate::BloomHash;

use super::{BloomFilter, BloomHashCount, BloomHashCounter, ShiftCondition};

/// A rolling window of 2 Bloom Filters.
pub struct RollingBloomFilter<const N: usize, T, const S: i64 = 0>
where
    BloomFilter<N, S>: Default,
    BloomHashCounter<N>: BloomHashCount,
    T: ShiftCondition,
{
    filters: [BloomFilter<N, S>; 2],
    shift_condition: T,
}

impl<const N: usize, T, const S: i64> Default for RollingBloomFilter<N, T, S>
where
    BloomFilter<N, S>: Default,
    BloomHashCounter<N>: BloomHashCount,
    T: Default + ShiftCondition,
{
    fn default() -> Self {
        Self {
            filters: [BloomFilter::default(), BloomFilter::default()],
            shift_condition: T::default(),
        }
    }
}

impl<const N: usize, T, const S: i64> RollingBloomFilter<N, T, S>
where
    BloomFilter<N, S>: Default,
    BloomHashCounter<N>: BloomHashCount,
    T: ShiftCondition,
{
    /// Create a new rolling window of Bloom Filters with the provided shift condition.
    pub fn new(shift_condition: T) -> Self {
        Self {
            filters: [BloomFilter::default(), BloomFilter::default()],
            shift_condition,
        }
    }

    /// Add an element to the rolling window of Bloom Filters.
    pub fn add(&mut self, value: &[u8]) {
        let hash: BloomHash<N, S> = value.into();

        self.filters[0].add_hash(&hash);
        self.filters[1].add_hash(&hash);

        // Shift the filter if the condition is met.
        if self.shift_condition.should_shift_after_increment() {
            self.shift()
        }
    }

    /// Shift the rolling window of Bloom Filters.
    pub fn shift(&mut self) {
        self.shift_condition.do_shift();

        // Replace the oldest filter with a new one.
        self.filters[0] = BloomFilter::default();

        // Swap the filters so that the oldest filter is always the first one.
        self.filters.swap(0, 1);
    }

    /// Check if an element is a member of the rolling window of Bloom Filters.
    pub fn contains(&self, value: &[u8]) -> bool {
        // We only need to check the oldest filter.
        self.filters[0].contains(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{ShiftByDuration, ShiftByInsertions};

    use super::*;

    #[test]
    fn shift_by_insertions() {
        let shift_condition = ShiftByInsertions::new(3);
        let mut rolling_bloom_filter = RollingBloomFilter::<4, _>::new(shift_condition);

        rolling_bloom_filter.add("hello".as_bytes());
        rolling_bloom_filter.add("world".as_bytes());

        assert!(rolling_bloom_filter.contains("hello".as_bytes()));
        assert!(rolling_bloom_filter.contains("world".as_bytes()));
        assert!(!rolling_bloom_filter.contains("foo".as_bytes()));

        rolling_bloom_filter.add("foo".as_bytes());
        rolling_bloom_filter.add("bar".as_bytes());

        assert!(rolling_bloom_filter.contains("foo".as_bytes()));
        assert!(rolling_bloom_filter.contains("bar".as_bytes()));
        assert!(rolling_bloom_filter.contains("hello".as_bytes()));
        assert!(rolling_bloom_filter.contains("world".as_bytes()));

        rolling_bloom_filter.add("baz".as_bytes());
        rolling_bloom_filter.add("qux".as_bytes());

        assert!(rolling_bloom_filter.contains("baz".as_bytes()));
        assert!(rolling_bloom_filter.contains("qux".as_bytes()));
        assert!(rolling_bloom_filter.contains("bar".as_bytes()));

        // The oldest filter should have been shifted, resulting in the first
        // three insertions being removed from the filter.
        assert!(!rolling_bloom_filter.contains("foo".as_bytes()));
        assert!(!rolling_bloom_filter.contains("hello".as_bytes()));
        assert!(!rolling_bloom_filter.contains("world".as_bytes()));

        rolling_bloom_filter.add("quack".as_bytes());
        assert!(rolling_bloom_filter.contains("quack".as_bytes()));
        assert!(rolling_bloom_filter.contains("bar".as_bytes()));
    }

    #[test]
    fn shift_by_duration() {
        let shift_condition = ShiftByDuration::new(std::time::Duration::from_millis(100));
        let mut rolling_bloom_filter = RollingBloomFilter::<4, _>::new(shift_condition);

        rolling_bloom_filter.add("hello".as_bytes());
        rolling_bloom_filter.add("world".as_bytes());

        assert!(rolling_bloom_filter.contains("hello".as_bytes()));
        assert!(rolling_bloom_filter.contains("world".as_bytes()));
        assert!(!rolling_bloom_filter.contains("foo".as_bytes()));

        std::thread::sleep(std::time::Duration::from_millis(100));

        rolling_bloom_filter.add("foo".as_bytes());
        rolling_bloom_filter.add("bar".as_bytes());

        assert!(rolling_bloom_filter.contains("foo".as_bytes()));
        assert!(rolling_bloom_filter.contains("bar".as_bytes()));
        assert!(rolling_bloom_filter.contains("hello".as_bytes()));
        assert!(rolling_bloom_filter.contains("world".as_bytes()));

        std::thread::sleep(std::time::Duration::from_millis(100));

        rolling_bloom_filter.add("baz".as_bytes());

        assert!(rolling_bloom_filter.contains("baz".as_bytes()));
        assert!(rolling_bloom_filter.contains("bar".as_bytes()));

        // `foo` won't exist because `foo` was added before the shift, and the
        // oldest filter was shifted.
        assert!(!rolling_bloom_filter.contains("foo".as_bytes()));
        assert!(!rolling_bloom_filter.contains("hello".as_bytes()));
        assert!(!rolling_bloom_filter.contains("world".as_bytes()));
    }
}
