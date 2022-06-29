use crate::cipher::Arc4;
use std::cmp;

/// An ARC4-based PRNG.
///
/// Period: ~2^1600
pub(crate) struct Prng {
    cipher: Arc4,
}

// See https://github.com/davidbau/seedrandom/blob/released/seedrandom.js
impl Prng {
    /// Initializes the PRNG with the given seed.
    pub(crate) fn with_seed(seed: &[u8]) -> Self {
        let key = mixkey(seed);
        let mut cipher = Arc4::with_key(&key);

        // For robust unpredictability, we discards an initial batch of values.
        // This is called RC4-drop[n].
        // See https://en.wikipedia.org/wiki/RC4#Fluhrer,_Mantin_and_Shamir_attack
        let mut sink = [0; 256];
        cipher.prga(&mut sink);

        Self { cipher }
    }

    /// Returns a random double in [0, 1).
    #[allow(clippy::cast_precision_loss)] // Messy but we know what we're doing.
    pub(crate) fn rand(&mut self) -> f64 {
        // f64 has a 52-bit significand (excluding the hidden bit).
        const SIGNIFICAND: u64 = 1 << 52;
        const OVERFLOW: u64 = 1 << 53;
        // Each RC4 output is 0 <= x < 256.
        const WIDTH: u64 = 256;

        // Start with a numerator < 2 ^ 48, denominator = 2 ^ 48 and no 'extra
        // last byte'.
        let mut numerator = self.rand48();
        // XXX: Use f64 to avoid denominator overflow + mimic original JS code.
        let mut denominator = (1u64 << 48) as f64;
        let mut lsb = 0;

        // Fill up all significant digits by shifting numerator and denominator
        // and generating a new least-significant-byte.
        while numerator < SIGNIFICAND {
            numerator = (numerator + u64::from(lsb)) * WIDTH;
            denominator *= WIDTH as f64;
            lsb = self.rand8();
        }

        // To avoid rounding up, before adding last byte, shift everything right
        // using integer math until we have exactly the desired bits.
        while numerator >= OVERFLOW {
            numerator >>= 1;
            denominator /= 2.;
            lsb >>= 1;
        }

        // Form the number within [0, 1).
        (numerator + u64::from(lsb)) as f64 / denominator
    }

    /// Returns a random 48-bit integer.
    fn rand48(&mut self) -> u64 {
        let mut bytes = [0; 6];
        self.cipher.prga(&mut bytes);

        u64::from(bytes[5])
            | (u64::from(bytes[4]) << 8)
            | (u64::from(bytes[3]) << 16)
            | (u64::from(bytes[2]) << 24)
            | (u64::from(bytes[1]) << 32)
            | (u64::from(bytes[0]) << 40)
    }

    /// Returns a random 8-bit integer.
    fn rand8(&mut self) -> u8 {
        let mut bytes = [0; 1];
        self.cipher.prga(&mut bytes);

        bytes[0]
    }
}

/// Mixes a seed into a shortened bytestring key.
#[allow(clippy::cast_possible_truncation)] // Force wraparound at 256 for index.
fn mixkey(key: &[u8]) -> Vec<u8> {
    let mut out = vec![0; cmp::min(key.len(), 256)];

    // Use the simplified algo from catsital/pycasso
    // instead of the original one from davidbau/seedrandom
    for (i, byte) in key.iter().enumerate() {
        out[usize::from(i as u8)] = *byte;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixkey_short() {
        let key = "seed".as_bytes();
        let res = mixkey(key);

        assert_eq!(res, [b's', b'e', b'e', b'd']);
    }

    #[test]
    fn mixkey_long() {
        let mut key = vec![b'1'; 255];
        key.push(b'2');
        key.push(b'2');
        let mut expected = vec![b'1'; 256];
        expected[255] = b'2';
        expected[0] = b'2'; // Wraparound

        let res = mixkey(&key);

        assert_eq!(res, expected);
    }

    #[test]
    fn rand() {
        let mut prng = Prng::with_seed(b"braque");
        // Generated using catsital/pycasso implementation.
        let expected = [
            0.12063955304223144,
            0.5166808775087299,
            0.15514044584437084,
            0.9418555792052827,
            0.5805404063693996,
            0.5518369778087185,
            0.5411486504395583,
            0.32282658448360363,
            0.7672009436485945,
            0.6170751309139755,
        ];

        for value in expected {
            assert_eq!(prng.rand(), value);
        }
    }
}
