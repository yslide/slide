#![allow(clippy::suspicious_arithmetic_impl)]
use crate::bignum::Bignum;
use std::ops;

// these functions are used for benchmarks, we currently use binary_skip_mod
fn binary_skip_mod(lhs: Bignum, rhs: Bignum) -> Bignum {
    // we only support positive mod for now
    if !rhs.dec.is_empty() {
        panic!("The module fuction only supports whole numbers!");
    }
    if rhs.int.is_empty() {
        panic!("Mod 0 is undefined");
    }
    let mut result = lhs;
    let zero = Bignum::new("0".to_string()).unwrap();
    let mut coeff = Bignum::new("1".to_string()).unwrap();
    let multiplier = Bignum::new("2".to_string()).unwrap();
    while result > zero {
        result = result - rhs.clone() * coeff.clone();
        coeff = coeff.clone() * multiplier.clone();
    }
    // an empty vec can be created which is equal to zero
    if result.int.is_empty() && result.dec.is_empty() {
        return result;
    }
    while result < zero {
        result = result + rhs.clone()
    }
    result
}

#[cfg(feature = "benchmark-internals")]
pub fn _binary_skip_mod(u: Bignum, v: Bignum) -> Bignum {
    binary_skip_mod(u, v)
}

fn single_skip_mod(lhs: Bignum, rhs: Bignum) -> Bignum {
    // we only support positive mod for now
    if !rhs.dec.is_empty() {
        panic!("The modulo fuction only supports whole numbers!");
    }
    if rhs.int.is_empty() {
        panic!("Mod 0 is undefined");
    }
    let mut result = lhs;
    let zero = Bignum::new("0".to_string()).unwrap();
    while result > zero {
        result = result - rhs.clone();
    }
    // an empty vec can be created which is equal to zero
    if result.int.is_empty() && result.dec.is_empty() {
        return result;
    }
    while result < zero {
        result = result + rhs.clone()
    }
    result
}
#[cfg(feature = "benchmark-internals")]
pub fn _single_skip_mod(u: Bignum, v: Bignum) -> Bignum {
    single_skip_mod(u, v)
}

impl ops::Rem for Bignum {
    type Output = Bignum;

    fn rem(self, rhs: Bignum) -> Bignum {
        binary_skip_mod(self, rhs)
    }
}

#[cfg(test)]
mod tests {
    macro_rules! bignum_test_mod {
        ($($name: ident: $lhs:expr, $rhs: expr, $program:expr)*) => {
        $(
            #[test]
            fn $name() {
                use crate::bignum::Bignum;
                let lhs = Bignum::new($lhs.to_string()).unwrap();
                let rhs = Bignum::new($rhs.to_string()).unwrap();
                let result = $program.to_string();
                assert_eq!((lhs%rhs).to_string(), result);
            }
        )*
        }
    }

    macro_rules! bignum_test_mod_should_panic {
        ($($name: ident: $lhs:expr, $rhs: expr)*) => {
        $(
            #[test]
            #[should_panic]
            #[allow(unused_must_use)]
            fn $name() {
                use crate::bignum::Bignum;
                let lhs = Bignum::new($lhs.to_string()).unwrap();
                let rhs = Bignum::new($rhs.to_string()).unwrap();

                lhs%rhs;
            }
        )*
        }
    }

    macro_rules! bignum_test_bench_functions {
        ($($name: ident: $lhs:expr, $rhs: expr, $program:expr, $mode: expr)*) => {
        $(
            #[test]
            fn $name() {
                use crate::bignum::Bignum;
                use crate::bignum::modulo::single_skip_mod;
                use crate::bignum::modulo::binary_skip_mod;
                let lhs = Bignum::new($lhs.to_string()).unwrap();
                let rhs = Bignum::new($rhs.to_string()).unwrap();
                let result = $program.to_string();
                match $mode {
                    "1" => assert_eq!(binary_skip_mod(lhs, rhs).to_string(), result),
                    "2" => assert_eq!(single_skip_mod(lhs, rhs).to_string(), result),
                    _ => panic!("Please provide valid testing options"),
                }
            }
        )*
        }
    }

    mod modulo {
        bignum_test_mod! {
            int1: "5", "3", "2"
            int2: "5", "7", "5"
            int3: "92138591", "29135", "13721"
            float1: "10201231.1235123", "139581", "11818.1235123"
            int4: "1000.00000", "1001", "1000"
            int5: "000010000.00000", "100000", "10000"
        }
        bignum_test_mod_should_panic! {
            float2: "10", "10.1"
            zero1: "10", "0.0"
        }
        bignum_test_bench_functions! {
            bin_int1: "5", "3", "2", "1"
            bin_int2: "5", "7", "5", "1"
            bin_int3: "92138591", "29135", "13721", "1"
            bin_float1: "10201231.1235123", "139581", "11818.1235123", "1"
            bin_zero1: "5", "5", "0", "1"
            sin_int1: "5", "3", "2", "2"
            sin_int2: "5", "7", "5", "2"
            sin_int3: "92138591", "29135", "13721", "2"
            sin_float1: "10201231.1235123", "139581", "11818.1235123", "2"
            sin_zero1: "5", "5", "0", "2"
        }
    }
}
