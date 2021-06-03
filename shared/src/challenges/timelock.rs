use crate::std_alloc::Vec;
use anyhow::{anyhow, Result};
use byteorder::{ByteOrder, NetworkEndian};
use num_bigint::BigUint;

#[cfg(feature = "std")]
use glass_pumpkin::prime;
#[cfg(feature = "std")]
use rand::RngCore;

pub struct Timelock {
    a: BigUint,
    n: BigUint,
    squarings: u32,
}

impl Timelock {
    /// perform_challenge tries to find timelock puzzle solution by
    /// performing squaring `self.squarings` times.
    pub fn perform_challenge(&self) -> BigUint {
        let mut computation = self.a.clone();
        let two = BigUint::from(2 as u8);
        computation = computation.modpow(&two.pow(self.squarings), &self.n);
        computation
    }

    /// Serializes Timelock puzzle
    pub fn to_wire(&self) -> Vec<u8> {
        let a_bytes = self.a.to_bytes_be();
        let n_bytes = self.n.to_bytes_be();

        // Total length = <a length> + serialized a + <n length> + serialized n + <squarings>
        let mut result: Vec<u8> = vec![0; 8 + a_bytes.len() + 8 + n_bytes.len() + 4];
        // cursor is used to keep track of where to write the data in the buffer
        let mut cursor: usize = 0;

        NetworkEndian::write_u64(&mut result[cursor..], a_bytes.len() as u64);
        cursor += 8;
        for i in 0..a_bytes.len() {
            result[cursor] = a_bytes[i];
            cursor += 1;
        }

        NetworkEndian::write_u64(&mut result[cursor..], n_bytes.len() as u64);
        cursor += 8;
        for i in 0..n_bytes.len() {
            result[cursor] = n_bytes[i];
            cursor += 1;
        }

        NetworkEndian::write_u32(&mut result[cursor..], self.squarings);
        result
    }

    /// Deserializes Timelock puzzle
    pub fn from_wire(data: Vec<u8>) -> Result<Self> {
        // cursor is used to keep track of where to read the data in the buffer
        let mut cursor: usize = 0;
        let parsing_error = anyhow!("unable to parse wire data");
        let unexpected_data_error = anyhow!("expected EOF; found additional data instead.");

        let a_bytes_length = NetworkEndian::read_u64(&data[0..]) as usize;
        cursor += 8;
        // sanity check
        if a_bytes_length > (data.len() - cursor) {
            return Err(parsing_error);
        }
        let a_bytes = &data[cursor..(cursor + a_bytes_length)];
        cursor += a_bytes_length;

        let n_bytes_length = NetworkEndian::read_u64(&data[cursor..]) as usize;
        cursor += 8;
        // sanity check
        if n_bytes_length > (data.len() - cursor) {
            return Err(parsing_error);
        }
        let n_bytes = &data[cursor..(cursor + n_bytes_length)];
        cursor += n_bytes_length;

        let squaring = NetworkEndian::read_u32(&data[cursor..]);
        cursor += 4;

        if cursor != data.len() {
            return Err(unexpected_data_error);
        }

        Ok(Self {
            a: BigUint::from_bytes_be(a_bytes),
            n: BigUint::from_bytes_be(n_bytes),
            squarings: squaring,
        })
    }

    /// Generates new timelock puzzle using RNG supplied
    /// Since, we know secret primes `p` and `q` it is quite
    /// easy for us to compute the answer.
    /// But, since client does not know them, client need to do square of
    /// `a` repeatedly `squarings` time.
    #[cfg(feature = "std")]
    pub fn generate<RNG>(rng: &mut RNG, squarings: u32) -> (Self, TimelockVerifier)
    where
        RNG: RngCore,
    {
        let p = prime::from_rng(128, rng).unwrap();
        let q = prime::from_rng(128, rng).unwrap();

        let phi = (p.clone().sub(1 as u8)) * (q.clone().sub(1 as u8));
        let n = p * q;

        let mut a_bytes: Vec<u8> = vec![0; 20];
        rng.fill_bytes(&mut a_bytes);

        let a = BigUint::from_bytes_be(a_bytes.as_slice());

        let e = BigUint::from(2 as u8).modpow(&BigUint::from(squarings), &phi);
        let answer = a.modpow(&e, &n);

        (Timelock { a, n, squarings }, TimelockVerifier { answer })
    }
}

