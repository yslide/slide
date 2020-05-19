use num_traits::{PrimInt, Unsigned};

/// Calculates the GCD for (u, v) ∈ (Z, Z).
///
/// Currently, a binary GCD method is used as an underlying implementation.
///
/// ```text
/// binary_gcd              time:   [2.4969 ns 2.5894 ns 2.6641 ns]
///
/// euclidean_gcd           time:   [2.8543 ns 2.8948 ns 2.9257 ns]
/// ```
#[allow(unused)]
pub fn gcd<N: Unsigned + PrimInt>(u: N, v: N) -> N {
    euclidean_gcd(u, v)
}

/// The [Binary GCD] algorithm, or Stein's algorithm.
/// Implemented ∀ (u, v) ∈ (Z, Z).
///
/// [Binary GCD]: https://en.wikipedia.org/wiki/Binary_GCD_algorithm
#[allow(unused)]
fn binary_gcd<N: Unsigned + PrimInt>(mut u: N, mut v: N) -> N {
    if u == N::zero() {
        return u;
    }
    if v == N::zero() {
        return v;
    }

    let shift_back = (u | v).trailing_zeros() as usize;
    u = u >> u.trailing_zeros() as usize;
    v = v >> v.trailing_zeros() as usize;
    if u > v {
        std::mem::swap(&mut u, &mut v);
    }
    v = v - u;
    while v != N::zero() {
        v = v >> v.trailing_zeros() as usize;
        if u > v {
            std::mem::swap(&mut u, &mut v);
        }
        v = v - u;
    }
    u << shift_back
}

#[cfg(feature = "benchmark-internals")]
pub fn _binary_gcd<N: Unsigned + PrimInt>(u: N, v: N) -> N {
    binary_gcd(u, v)
}

/// The [Euclidean GCD] algorithm.
/// Implemented ∀ (u, v) ∈ (Z, Z).
///
/// [Euclidean GCD]: https://en.wikipedia.org/wiki/Euclidean_algorithm
#[allow(unused)]
fn euclidean_gcd<N: Unsigned + PrimInt>(mut u: N, mut v: N) -> N {
    let mut t;
    while !v.is_zero() {
        t = v;
        v = u % v;
        u = t;
    }
    u
}

#[cfg(feature = "benchmark-internals")]
pub fn _euclidean_gcd<N: Unsigned + PrimInt>(u: N, v: N) -> N {
    euclidean_gcd(u, v)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CASES: [(u32, u32, u32); 7] = [
        (13, 13, 13),
        (37, 600, 1),
        (20, 100, 20),
        (624_129, 2_061_517, 18_913),
        (600, 37, 1),
        (100, 20, 20),
        (2_061_517, 624_129, 18_913),
    ];

    #[test]
    fn test_binary_gcd() {
        for (u, v, r) in CASES.iter() {
            assert_eq!(binary_gcd(*u, *v), *r);
        }
    }

    #[test]
    fn test_euclidean_gcd() {
        for (u, v, r) in CASES.iter() {
            assert_eq!(euclidean_gcd(*u, *v), *r);
        }
    }
}
