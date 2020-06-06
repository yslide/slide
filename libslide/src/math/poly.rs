#![allow(unused)] // TODO(unused)
#![allow(clippy::should_implement_trait)]

use crate::math::gcd;
use core::cmp::{max, min};
use core::convert::TryInto;

#[derive(Default, Clone, Eq, PartialEq, Debug)]
pub struct Poly {
    pub vec: Vec<isize>,
}

impl From<Vec<isize>> for Poly {
    fn from(v: Vec<isize>) -> Poly {
        Self::new(v)
    }
}

impl From<&Vec<isize>> for Poly {
    fn from(v: &Vec<isize>) -> Poly {
        Self::new(v.clone())
    }
}

/// Creates a new polynomial.
///
/// # Examples:
///
/// ```ignore
/// poly![1, 2, -4]; // x^2 + 2x - 4
/// poly![]; // empty polynomial
/// ```
#[macro_export]
macro_rules! poly {
    ($($x:expr),+ $(,)?) => (
        Poly::new(vec![$($x),+])
    );

    () => {
        Poly::empty()
    };
}

impl Poly {
    pub fn new(vec: Vec<isize>) -> Self {
        Self { vec }
    }

    pub fn empty() -> Self {
        Self::new(vec![])
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty() || self.vec.iter().all(|&n| n == 0)
    }

    /// Gets the degree of the polynomial.
    #[inline]
    pub fn deg(&self) -> isize {
        self.vec.len() as isize - 1
    }

    /// Returns the [primitive polynomial] of `self` over the integers.
    ///
    /// # Examples:
    ///
    /// ```ignore
    /// // 6x^2 + 4x + 2 -> 3x^2 + 2x + 1
    /// assert_eq!(poly![6, 4, 2], poly![3, 2, 1]);
    /// ```
    ///
    /// [primitive polynomial]: https://en.wikipedia.org/wiki/Primitive_part_and_content
    pub fn primitive(self) -> Self {
        if self.is_empty() {
            return self;
        }
        let largest_gcd = self
            .vec
            .iter()
            .fold(self.vec[0].abs() as usize, |largest_gcd, item| {
                gcd(item.abs() as usize, largest_gcd)
            }) as isize;

        if largest_gcd == 1 {
            self
        } else {
            Poly::new(self.vec.iter().map(|e| e / largest_gcd).collect())
        }
    }

    /// Adds a term of form `coeff`x^`pow` to `self`.
    ///
    /// # Examples:
    ///
    /// ```ignore
    /// // (x + 2) + 3x^2 -> 3x^2 + x + 2
    /// assert_eq!(poly![1, 2].add_term(3, 2), poly![3, 1, 2]);
    /// ```
    fn add_term(mut self, coeff: isize, pow: isize) -> Self {
        if coeff == 0 {
            return self;
        }

        let deg = self.deg();
        if pow == deg {
            // (x^2 + 2x + 3) + x^2
            self.vec[0] += coeff;
        } else if pow >= deg {
            // (x^2 + 2x + 3) + x^4
            // [1, 2, 3] -> [3, 2, 1] -> [3, 2, 1, 0] :: [1] -> [3, 2, 1, 0, 4] -> [1, 0, 1, 2, 3]
            let extra_needed = pow - self.deg() - 1;
            let extended = self
                .vec
                .into_iter()
                .rev()
                .chain(vec![0; extra_needed as usize].into_iter())
                .chain(vec![coeff].into_iter());
            self.vec = extended.rev().collect();
        } else {
            // (x^2 + 2x + 3) + 2x
            self.vec[(deg - pow) as usize] += coeff;
        }
        self
    }

    /// Multiplies a term of form `coeff`x^`pow` to `self`.
    ///
    /// # Examples:
    ///
    /// ```ignore
    /// // (x + 2) * 3x^2 -> 3x^3 + 6x^2
    /// assert_eq!(poly![1, 2].mul_term(3, 2), poly![3, 6, 0, 0]);
    /// ```
    fn mul_term(mut self, coeff: isize, pow: isize) -> Self {
        if coeff == 0 || self.is_empty() {
            return Poly::empty();
        }
        for term in self.vec.iter_mut() {
            *term *= coeff;
        }
        for _ in 0..pow {
            self.vec.push(0);
        }
        self
    }

