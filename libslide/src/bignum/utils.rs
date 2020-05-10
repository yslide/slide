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
