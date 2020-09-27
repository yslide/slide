//! Utilities for dealing with [rug][::rug] arithmetic.

/// Attempts to parse a [Float] precision from a string, returning an error if the precision could
/// not be parsed or does not fit in a [Float]'s precision bounds.
///
/// [Float]: [rug::Float]
pub fn validate_precision(prec: String) -> Result<(), String> {
    match prec.parse::<u32>() {
        Ok(prec) => {
            let min = rug::float::prec_min();
            let max = rug::float::prec_max();
            if prec >= min && prec <= max {
                Ok(())
            } else {
                Err(format!(
                    "Specified precision {} does not fit in precision range [{}, {}]",
                    prec, min, max
                ))
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Formats a [Rational][rug::Rational] in a user-pretty format.
pub fn fmt_rational(rational: &rug::Rational, prec: u32) -> String {
    if rational.denom() == &1 {
        return rational.numer().to_string();
    }

    let float = rug::Float::with_val(prec, rational);

    // If the Float's margin of error is less than that of a 64-bit FP epsilon, assume the error is
    // superfluous and return a "pretty" formatting as if the Float was an f64.
    let diff = (float.clone() - rug::Float::with_val(float.prec(), float.to_f64())).abs();
    if diff <= std::f64::EPSILON {
        return float.to_f64().to_string();
    }

    let (is_neg, raw_int, exp) = float.to_sign_string_exp(10, None);
    let neg = if is_neg { "-" } else { "" };
    let num = match exp {
        Some(exp) => {
            if exp <= 0 {
                // 62, e = -2 -> 0.0062
                let mut num =
                    String::with_capacity(neg.len() + /* 0. */ 2 + (-exp) as usize + raw_int.len());

                num.push_str(neg);
                num.push_str("0.");
                for _ in 0..(-exp) {
                    num.push('0');
                }
                num.push_str(&raw_int);
                num
            } else if exp >= raw_int.len() as i32 {
                // 62, e = 3 -> 620
                let extra_zeros = (exp - raw_int.len() as i32) as usize;
                let mut num = String::with_capacity(neg.len() + raw_int.len() + extra_zeros);

                num.push_str(neg);
                num.push_str(&raw_int);
                for _ in 0..extra_zeros {
                    num.push('0');
                }
                num
            } else {
                // 62, e = 1 -> 6.2
                let mut num = String::with_capacity(neg.len() + raw_int.len() + /* . */ 1);

                num.push_str(neg);
                for (i, ch) in raw_int.chars().enumerate() {
                    if i == exp as usize {
                        num.push('.');
                    }
                    num.push(ch)
                }
                num
            }
        }
        None => {
            let mut num = String::with_capacity(neg.len() + raw_int.len());

            num.push_str(neg);
            num.push_str(&raw_int);
            num
        }
    };

    if num.contains('.') {
        num.trim_end_matches('0').trim_end_matches('.').to_owned()
    } else {
        num
    }
}
