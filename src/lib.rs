

pub mod true_rng{
    use candid::Principal;
    use ic_cdk::call;

    pub async fn async_generate() -> Result<usize, String> {
        let (random_bytes,): (Vec<u8>,) = call(Principal::management_canister(), "raw_rand", ()).await.map_err(|err| format!("{:?}", err))?;
        
        // Determine the size of `usize` and pad the bytes accordingly
        const USIZE_SIZE: usize = std::mem::size_of::<usize>();
        let mut padded_bytes = [0u8; std::mem::size_of::<usize>()];
        
        let len = random_bytes.len().min(USIZE_SIZE);
        padded_bytes[USIZE_SIZE - len..USIZE_SIZE].copy_from_slice(&random_bytes[0..len]);
        
        let random_number = usize::from_be_bytes(padded_bytes);
    
        Ok(random_number)
    }

    pub fn generate() -> Result<usize, String> {

        let future = async_generate(); // this creates a future
        let result = futures::executor::block_on(future); // blocks on the future

        result
    }
}

pub mod rng{
    use std::hash::{DefaultHasher, Hash, Hasher};
    use std::ops::{Add, Mul, Rem};
    use std::num::Wrapping;
    use num_traits::{PrimInt, FromPrimitive, Unsigned, Bounded};

    pub fn random_seed() -> usize {
        let x = 42usize;
        let y = &x as *const usize as usize;
        let stack_value = &x as *const usize as usize;
        let stack_value2 = &y as *const usize as usize;
        
        // Combine memory address and hash
        let mut hasher = DefaultHasher::new();
        stack_value.hash(&mut hasher);
        stack_value2.hash(&mut hasher);
        ic_cdk::api::time().hash(&mut hasher);
        let hash = hasher.finish() as usize;
        
        // Use smaller constants and valid shifts
        let mut seed = y ^ (hash << 7) ^ (stack_value >> 3);
        seed = seed.wrapping_add(0x9e3779b9); // Golden ratio constant
        seed = seed ^ (seed >> 31); // Use shift within range
        seed = seed.wrapping_mul(0x85ebca6b); // Smaller multiplier
        seed = seed ^ (seed >> 31); // Use shift within range
        seed = seed.wrapping_mul(0xc2b2ae35); // Smaller multiplier
        seed = seed ^ (seed >> 31); // Use shift within range
    
        seed
    }
    
    pub struct RandomNumberGenerator<T>
    where
        T: PrimInt + FromPrimitive + Unsigned + Bounded + Mul<Output = T>,
    {
        seed: Wrapping<T>,
        a: Wrapping<T>, // Multiplier
        c: Wrapping<T>, // Increment
        m: Wrapping<T>, // Modulus
    }
    
    impl<T> RandomNumberGenerator<T>
    where
        T: PrimInt + FromPrimitive + Unsigned + Bounded ,
        Wrapping<T>: Mul<Output = Wrapping<T>> +  Add<Output = Wrapping<T>> + Rem<Output = Wrapping<T>>

    {
        /// Creates a new `RandomNumberGenerator` with user-provided `a`, `c`, and `m`.
        pub fn new_custom(seed: T, a: T, c: T, m: T) -> Self {
            RandomNumberGenerator {
                seed: Wrapping(seed),
                a: Wrapping(a),
                c: Wrapping(c),
                m: Wrapping(m),
            }
        }
    
        /// Creates a new `RandomNumberGenerator` with default values for `a`, `c`, and `m`.
        pub fn new() -> Self {
            let (a, c, m) = Self::default_values();
            RandomNumberGenerator {
                seed: Wrapping(T::from(random_seed()).unwrap_or_else(T::min_value)),
                a: Wrapping(a),
                c: Wrapping(c),
                m: Wrapping(m),
            }
        }
    
        /// Generates the next random number in the sequence.
        pub fn next(&mut self) -> T {
            self.seed = (self.a * self.seed + self.c) % self.m;
            self.seed.0
        }
    
        /// Generates a random number in the range [0, max).
        pub fn range(&mut self, max: T) -> T {
            self.next() % max
        }
    
        /// Choose default values based on the size of `T`.
        fn default_values() -> (T, T, T) {
            let bits = T::zero().count_zeros();
            match bits {
                8 => (
                    T::from(13).unwrap(),           // Smaller constants for `u8`
                    T::from(7).unwrap(),
                    T::from(31).unwrap(),           // Use a small modulus for `u8`
                ),
                16 => (
                    T::from(25173).unwrap(),        // Common values for `u16`
                    T::from(13849).unwrap(),
                    T::from(2u32.pow(16) - 1).unwrap(),        // 2^16
                ),
                32 => (
                    T::from(1664525).unwrap(),      // Larger constants for `u32`
                    T::from(1013904223).unwrap(),
                    T::from(2u64.pow(32) - 1).unwrap(),  // 2^32
                ),
                _ => (
                    T::from(1664525).unwrap(),      // Default values
                    T::from(1013904223).unwrap(),
                    T::from(2u64.pow(32)).unwrap(),
                )
            }
        }
    }
}
