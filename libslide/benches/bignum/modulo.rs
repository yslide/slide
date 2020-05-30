#[macro_use]
extern crate criterion;
extern crate libslide;
extern crate lazy_static;

use criterion::Criterion;
use libslide::{Bignum, _binary_skip_mod, _single_skip_mod};
use lazy_static::lazy_static;

lazy_static! {
    static ref INPUT: [(Bignum, Bignum); 2] = 
        [(Bignum::new("92138591".to_string()).unwrap() , 
            Bignum::new("29135".to_string()).unwrap()),
         (Bignum::new("10201231.1235123".to_string()).unwrap(),
            Bignum::new("139581".to_string()).unwrap())];
}

fn bench_binary_modulo(c: &mut Criterion) {
    c.bench_function("binary_modulo", |b| {
        b.iter(|| {
            for (u, v) in INPUT.iter() {
                _binary_skip_mod(u.to_owned(), v.to_owned());
            }
        })
    });
}

fn bench_single_modulo(c: &mut Criterion) {
    c.bench_function("single_modulo", |b| {
        b.iter(|| {
            for (u, v) in INPUT.iter() {
                _single_skip_mod(u.to_owned(), v.to_owned());
            }
        })
    });
}

criterion_group!(mod_benches, bench_binary_modulo, bench_single_modulo);
criterion_main!(mod_benches);
