pub fn xorshift64(seed: u64) -> u64 {
    let mut x = seed;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x
}

pub struct RandomNumberGenerator {
    pub seed: u64,
}

impl RandomNumberGenerator {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    pub fn next(&mut self) -> u64 {
        self.seed = xorshift64(self.seed);
        self.seed
    }

    pub fn roll_dice(&mut self, sides: usize) -> u64 {
        self.next() % (sides as u64) + 1
    }

    // Start inclusive, end exlusive
    pub fn range(&mut self, start: u64, end: u64) -> u64 {
        start + self.next() % (end - start)
    }
}

#[cfg(test)]
mod tests {
    use {super::*, proptest::prelude::*};

    proptest! {
        #[test]
        fn test_next(seed: u64) {
            let mut rng = RandomNumberGenerator::new(seed);
            prop_assert_eq!(rng.next(), xorshift64(seed));
        }

        #[test]
        fn test_roll_dice(seed: u64, sides in 1usize..100) {
            let mut rng = RandomNumberGenerator::new(seed);
            let result = rng.roll_dice(sides);
            prop_assert!(result >= 1 && result <= (sides as u64));
        }

        #[test]
        fn test_range(seed: u64, start in 1u64..100, end in 101u64..200) {
            let mut rng = RandomNumberGenerator::new(seed);
            let result = rng.range(start, end);
            prop_assert!(result >= start && result < end);
        }
    }
}
