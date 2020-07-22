#![allow(unused)]

/// Represents a fraction, consisting of a numerator and denominator.
type Fraction = (/* num */ i64, /* den */ u64);

#[derive(PartialEq, Debug)]
pub struct Dec2FracError {
    num_iter: u64,
    decimal_error: f64,
}

impl std::fmt::Display for Dec2FracError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to find an exact fraction representation after {} iterations.\
            The error at this precision is {}.",
            self.num_iter, self.decimal_error,
        )
    }
}

/// Converts a decimal number to its irreducible fractional representation by walking the
/// [Stern-Brocot tree](https://en.wikipedia.org/wiki/Stern%E2%80%93Brocot_tree). This is
/// equivalent to a binary search of the [Farey sequence](https://en.wikipedia.org/wiki/Farey_sequence).
///
/// ## Algorithm
///
/// Say we are looking for a number A, where A = Whole + Decimal, Whole ∈ W, and Decimal ∈ [0, 1).
/// Then if we can find T and B such that Decimal = T / B,
///      A = Whole + Decimal = (Whole * B) / B + (T / B)
///        = (Whole * B + T) / B
///
/// To find T and B we perform a binary search of irreducible fractions starting with the range [0, 1].
/// On each iteration of the search, we compute the mediant of the range.
///
/// > _mediant_(A / B, C / D) = (A + C) / (B + D)
/// >
/// > The _mediant_ of two fractions is guaranteed to be between those two fractions, though is
/// > **not** always the _median_ of those two fractions.
/// >
/// > In the Stern-Brocot tree, the _mediant_ of two sibling tree nodes is an irreducible fraction.
///      
/// ```text
///                       * ~ T / B
/// 0 / 1 ----------------------|---------------------- 1 / 1
///                           1 / 2 ~ mediant
/// ```
///
/// We then bisect the search to the medaint-bound range the decimal we are interested in.
///
/// ```text
///                                    * ~ T / B
/// 0 / 1 -----------------------------|--------------- 1 / 2
///                                  1 / 3 ~ mediant
/// ```
///
/// We stop when the mediant is equivalent to the decimal searched for.
///
/// ## Failure
///
/// This algorithm is iterative and convergent, but terminates after `max_iter` iterations. If a
/// fractional representation equivalent to the decimal at floating point precision cannot be found
/// after `max_iter`, nothing is returned.
///
/// ## Examples
///
/// ```ignore
/// assert_eq!(dec2frac(0.5, 10), Some((1, 2)))
/// assert_eq!(dec2frac(3.14159265358979323, 10), None)
/// assert_eq!(dec2frac(-3.142857142857142857, 100_000), Some((-22, 7)))
/// ```
pub fn dec2frac(mut num: f64, max_iter: u64) -> Result<Fraction, Dec2FracError> {
    let coeff = if num < 0. { -1 } else { 1 };
    let num = num.abs();

    let whole_part = num.floor() as u64;

    let decimal = num - num.floor();
    let mut lo = (0, 1);
    let mut hi = (1, 1);

    fn mediant(lo: (u64, u64), hi: (u64, u64)) -> (u64, u64) {
        (lo.0 + hi.0, lo.1 + hi.1)
    }

    let frac = {
        || {
            if decimal == 0. {
                return Ok(lo);
            }

            let mut med = mediant(lo, hi);
            let mut med_dec = (med.0 as f64) / (med.1 as f64);
            for _ in 0..max_iter {
                med = mediant(lo, hi);
                med_dec = (med.0 as f64) / (med.1 as f64);
                // lo      mediant       hi
                //            ^-- if == decimal, we're done
                //   ^^^^^^       ^^^^^^^-- otherwise, the decimal is in one of these two ranges.
                //                          update lo/hi to search in the appropriate range.
                if (med_dec - decimal).abs() <= std::f64::EPSILON {
                    return Ok(med);
                } else if decimal < med_dec {
                    hi = med;
                } else {
                    lo = med;
                }
            }

            Err(Dec2FracError {
                num_iter: max_iter,
                decimal_error: (decimal - (med.0 as f64) / (med.1 as f64)).abs(),
            })
        }
    }()?;

    let (numerator, denominator) = frac;
    let combined_numerator = (numerator + whole_part * denominator) as i64;
    Ok((coeff * combined_numerator, denominator))
}

#[cfg(test)]
mod tests {
    use super::*;

    type Dec2FracCase = (f64, Result<(i64, u64), Dec2FracError>);
    #[allow(clippy::excessive_precision)]
    const CASES: [Dec2FracCase; 7] = [
        (0.,  Ok((0 , 1))),
        (1.,  Ok((1 , 1))),
        (0.5, Ok((1 , 2))),
        (
            0.318181818181818181818181818181818181818181818181818181818,
            Ok((7 , 22)),
        ),
        (
            3.142857142857142857142857142857142857142857142857142857142,
            Ok((22 , 7)),
        ),
        (
            3.141592653589793238462643383279502884197169399375105820974944592307816406286208998628034825342117067982148086513282306647093844609550582231725359408128,
            Ok((245_850_922, 78_256_779)),
        ),
        (
            -3.141592653589793238462643383279502884197169399375105820974944592307816406286208998628034825342117067982148086513282306647093844609550582231725359408128,
            Ok((-245_850_922, 78_256_779)),
        ),
    ];

    #[test]
    fn test_dec2frac() {
        for (dec, frac) in CASES.iter() {
            assert_eq!(dec2frac(*dec, 1_000_000), *frac, "{} != {:?}", dec, frac);
        }
    }
}
