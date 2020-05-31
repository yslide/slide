use crate::bignum::Bignum;
use std::cmp::Ordering;

impl Ord for Bignum {
    fn cmp(&self, other: &Bignum) -> Ordering {
        let greater: Ordering;
        let less: Ordering;
        if self.is_neg ^ other.is_neg {
            return other.is_neg.cmp(&self.is_neg);
        } else if other.is_neg && self.is_neg {
            greater = Ordering::Less;
            less = Ordering::Greater;
        } else {
            greater = Ordering::Greater;
            less = Ordering::Less;
        }
        match self.int.len().cmp(&other.int.len()) {
            Ordering::Greater => return greater,
            Ordering::Less => return less,
            _ => (),
        }

        for i in (0..other.int.len()).rev() {
            match self.int[i].cmp(&other.int[i]) {
                Ordering::Greater => return greater,
                Ordering::Less => return less,
                _ => (),
            }
        }

        let smaller_dec_size: usize = if self.dec.len() > other.dec.len() {
            other.dec.len()
        } else {
            self.dec.len()
        };

        for i in 0..smaller_dec_size {
            match self.dec[i].cmp(&other.dec[i]) {
                Ordering::Greater => return greater,
                Ordering::Less => return less,
                _ => (),
            }
        }

        match other.dec.len().cmp(&self.dec.len()) {
            Ordering::Equal => Ordering::Equal,
            Ordering::Less => less,
            Ordering::Greater => greater,
        }
    }
}

impl Eq for Bignum {}

impl PartialEq for Bignum {
    fn eq(&self, other: &Bignum) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

// returns true if the rhs is smaller then lhs
impl PartialOrd for Bignum {
    fn partial_cmp(&self, other: &Bignum) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(feature = "benchmark-internals")]
pub fn _compare(u: Bignum, v: Bignum, s: &str) -> bool {
    match s {
        "eq" => u == v,
        "lte" => u <= v,
        "lt" => u < v,
        "gte" => u >= v,
        "gt" => u > v,
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    macro_rules! bignum_test_cmp {
        ($($name: ident: $stmt:expr)*) => {
        $(
            #[test]
            fn $name() {
                use crate::bignum::Bignum;
                let v: Vec<&str> = $stmt.split(' ').collect();
                let lhs = Bignum::new(v[0].to_string()).unwrap();
                let rhs = Bignum::new(v[2].to_string()).unwrap();
                match v[1] {
                    "=" => assert!(lhs == rhs),
                    "!=" => assert!(lhs != rhs),
                    "<=" => assert!(lhs <= rhs),
                    ">=" => assert!(lhs >= rhs),
                    ">" => assert!(lhs > rhs),
                    "<" => assert!(lhs < rhs),
                    _ => panic!("Test input invalid"),
                }
            }
        )*
        }
    }

    mod cmp {
        bignum_test_cmp! {
            equal1: "1 = 1"
            equal2: "1.1 = 1.1"
            equal3: "-25.25 = -25.25"
            not_equal1: "1 != 2"
            not_equal2: "1.1 != 1.2"
            not_equal3: "2.1 != 1.1"
            not_equal4: "-1.1 != 1.1"
            less_than1: "1 < 2"
            less_than2: "1.1 < 2.1"
            less_than3: "2.1 < 2.2"
            less_than4: "-1 < 1"
            less_than5: "-2 < -1"
            less_than_equal1: "1 < 2"
            less_than_equal2: "1.1 <= 2.1"
            less_than_equal3: "2.1 <= 2.2"
            less_than_equal4: "-1 <= 1"
            less_than_equal5: "-2 <= -1"
            less_than_equal6: "1 <= 1"
            less_than_equal7: "1.1 <= 1.1"
            less_than_equal8: "-25.25 <= -25.25"
            greater_than1: "2 > 1"
            greater_than2: "2.1 > 1.1"
            greater_than3: "2.2 > 2.1"
            greater_than4: "1 > -1"
            greater_than5: "-1 > -2"
            greater_than_equal1: "2 >= 1"
            greater_than_equal2: "2.1 >= 1.1"
            greater_than_equal3: "2.2 >= 2.1"
            greater_than_equal4: "1 >= -1"
            greater_than_equal5: "-1 >= -2"
            greater_than_equal6: "1 >= 1"
            greater_than_equal7: "1.1 >= 1.1"
            greater_than_equal8: "-25.25 >= -25.25"
        }
    }
}
