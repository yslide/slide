#[macro_use]
extern crate criterion;
extern crate libslide;

use criterion::Criterion;
use libslide::{Bignum, _binary_skip_mod, _single_skip_mod};

const INPUT: [(&str, &str); 2] = [("92138591", "29135"), ("10201231.1235123", "139581")];

fn bench_binary_modulo(c: &mut Criterion) {
    c.bench_function("binary_modulo", |b| {
        b.iter(|| {
            for (u, v) in INPUT.iter() {
                let lhs = Bignum::new((*u).to_string()).unwrap();
                let rhs = Bignum::new((*v).to_string()).unwrap();
                _binary_skip_mod(lhs, rhs);
            }
        })
    });
}

fn bench_single_modulo(c: &mut Criterion) {
    c.bench_function("single_modulo", |b| {
        b.iter(|| {
            for (u, v) in INPUT.iter() {
                let lhs = Bignum::new((*u).to_string()).unwrap();
                let rhs = Bignum::new((*v).to_string()).unwrap();
                _single_skip_mod(lhs, rhs);
            }
        })
    });
}

criterion_group!(mod_benches, bench_binary_modulo, bench_single_modulo);
criterion_main!(mod_benches);
