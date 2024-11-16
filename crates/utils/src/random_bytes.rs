use rand::{rngs::OsRng, RngCore};

pub fn random_128_bits() -> [u8; 16] {
    let mut data = [0u8; 16];
    OsRng.try_fill_bytes(&mut data).expect("Random generator failure");
    data
}
