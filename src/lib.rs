

pub mod true_rng{
    use candid::Principal;
    use ic_cdk::call;

    pub async fn async_generate() -> Result<usize, String >{
        let (random_bytes,): (Vec<u8>,) = call(Principal::management_canister(), "raw_rand", ()).await.map_err(|err| format!("{:?}", err))?;
        
        let random_number = usize::from_be_bytes(random_bytes[0..4].try_into().unwrap());
    
        Ok(random_number)
    }

    pub fn generate() -> Result<usize, String> {

        let future = async_generate(); // this creates a future
        let result = futures::executor::block_on(future); // blocks on the future

        result
    }
}

pub mod rng{
    use std::ops::Rem;
    use num_traits::{PrimInt, WrappingAdd, WrappingMul};
    pub fn random_seed() -> usize {
        // Use a combination of memory addresses, loop counters, or simple operations to generate variability
        let x = 42usize;
        let y = &x as *const usize as usize;  // Use the memory address of `x` as a factor
        let z = (y ^ 0xdeadbeef) + (y >> 3); // Apply some bitwise operations for variability
        z
    }
    pub struct RandomNumberGenerator<T>
    where
        T: PrimInt + WrappingAdd + WrappingMul + Rem<Output = T>,
    {
        seed: T,
        a: T, // Multiplier
        c: T, // Increment
        m: T, // Modulus
    }
    
    impl<T> RandomNumberGenerator<T>
    where
        T: PrimInt + WrappingAdd + WrappingMul + Rem<Output = T>,
    {
        /// Creates a new `RandomNumberGenerator` with user-provided `a`, `c`, and `m`.
        pub fn new_custom(seed: T, a: T, c: T, m: T) -> Self {
            RandomNumberGenerator { seed, a, c, m }
        }
    
        /// Creates a new `RandomNumberGenerator` with default values for `a`, `c`, and `m`.
        pub fn new() -> Self {
            // Default values for `a`, `c`, and `m` are based on typical LCG parameters for 32-bit numbers.
            let a = T::from(1664525).unwrap();
            let c = T::from(1013904223).unwrap();
            let m = T::from(2u64.pow(32)).unwrap(); // Use `u64` here since `T` could be larger than `u32`.
    
            RandomNumberGenerator { seed: T::from(random_seed()).unwrap(), a, c, m }
        }
    
        /// Generates the next random number in the sequence.
        pub fn next(&mut self) -> T {
            self.seed = self.a.wrapping_mul(&self.seed).wrapping_add(&self.c) % self.m;
            self.seed
        }
    
        /// Generates a random number in the range [0, max).
        pub fn range(&mut self, max: T) -> T {
            self.next() % max
        }
    }
}
