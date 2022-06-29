use crate::prng::Prng;

/// Array shuffle using the given seed.
pub(crate) fn shuffle<T: Copy>(arr: &[T], seed: &[u8]) -> Vec<T> {
    // See https://github.com/webcaetano/shuffle-seed/blob/master/shuffle-seed.js
    let mut prng = Prng::with_seed(seed);
    let mut keys = arr.iter().enumerate().map(|(i, _)| i).collect::<Vec<_>>();

    (0..arr.len())
        .map(|_| {
            let idx = pop_rand(&mut keys, &mut prng);
            arr[idx]
        })
        .collect()
}

/// Array unshuffle using the given seed.
pub(crate) fn unshuffle<T: Copy>(arr: &[T], seed: &[u8]) -> Vec<T> {
    // See https://github.com/webcaetano/shuffle-seed/blob/master/shuffle-seed.js
    let mut prng = Prng::with_seed(seed);
    let mut keys = arr.iter().enumerate().map(|(i, _)| i).collect::<Vec<_>>();
    let mut res = arr.to_vec();

    for value in arr {
        let idx = pop_rand(&mut keys, &mut prng);
        res[idx] = *value;
    }

    res
}

/// Remove a random item from the array.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)] // Safe given the ranges involved.
fn pop_rand(arr: &mut Vec<usize>, prng: &mut Prng) -> usize {
    let idx = (prng.rand() * (arr.len() as f64)).floor() as usize;
    arr.remove(idx)
}
#[cfg(test)]
mod tests {
    #[test]
    fn shuffle() {
        let arr = b"Pycasso";
        let seed = b"Pycasso";
        let res = super::shuffle(arr, seed);

        assert_eq!(res, b"cPosysa");
    }

    #[test]
    fn unshuffle() {
        let arr = b"cPosysa";
        let seed = b"Pycasso";
        let res = super::unshuffle(arr, seed);

        assert_eq!(res, b"Pycasso");
    }
}
