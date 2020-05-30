#[macro_use]
extern crate criterion;
extern crate libslide;
extern crate lazy_static;

use criterion::Criterion;
use libslide::{Bignum, _sub};
use lazy_static::lazy_static;

lazy_static! {
    static ref INPUT: [(Bignum, Bignum); 3] = 
        [(Bignum::new("9000000000".to_string()).unwrap(),
            Bignum::new("101000000000000000000".to_string()).unwrap()),
        (Bignum::new("0.999999999999999999999900000000000000".to_string()).unwrap(),
            Bignum::new("0.55555539999999999999".to_string()).unwrap()), 
        (Bignum::new("99999999999999.999999999999".to_string()).unwrap(), 
            Bignum::new("100000000000000000000000000".to_string()).unwrap())];
}

fn bench_sub(c: &mut Criterion) {
    c.bench_function("sub",|b| {
        b.iter(|| {
            for (u, v) in INPUT.iter() {
                _sub(u.to_owned(), v.to_owned());
            }
        })
    });
}

criterion_group!(sub_benches, bench_sub);
criterion_main!(sub_benches);
