use crate::std_alloc::Vec;

#[cfg(feature = "std")]
use rand::RngCore;

#[cfg(feature = "std")]
use sha2::{Digest, Sha256};

#[cfg(feature = "std")]
mod roundtrip_utils {
    use crate::std_alloc::Vec;
    use rand::RngCore;

    pub const KB: usize = 1024;
    pub const MB: usize = 1024 * KB;

    pub fn generate_random_data_mb<RNG>(rng: &mut RNG, megabytes: usize) -> Vec<u8>
    where
        RNG: RngCore,
    {
        generate_data(rng, megabytes * MB)
    }

    pub fn generate_random_data_kb<RNG>(rng: &mut RNG, kilobytes: usize) -> Vec<u8>
    where
        RNG: RngCore,
    {
        generate_data(rng, kilobytes * KB)
    }

    fn generate_data<RNG>(rng: &mut RNG, bytes: usize) -> Vec<u8>
    where
        RNG: RngCore,
    {
        let mut data = vec![0 as u8; bytes];
        rng.fill_bytes(&mut data);
        data
    }
}

pub struct Roundtrip {
    data: Vec<u8>,
}

impl Roundtrip {
    /// Generate new roundtrip data using RNG provided, verifier contains SHA2 hash of the data
    /// which will be used to check integrity of data we get back from the client.
    /// Verifier contains SHA2 hash, which is used to verify that client returned same data.
    #[cfg(feature = "std")]
    pub fn generate<RNG>(rng: &mut RNG, size_in_kbs: usize) -> (Self, RoundtripVerifier)
    where
        RNG: RngCore,
    {
        let me = Roundtrip {
            data: roundtrip_utils::generate_random_data_kb(rng, size_in_kbs),
        };

        let mut hasher = Sha256::new();
        hasher.update(&me.data);

        (
            me,
            RoundtripVerifier {
                hash: hasher.finalize().as_slice().to_vec(),
            },
        )
    }

    /// Serializes Roundtrip data
    /// Since there is no processing involved we are consuming `self` and returning inner data
    pub fn to_wire(self) -> Vec<u8> {
        // We are taking self to avoid copying data
        self.data
    }

    /// Deserializes Roundtrip data
    /// Consumes `data` argument and returns Roundtrip
    pub fn from_wire(data: Vec<u8>) -> Self {
        Self { data }
    }
}

#[cfg(feature = "std")]
pub struct RoundtripVerifier {
    hash: Vec<u8>,
}

#[cfg(feature = "std")]
impl RoundtripVerifier {
    pub fn verify(&self, client_response: Vec<u8>) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(&client_response);
        hasher.finalize().to_vec().eq(&self.hash)
    }
}

#[cfg(test)]
mod tests {
    use crate::challenges::roundtrip::Roundtrip;
    use rand::rngs::OsRng;

    #[test]
    fn test_roundtrip_verifier() {
        let mut rng = OsRng::default();
        let (roundtrip, roundtrip_verifier) = Roundtrip::generate(&mut rng, 1024);
        assert_eq!(roundtrip.data.len(), 1024 * 1024);
        assert!(roundtrip_verifier.verify(roundtrip.data.clone()));
        // Let's pass data which is different by one byte
        let mut invalid_data = roundtrip.data.clone();
        invalid_data[2] = 5;
        assert!(!roundtrip_verifier.verify(invalid_data));
    }
}
