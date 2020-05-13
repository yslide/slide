#![allow(dead_code)]
use crate::bignum::complex::Complex;
use crate::bignum::Bignum;

pub fn recast_vec<T, U>(v: Vec<T>) -> Vec<U> {
    let mut v = std::mem::ManuallyDrop::new(v);
    let p = v.as_mut_ptr();
    let len = v.len();
    let cap = v.capacity();
    unsafe { Vec::from_raw_parts(p as *mut U, len, cap) }
}

pub fn abs(rhs: &Bignum) -> Bignum {
    Bignum {
        is_neg: false,
        dec: rhs.dec.clone(),
        int: rhs.int.clone(),
    }
}

pub fn fft(item: Vec<Complex>, n: usize, ifft: bool) -> Vec<Complex> {
    if n == 1 {
        return item;
    }
    if n % 2 == 1 {
        panic!("Size must be a multiple of 2");
    }
    let w_n: Complex = if ifft {
        Complex::new(0.0, -2.0 * std::f64::consts::PI / n as f64).exp()
    } else {
        Complex::new(0.0, 2.0 * std::f64::consts::PI / n as f64).exp()
    };
    let mut w = Complex::new(1.0, 0.0);
    let mut a_even = item.clone();
    let mut a_odd = item.clone();
    let mut i = 0;
    while i < n {
        a_even[i / 2] = item[i];
        i += 2;
    }
    i = 1;
    while i < n {
        a_odd[i / 2] = item[i];
        i += 2;
    }
    let y_even = fft(a_even, n / 2, ifft);
    let y_odd = fft(a_odd, n / 2, ifft);
    let mut y = item;
    for i in 0..n / 2 {
        y[i] = y_even[i] + w * y_odd[i];
        y[i + n / 2] = y_even[i] - w * y_odd[i];
        w = w * w_n;
    }
    y
}

pub fn ifft(item: Vec<Complex>, n: usize) -> Vec<Complex> {
    let n_complex = Complex::new(n as f64, 0.0);
    let a: Vec<Complex> = fft(item, n, true)
        .into_iter()
        .enumerate()
        .map(|(_i, e)| e / n_complex)
        .collect();
    a
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
                complex_minus_2_twice,
                complex_minus_2,
                complex_2_twice,
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