    /// Negates each term of the polynomial.
    #[inline]
    fn negate(mut self) -> Self {
        self.mul_scalar(-1)
    }

    /// Multiplies each term in the polynomial by a scalar.
    #[inline]
    pub fn mul_scalar(mut self, c: isize) -> Self {
        for term in self.vec.iter_mut() {
            *term *= c;
        }
        self
    }

    /// Divides each term in the polynomial by a scalar.
    /// If the scalar divisor is 0, an error is returned.
    pub fn div_scalar(self, c: isize) -> Result<Self, &'static str> {
        if c == 0 {
            Err("Cannot divide a polynomial by 0")
        } else {
            Ok(self.mul_scalar(1 / c))
        }
    }

    /// Subtracts `other` from `self`, yielding a new polynomial.
    ///
    /// # Examples:
    ///
    /// ```ignore
    /// // (x + 2) - (3x^2 + 2x) -> -3x^2 - x + 2
    /// assert_eq!(poly![1, 2].sub(poly![3, 2, 0]), poly![-3, -1, 2]);
    /// ```
    fn sub(mut self, mut other: Self) -> Self {
        if other.vec.is_empty() {
            return other.negate();
        }
        if other.vec.is_empty() {
            return self;
        }
        let d_self = self.deg();
        let d_other = other.deg();
        if d_self == d_other {
            for i in (0..self.vec.len()).rev() {
                self.vec[i] -= other.vec[i];
            }
            self.truncate_zeros()
        } else {
            let mut lhs = self.vec;
            let mut rhs = other.vec;
            let extra_terms = (d_self as isize - d_other as isize).abs() as usize;
            let mut new_poly: Poly;
            if d_self > d_other {
                // (x^2 + x + 1) - (x + 1)
                new_poly = Poly::new(lhs[..extra_terms].to_vec()); // push extra terms not in `other`
                lhs = lhs[extra_terms..].to_vec();
            } else {
                // (x + 1) - (x^2 + x + 1)
                new_poly = Poly::new(rhs[..extra_terms].to_vec()).negate(); // push extra terms not in `self`
                rhs = rhs[extra_terms..].to_vec();
            }
            for i in 0..min(lhs.len(), rhs.len()) {
                lhs[i] -= rhs[i];
            }
            new_poly.vec.append(&mut lhs);

            new_poly.truncate_zeros()
        }
    }

    /// Removes leading zero terms in a polynomial.
    #[inline]
    fn truncate_zeros(mut self) -> Self {
        let mut count = 0;
        for i in &self.vec {
            if *i == 0 {
                count += 1;
            } else {
                break;
            }
        }
        self.vec = self.vec.into_iter().skip(count).collect();
        self
    }

    /// Divides one polynomial by another, returning a tuple of (quotient, remainder) or an error
    /// if division failed.
    ///
    /// # Examples:
    ///
    /// ```ignore
    /// // (x^2 - 4) / (x + 2) -> ((x - 2), 0)
    /// assert_eq!(poly![1, 0, -4].div(poly![1, 2]), Ok((poly![1, -2], poly![])));
    ///
    /// // (x^2 - 2x) / (x + 1) -> ((x - 3), 3)
    /// assert_eq!(poly![1, 0, -2].div(poly![1, 1]), Ok((poly![1, -3], poly![3])));
    /// ```
    pub fn div(self, other: Poly) -> Result<(Self, Self), &'static str> {
        let d_self = self.deg();
        let d_other = other.deg();
        if other.vec.is_empty() {
            return Err("Cannot divide by a 0 polynomial");
        } else if d_self < d_other {
            return Ok((poly![], Poly::new(self.vec)));
        }
        let lc_other = other.lc();
        let mut rem_poly = self;
        let mut d_rem = d_self;
        let mut quo = poly![];
        let mut heu: Poly;
        loop {
            let lc_rem = rem_poly.lc();
            // Currently, this only supports integer division.
            // TODO: Expand this to handle fractions.
            if lc_rem % lc_other != 0 {
                // The current iteration won't divide evenly, so we're done.
                // TODO: above
                break;
            }
            let cur_term_coeff = lc_rem / lc_other;
            quo = quo.add_term(cur_term_coeff, (d_rem - d_other));

            rem_poly = rem_poly.sub(
                // Subtract (current term coeff * rhs) from the rest of polynomial we need to
                // divide.
                other.clone().mul_term(cur_term_coeff, d_rem - d_other),
            );

            let d_rem_old = d_rem;
            d_rem = rem_poly.deg();
            if d_rem < d_other {
                break;
            } else if d_rem >= d_rem_old {
                return Err("Unexpected state: remainder degreee not lower after division");
            }
        }
        Ok((quo, rem_poly))
    }

    /// Returns the max norm of a polynomial.
    /// This is equivalent to the largest absolute value of each term's coefficient.
    pub fn max_norm(&self) -> usize {
        let mut max_n = 0;
        for i in &self.vec {
            max_n = max(max_n, i.abs() as usize);
        }
        max_n
    }

    /// Returns the leading coefficient, i.e. the coefficient of the highest-degree term, of the
    /// polynomial.
    /// If the polynomial is empty, the leading coefficient is 0.
    #[inline]
    pub fn lc(&self) -> isize {
        *self.vec.get(0).unwrap_or(&0)
    }

    /// Evaluates the polynomial at a value `x`.
    ///
    /// # Examples:
    ///
    /// ```ignore
    /// // (x^2 - 4)(1) -> -3
    /// assert_eq!(poly![1, 0, -4].eval(1), -3);
    /// ```
    #[inline]
    pub fn eval(&self, x: isize) -> isize {
        self.vec.iter().fold(0, |mut res, &n| {
            res *= x;
            res + n
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add_1() {
        assert_eq!(poly![2, -1].add_term(2, 4), poly![2, 0, 0, 2, -1]);
    }

    #[test]
    fn add_2() {
        assert_eq!(poly![2, -1].add_term(2, 1), poly![4, -1]);
    }

    #[test]
    fn add_3() {
        assert_eq!(poly![3, 0, 5].add_term(2, 1), poly![3, 2, 5]);
    }

    #[test]
    fn mul_1() {
        assert_eq!(poly![3, 0, 5].mul_term(0, 2), poly![]);
    }

    #[test]
    fn mul_2() {
        assert_eq!(poly![3, 0, 5].mul_term(2, 2), poly![6, 0, 10, 0, 0]);
    }

    #[test]
    fn sub_1() {
        assert_eq!(poly![3, 0, 5].sub(poly![1, 0, 1]), poly![2, 0, 4]);
    }

    #[test]
    fn sub_2() {
        assert_eq!(poly![1, 0, -1].sub(poly![1, -2]), poly![1, -1, 1]);
    }

    #[test]
    fn sub_3() {
        assert_eq!(poly![1, 0, -1].sub(poly![1, -1, 0]), poly![1, -1]);
    }

    #[test]
    fn sub_4() {
        assert_eq!(poly![1, 0, -1].sub(poly![2, 3]), poly![1, -2, -4]);
    }

    #[test]
    fn sub_5() {
        assert_eq!(poly![2, 3].sub(poly![1, 0, -1]), poly![-1, 2, 4]);
    }

    #[test]
    fn div_1() {
        assert_eq!(
            poly![1, 0, -1].div(poly![2, -4]).unwrap(),
            (poly![], poly![1, 0, -1])
        );
    }

    #[test]
    fn div_2() {
        assert_eq!(
            poly![1, 0, -1].div(poly![1, -1]).unwrap(),
            (poly![1, 1], poly![])
        )
    }
}
