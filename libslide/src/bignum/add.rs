#![allow(clippy::suspicious_arithmetic_impl)]
use crate::bignum::utils::abs;
use crate::bignum::Bignum;
use std::mem;
use std::ops;

impl ops::Add for Bignum {
    type Output = Bignum;

    fn add(self, rhs: Bignum) -> Bignum {
        let mut is_neg: bool = false;
        if self.is_neg && rhs.is_neg {
            is_neg = true;
        } else if self.is_neg && !rhs.is_neg {
            return rhs - abs(&self);
        } else if !self.is_neg && rhs.is_neg {
            return self - abs(&rhs);
        }

        // this assumes the signs are both positive for now
        let mut carry: u8 = 0;
        let mut lhs_size: usize = self.dec.len();
        let mut rhs_size: usize = rhs.dec.len();

        // make lhs smaller vector
        // lhs_size = self, rhs_self = rhs
        let (mut res_dec, lhs_vec) = if lhs_size > rhs_size {
            mem::swap(&mut lhs_size, &mut rhs_size);
            (self.dec, rhs.dec)
        } else {
            (rhs.dec, self.dec)
        };

        // 1. Handle Decimal
        for i in (0..lhs_size).rev() {
            res_dec[i] += lhs_vec[i] + carry;
            carry = (res_dec[i] > 9) as u8;
            res_dec[i] %= 10;
        }

        // 2. Handle Integer

        lhs_size = self.int.len();
        rhs_size = rhs.int.len();

        // make lhs smaller vector
        let (mut res_int, lhs_vec) = if lhs_size > rhs_size {
            mem::swap(&mut lhs_size, &mut rhs_size);
            (self.int, rhs.int)
        } else {
            (rhs.int, self.int)
        };

        // compute addition with both ints
        for i in 0..lhs_size {
            res_int[i] += lhs_vec[i] + carry;
            carry = (res_int[i] > 9) as u8;
            res_int[i] %= 10;
        }

        // fill with rest of the larger integer vector
        // note we need to propogate carry here while in decimal we do not, but if carry = 0 we are
        // done since res_int already contains the values
        let mut i = lhs_size;
        while carry != 0 && i < rhs_size {
            res_int[i] += carry;
            carry = (res_int[i] > 9) as u8;
            res_int[i] %= 10;
            i += 1;
        }

        // add 1 if a carry is leftover
        if carry == 1 {
            res_int.push(1);
        }

        Bignum {
            is_neg,
            int: res_int,
            dec: res_dec,
        }
    }
}

#[cfg(test)]
mod tests {
    macro_rules! bignum_test_add {
        ($($name: ident: $lhs:expr, $rhs:expr, $program:expr)*) => {
        $(
            #[test]
            fn $name() {
                use crate::bignum::Bignum;
                let lhs = Bignum::new($lhs.to_string()).unwrap();
                let rhs = Bignum::new($rhs.to_string()).unwrap();
                let result = $program.to_string();
                assert_eq!((lhs+rhs).to_string(), result);
            }
        )*
        }
    }

    mod add {
        bignum_test_add! {
            int1: "5", "5", "10"
            int2: "55", "13", "68"
            int3: "5", "15", "20"
            int4: "55555555555555555555555", "5555555555555555", "55555561111111111111110"
            int5: "99999999999999999999999999999999" , "999999999999999999999", "100000000000999999999999999999998"
            int6: "111", "1", "112"
            float: "0.1", "0.1", "0.2"
            float2: "0.111", "0.1", "0.211"
            float3: "0.1", "0.111", "0.211"
            float4: "0.5", "0.9", "1.4"
            float5: "0.55555", "0.99", "1.54555"
            float6: "0.555555555555555555555555555", "0.555555555555555555", "1.111111111111111110555555555"
            float7: "0.99999999999999999", "0.999999999999999999999999999999", "1.999999999999999989999999999999"
            float8: "0.1112", "0.923", "1.0342"
            mixed: "1.1", "1.1", "2.2"
            mixed2: "1.9999999", "9999.99999", "10001.9999899"
            mixed3: "99999999999999.999999999999", "99999999999.99999", "100099999999999.999989999999"
            negative_int1: "-5", "5", "0"
            negative_int2: "-10", "5", "-5"
            negative_int3: "5", "-10", "-5"
            negative_int4: "-555555", "999999", "444444"
            negative_int5: "-10", "-10", "-20"
            negative_float1: "-0.1", "0.1", "0"
            negative_float2: "-0.2", "0.1", "-0.1"
            negative_float3: "-0.1", "0.2", "0.1"
            negative_float4: "-0.1", "-0.1", "-0.2"
            negative_mixed: "-12332.55", "1.0", "-12331.55"
            negative_mixed1: "-12332.55", "-1.0", "-12333.55"
            negative_mixed2: "1.0", "-12332.55", "-12331.55"
        }
    }
}
