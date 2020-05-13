#![allow(clippy::suspicious_arithmetic_impl)]
#![allow(dead_code)]
use std::convert::TryFrom;
use std::fmt;
use std::ops;

static INPUT_ERR_MSG: &str = "Input is not valid";
const TOLERANCE: f64 = 1E-12;

#[derive(Clone, Copy)]
pub struct Complex {
    real: f64,
    imag: f64,
}

impl Complex {
    pub fn new(real: f64, imag: f64) -> Complex {
        Complex { real, imag }
    }

    pub fn real(self) -> f64 {
        self.real
    }

    pub fn imag(self) -> f64 {
        self.imag
    }

    pub fn conjg(self) -> Complex {
        Complex {
            real: self.real,
            imag: -1.0 * self.imag,
        }
    }

    pub fn exp(self) -> Complex {
        // we split exponential into e^real * e^imag and apply euler's identity
        let e_real = self.real.exp();
        let e_imag_real_part = self.imag.cos();
        let e_imag_imag_part = self.imag.sin();
        Complex {
            real: e_real * e_imag_real_part,
            imag: e_real * e_imag_imag_part,
        }
    }
}

impl ops::Add for Complex {
    type Output = Complex;
    fn add(self, rhs: Complex) -> Complex {
        Complex {
            real: self.real + rhs.real,
            imag: self.imag + rhs.imag,
        }
    }
}

impl ops::Sub for Complex {
    type Output = Complex;
    fn sub(self, rhs: Complex) -> Complex {
        Complex {
            real: self.real - rhs.real,
            imag: self.imag - rhs.imag,
        }
    }
}

impl ops::Mul for Complex {
    type Output = Complex;
    fn mul(self, rhs: Complex) -> Complex {
        Complex {
            real: self.real * rhs.real - self.imag * rhs.imag,
            imag: self.real * rhs.imag + self.imag * rhs.real,
        }
    }
}

impl ops::Div for Complex {
    type Output = Complex;
    fn div(self, rhs: Complex) -> Complex {
        let conjg = rhs.conjg();
        let numerator = self * conjg;
        let denominator = rhs * conjg;
        Complex {
            real: numerator.real / denominator.real,
            imag: numerator.imag / denominator.real,
        }
    }
}

impl fmt::Display for Complex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let result: String = if self.imag < 0.0 {
            self.real.to_string() + "-" + &self.imag.abs().to_string() + "i"
        } else {
            self.real.to_string() + "+" + &self.imag.to_string() + "i"
        };
        write!(f, "{}", result)
    }
}

impl TryFrom<String> for Complex {
    type Error = &'static str;
    fn try_from(item: String) -> Result<Self, &'static str> {
        let v: Vec<&str> = item.split(' ').collect();
        let real: f64;
        let is_neg: f64;
        let imag: f64;
        match v[0].parse::<f64>() {
            Ok(val) => real = val,
            Err(_) => return Err(INPUT_ERR_MSG),
        }
        match v[1] {
            "+" => is_neg = 1.0,
            "-" => is_neg = -1.0,
            _ => return Err(INPUT_ERR_MSG),
        }
        let rhs: Vec<&str> = v[2].split('i').collect();
        match rhs[0].parse::<f64>() {
            Ok(val) => imag = val * is_neg,
            Err(_) => return Err(INPUT_ERR_MSG),
        }
        Ok(Complex { real, imag })
    }
}

impl PartialEq for Complex {
    fn eq(&self, other: &Self) -> bool {
        self.real - other.real < TOLERANCE && self.imag - other.imag < TOLERANCE
    }
}

#[cfg(test)]
mod tests {
    macro_rules! complex_test_op {
        ($($name: ident: $lhs: expr, $op: expr, $rhs: expr, $result: expr)*) => {
        $(
            #[test]
            fn $name() {
                use crate::bignum::complex::Complex;
                use std::convert::TryFrom;
                let lhs = Complex::try_from($lhs.to_string()).unwrap();
                let rhs = Complex::try_from($rhs.to_string()).unwrap();
                let result = Complex::try_from($result.to_string()).unwrap();
                match $op {
                    "+" => assert!(lhs+rhs == result),
                    "-" => assert!(lhs-rhs == result),
                    "*" => assert!(lhs*rhs == result),
                    "/" => assert!(lhs/rhs == result),
                    _ => panic!("Test input invalid"),
                }
            }
        )*
        }
    }

    macro_rules! complex_test_conjg {
        ($($name: ident: $lhs: expr, $rhs: expr)*) => {
        $(
            #[test]
            fn $name() {
                use crate::bignum::complex::Complex;
                use std::convert::TryFrom;
                let lhs = Complex::try_from($lhs.to_string()).unwrap();
                let rhs = Complex::try_from($rhs.to_string()).unwrap();
                assert!(lhs.conjg() == rhs);
            }
        )*
        }
    }

    macro_rules! complex_test_exp {
        ($($name: ident: $lhs: expr, $rhs: expr)*) => {
        $(
            #[test]
            fn $name() {
                use crate::bignum::complex::Complex;
                use std::convert::TryFrom;
                let lhs = Complex::try_from($lhs.to_string()).unwrap();
                let rhs = Complex::try_from($rhs.to_string()).unwrap();
                assert!(lhs.exp() == rhs);
            }
        )*
        }
    }

    mod cmplx {
        complex_test_op! {
            // tests have to be in this weird form for now
            // @todo: Fix testing formulation
            add1: "2 + 2i", "+", "2 + 2i", "4 + 4i"
            add2: "2 + 0i", "+", "2 + 0i", "4 + 0i"
            add3: "4.5 + 55i", "+", "2 + 2.2i", "6.5 + 57.2i"
            add4: "2 - 2i", "+", "2 + 2i", "4 + 0i"
            sub1: "2 + 2i", "-", "2 + 2i", "0 + 0i"
            sub2: "2 + 2i", "-", "1 + 1i", "1 + 1i"
            sub3: "2.1 + .1i", "-", "1 + 1i", "1.1 - .9i"
            sub4: "-1.1 + 1i", "-", "-1.1 + 1i", "0 + 0i"
            mul1: "2 + 2i", "*", "2 + 2i", "0 + 8i"
            mul2: "2.2 - 4.1i", "*", "1.2 + 1.2i", "7.56 - 2.28i"
            div1: "-1.1 + 1i", "/", "-1.1 + 1i", "1 + 1i"
            div2: "2.2 - 4.1i", "/", "1.2 + 1.2i", "-0.791 - 2.625i"
        }
    }

    mod conjg {
        complex_test_conjg! {
            conjg1: "2 + 2i", "2 - 2i"
            conjg2: "2 - 2i", "2 + 2i"
            conjg3: "0 + 0i", "0 + 0i"
        }
    }

    mod exp {
        complex_test_exp! {
            exp1: "0 + 0i", "1 + 0i"
            exp2: "1 + 5i", "0.771073764165 - 2.6066264306850i"
            // a fun little excerise
            exp3: "0 + 3.14159265359i", "-1 + 0i"
        }
    }
}
