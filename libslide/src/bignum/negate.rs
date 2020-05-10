use crate::bignum::Bignum;
use std::ops;

impl ops::Neg for Bignum {
    type Output = Bignum;

    fn neg(self) -> Bignum {
        Bignum {
            is_neg: !self.is_neg,
            dec: self.dec,
            int: self.int,
        }
    }
}
