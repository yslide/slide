#![allow(dead_code)]
use crate::bignum::complex::Complex;
use crate::bignum::Bignum;
use std::cmp::Ordering;
use std::f64::consts::PI;

const TOLERANCE: f64 = 1E-12;
static ERR_MSG: &str = "Try From Method failed to cast";

pub fn recast_vec<T, U>(v: Vec<T>) -> Vec<U> {
    let mut v = std::mem::ManuallyDrop::new(v);
    let p = v.as_mut_ptr();
    let len = v.len();
    let cap = v.capacity();
    unsafe { Vec::from_raw_parts(p as *mut U, len, cap) }
}

// note we cannot use the above recast since we have a user defined type with
// an undefined way of storing its memory
// prereqs: tryfrom(or from) is implemented for type coercion and default and clone are implemented
pub fn recast_user_vec<T, U>(v: Vec<T>) -> Result<Vec<U>, &'static str>
where
    T: std::clone::Clone,
    U: std::default::Default + std::clone::Clone + std::convert::TryFrom<T>,
{
    let mut res: Vec<U> = vec![U::default(); v.len()];
    for i in 0..v.len() {
        let val = match U::try_from(v[i].clone()) {
            Ok(val) => val,
            Err(_err) => return Err(ERR_MSG),
        };
        res[i] = val;
    }
    Ok(res)
}

pub fn truncate_zeros(mut v: Vec<u8>) -> Vec<u8> {
    let mut count = 0;
    for i in (0..v.len()).rev() {
        if v[i] == 0 {
            count += 1;
        } else {
            break;
        }
    }
    v.truncate(v.len() - count);
    v
}

pub fn normalize_vecs(v1: Vec<u8>, v2: Vec<u8>) -> (Vec<u8>, Vec<u8>) {
    let mut res: Vec<u8>;
    match v1.len().cmp(&v2.len()) {
        Ordering::Greater => {
            res = vec![0; v1.len()];
            res[..v2.len()].clone_from_slice(&v2[..v2.len()]);
            (v1, res)
        }
        Ordering::Less => {
            res = vec![0; v2.len()];
            res[..v1.len()].clone_from_slice(&v1[..v1.len()]);
            (res, v2)
        }
        Ordering::Equal => (v1, v2),
    }
}

pub fn abs(rhs: &Bignum) -> Bignum {
    Bignum {
        is_neg: false,
        dec: rhs.dec.clone(),
        int: rhs.int.clone(),
    }
}

pub fn ispowerof2(n: usize) -> bool {
    n != 0 && (n & (n - 1) == 0)
}

pub fn fft(item: Vec<Complex>, n: usize, ifft: bool) -> Vec<Complex> {
    if n == 1 {
        return item;
    }
    if !ispowerof2(n) {
        panic!("Size must be a power of 2");
    }
    let mut a = item.clone();
    let mut a_even = item.clone();
    let mut a_odd = item.clone();
    for i in 0..n / 2 {
        a_even[i] = item[i * 2];
        a_odd[i] = item[i * 2 + 1];
    }
    a_even = fft(a_even, n / 2, ifft);
    a_odd = fft(a_odd, n / 2, ifft);

    let mut t: Complex;
    for k in 0..n / 2 {
        let coeff: f64 = 4.0 * (ifft as usize) as f64 - 2.0;
        t = Complex::new(0.0, coeff * PI * k as f64 / n as f64).exp() * a_odd[k];
        a[k] = a_even[k] + t;
        a[n / 2 + k] = a_even[k] - t;
    }
    a
}

#[cfg(feature = "benchmark-internals")]
pub fn _fft(v: Vec<u8>) -> Vec<Complex> {
    let vlen = v.len();
    fft(recast_user_vec(v).unwrap(), vlen, false)
}

pub fn ifft(item: Vec<Complex>, n: usize) -> Vec<Complex> {
    let n_complex = Complex::new(n as f64, 0.0);
    let a: Vec<Complex> = fft(item, n, true)
        .into_iter()
        .map(|e| e / n_complex)
        .collect();
    a
}

// converts a multi digit polynomial to an single digit vector
pub fn convert_poly(v: Vec<u16>) -> Vec<u8> {
    let mut carry: u16 = 0;
    let mut a: Vec<u16> = v
        .into_iter()
        .map(|mut e| {
            e += carry;
            carry = e / 10;
            e % 10
        })
        .collect();
    if carry > 0 {
        a.insert(0, carry);
    }
    recast_user_vec(a).unwrap()
}

#[cfg(test)]
mod tests {
    mod fft {
        // just note wolfram uses a 1/sqrt(n) scalling factor that we don't if results are compared
        use crate::bignum::complex::Complex;
        fn create_vec_1() -> Vec<Complex> {
            let complex_1 = Complex::new(1.0, 0.0);
            let complex_2 = Complex::new(2.0, 0.0);
            let complex_3 = Complex::new(3.0, 0.0);
            let complex_4 = Complex::new(4.0, 0.0);
            vec![complex_1, complex_2, complex_3, complex_4]
        }

        fn create_vec_2() -> Vec<Complex> {
            let complex_10 = Complex::new(10.0, 0.0);
            let complex_minus_2_twice = Complex::new(-2.0, -2.0);
            let complex_minus_2 = Complex::new(-2.0, 0.0);
            let complex_2_twice = Complex::new(-2.0, 2.0);
            vec![
                complex_10,
                complex_2_twice,
                complex_minus_2,
                complex_minus_2_twice,
            ]
        }

        #[test]
        fn test_fft() {
            use crate::bignum::utils::fft;
            let vec = create_vec_1();
            let n = vec.len();
            let res = fft(vec, n, false);
            assert!(res == create_vec_2());
        }

        #[test]
        fn test_ifft() {
            use crate::bignum::utils::ifft;
            let vec = create_vec_2();
            let n = vec.len();
            let res = ifft(vec, n);
            assert!(res == create_vec_1());
        }
    }
}
