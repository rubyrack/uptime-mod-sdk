//! The mod's random stream. Seeded from `quantum.seed`, so a replay draws the
//! same numbers; a mod must not read any other entropy.

/// SplitMix64: a 64-bit state, one multiply-xorshift round per draw.
#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// A stream that depends on `seed` alone.
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// The next 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// The high 32 bits of the next draw.
    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    /// Uniform in `0..n` without modulo bias. `n == 0` returns 0.
    pub fn below(&mut self, n: u64) -> u64 {
        if n == 0 {
            return 0;
        }
        let zone = u64::MAX - (u64::MAX % n);
        loop {
            let x = self.next_u64();
            if x < zone {
                return x % n;
            }
        }
    }

    /// Uniform in `[0, 1)` with 53 bits of precision.
    pub fn unit_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// True with probability `p`.
    pub fn chance(&mut self, p: f64) -> bool {
        self.unit_f64() < p
    }

    /// One element of `items`, or `None` when it is empty.
    pub fn choose<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        if items.is_empty() {
            None
        } else {
            items.get(self.below(items.len() as u64) as usize)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Rng;

    #[test]
    fn the_stream_is_a_pure_function_of_the_seed() {
        let a: Vec<u64> = {
            let mut r = Rng::new(42);
            (0..4).map(|_| r.next_u64()).collect()
        };
        let b: Vec<u64> = {
            let mut r = Rng::new(42);
            (0..4).map(|_| r.next_u64()).collect()
        };
        assert_eq!(a, b);
        assert_eq!(
            a[0], 0xBDD7_3226_2FEB_6E95,
            "SplitMix64 reference value for seed 42"
        );
        let mut other = Rng::new(43);
        assert_ne!(a[0], other.next_u64());
    }

    #[test]
    fn below_stays_in_range_and_covers_it() {
        let mut r = Rng::new(7);
        let mut seen = [0u32; 5];
        for _ in 0..1000 {
            seen[r.below(5) as usize] += 1;
        }
        assert!(seen.iter().all(|&n| n > 100), "{seen:?}");
        assert_eq!(r.below(0), 0);
        assert_eq!(r.choose::<u8>(&[]), None);
    }
}
