

pub mod true_rng{
    use candid::Principal;
    use ic_cdk::call;

    pub async fn async_generate() -> Result<usize, String >{
        let (random_bytes,): (Vec<u8>,) = call(Principal::management_canister(), "raw_rand", ()).await.map_err(|err| format!("{:?}", err))?;
        
        let random_number = usize::from_be_bytes(random_bytes[0..4.min(random_bytes.len())].try_into().unwrap());
    
        Ok(random_number)
    }

    pub fn generate() -> Result<usize, String> {

        let future = async_generate(); // this creates a future
        let result = futures::executor::block_on(future); // blocks on the future

        result
    }
}

pub mod rng{
    use std::ops::{Add, Mul, Rem};
    use std::num::Wrapping;
    use num_traits::{PrimInt, FromPrimitive, Unsigned, Bounded};

    pub fn random_seed() -> usize {
        let x = 42usize;
        let y = &x as *const usize as usize;
        let z = (y ^ 0xdeadbeef) + (y >> 3);
        z
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
