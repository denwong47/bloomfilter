//! The base implementation of the [`BloomFilter`] and [`BloomHash`] types.

pub struct BloomHashCounter<const N: usize> {}

/// Marker trait for `const`s that can be used as a Bloom filter hash function.
pub trait BloomHashCount {}

// impl BloomHashCount for BloomHashCounter<2> {}
impl BloomHashCount for BloomHashCounter<4> {}
impl BloomHashCount for BloomHashCounter<8> {}

pub const SEED: u128 = 127;

pub struct BloomHash<const N: usize, const S: i64 = 0>
where
    BloomHashCounter<N>: BloomHashCount,
{
    pub hashes: Box<[usize]>,
}

impl<const N: usize, const S: i64, T> From<T> for BloomHash<N, S>
where
    BloomHashCounter<N>: BloomHashCount,
    T: AsRef<[u8]>,
{
    fn from(value: T) -> Self {
        let hashed = gxhash::gxhash128(value.as_ref(), 0);
        let mask = (1 << (128 / N)) - 1;

        Self {
            hashes: (0..N)
                .map(|i| (hashed >> (i * (128 / N)) & mask) as usize)
                .collect::<Box<[_]>>(),
        }
    }
}

pub struct BloomFilter<const N: usize, const S: i64 = 0>
where
    BloomHashCounter<N>: BloomHashCount,
{
    pub bits: Box<[bool]>,
}

impl<const N: usize, const S: i64> Default for BloomFilter<N, S>
where
    BloomHashCounter<N>: BloomHashCount,
{
    /// Create a new Bloom Filter.
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize, const S: i64> BloomFilter<N, S>
where
    BloomHashCounter<N>: BloomHashCount,
{
    /// Create a new Bloom Filter.
    pub fn new() -> Self {
        Self {
            bits: vec![false; 1 << (128 / N)].into_boxed_slice(),
        }
    }

    /// Add a hash to the Bloom Filter.
    pub fn add_hash(&mut self, hash: &BloomHash<N, S>) {
        hash.hashes.iter().for_each(|i| {
            self.bits[*i] = true;
        });
    }

    /// Add a value to the Bloom Filter.
    pub fn add(&mut self, value: impl AsRef<[u8]>) {
        let hash = BloomHash::<N, S>::from(value);

        self.add_hash(&hash);
    }

    /// Check if a value is a member of the Bloom Filter.
    ///
    /// This can only return false positives, not false negatives.
    pub fn contains(&self, value: &[u8]) -> bool {
        let hash: BloomHash<N> = From::<&[u8]>::from(value);

        hash.hashes.iter().all(|i| self.bits[*i])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! expand_n {
        ($(($name:ident, $n:literal)),*$(,)?) => {
            $(
                #[test]
                fn $name() {
                    let mut filter = BloomFilter::<$n>::new();

                    filter.add(b"hello");
                    filter.add(b"world");

                    assert!(filter.contains(b"hello"));
                    assert!(filter.contains(b"world"));

                    assert!(!filter.contains(b"foo"));
                    assert!(!filter.contains(b"bar"));
                }
            )*

    }}

    expand_n!(
        // (n2, 2),
        (n4, 4),
        (n8, 8),
    );
}