#[cfg(feature = "std")]
pub struct TimelockVerifier {
    answer: BigUint,
}

#[cfg(feature = "std")]
impl TimelockVerifier {
    pub fn verify(&self, client_response: BigUint) -> bool {
        self.answer.eq(&client_response)
    }
}

#[cfg(test)]
mod test {
    use crate::challenges::timelock::Timelock;
    use core::ops::{Add, Sub};
    use rand::rngs::OsRng;

    #[test]
    fn test_timelock_correctness() {
        let mut rng = OsRng::default();
        let (timelock, verifier) = Timelock::generate(&mut rng, 30);
        assert_eq!(timelock.perform_challenge(), verifier.answer);

        // Since, generated challenge are random, no two instances should be same.
        let (new_timelock, new_timelock_verifier) = Timelock::generate(&mut rng, 30);
        assert_ne!(new_timelock.perform_challenge(), verifier.answer);

        // Modifying any of the parameter result in different answer.

        // Modifying n
        let invalid_timelock = Timelock {
            a: new_timelock.a.clone(),
            n: new_timelock.n.clone().sub(1u8),
            squarings: 30,
        };
        assert_ne!(
            invalid_timelock.perform_challenge(),
            new_timelock_verifier.answer
        );

        // Modifying n
        let invalid_timelock = Timelock {
            a: new_timelock.a.clone().add(2u8),
            n: new_timelock.n.clone(),
            squarings: 30,
        };
        assert_ne!(
            invalid_timelock.perform_challenge(),
            new_timelock_verifier.answer
        );

        // Modifying squarings
        let invalid_timelock = Timelock {
            a: new_timelock.a,
            n: new_timelock.n,
            squarings: 31,
        };
        assert_ne!(
            invalid_timelock.perform_challenge(),
            new_timelock_verifier.answer
        );
    }

    #[test]
    fn test_timelock_to_wire_success() {
        let mut rng = OsRng::default();
        let (timelock, verifier) = Timelock::generate(&mut rng, 30);
        let wire_output = timelock.to_wire();
        let possible_constructed_timelock = Timelock::from_wire(wire_output);
        assert!(possible_constructed_timelock.is_ok());
        let constructed_timelock = possible_constructed_timelock.unwrap();
        assert_eq!(constructed_timelock.squarings, timelock.squarings);
        assert_eq!(constructed_timelock.a, timelock.a);
        assert_eq!(constructed_timelock.n, timelock.n);

        assert_eq!(constructed_timelock.perform_challenge(), verifier.answer);
    }

    #[test]
    fn test_timelock_to_wire_failure() {
        let mut rng = OsRng::default();
        let (timelock, verifier) = Timelock::generate(&mut rng, 30);
        let mut wire_output = timelock.to_wire();

        let mut first_wire_output = wire_output.clone();
        // Trying to modify a's length
        first_wire_output[2] = 45;
        let possible_constructed_timelock = Timelock::from_wire(first_wire_output.clone());
        assert!(!possible_constructed_timelock.is_ok());

        let mut second_clone_wire_output = wire_output.clone();
        // Modifying a itself
        second_clone_wire_output[8] = 20;
        second_clone_wire_output[9] = 21;
        second_clone_wire_output[10] = 22;
        let possible_constructed_timelock = Timelock::from_wire(second_clone_wire_output.clone());
        assert!(possible_constructed_timelock.is_ok());
        assert_ne!(
            possible_constructed_timelock.unwrap().perform_challenge(),
            verifier.answer
        );
    }
}
