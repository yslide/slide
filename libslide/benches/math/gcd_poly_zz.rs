#[macro_use]
extern crate criterion;
extern crate libslide;

use criterion::{black_box, Criterion};
use libslide::_gcd_poly_zz_heu;

const INPUT: [([isize; 3], [isize; 3]); 1] = [([1, 0, -1], [1, -3, 2])];

fn bench_poly_gcd(c: &mut Criterion) {
    c.bench_function("polynomial_gcd", |b| {
        b.iter(|| {
            for (u, v) in INPUT.iter() {
                _gcd_poly_zz_heu(black_box(&u.to_vec()), black_box(&v.to_vec())).unwrap();
            }
        })
    });
}

criterion_group!(gcd_poly_bench, bench_poly_gcd);
criterion_main!(gcd_poly_bench);
