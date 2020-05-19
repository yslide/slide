#[macro_use]
extern crate criterion;
extern crate libslide;

use criterion::{black_box, Criterion};
use libslide::{_binary_gcd, _euclidean_gcd};

const INPUT: [(usize, usize); 3] = [
    (288_481, 22_783),
    (939_841_321, 28_847_717),
    (48_812, 284_829),
];

fn bench_binary_gcd(c: &mut Criterion) {
    c.bench_function("binary_gcd", |b| {
        b.iter(|| {
            for (u, v) in INPUT.iter() {
                _binary_gcd(black_box(*u), black_box(*v));
            }
        })
    });
}

fn bench_euclidean_gcd(c: &mut Criterion) {
    c.bench_function("euclidean_gcd", |b| {
        b.iter(|| {
            for (u, v) in INPUT.iter() {
                _euclidean_gcd(black_box(*u), black_box(*v));
            }
        })
    });
}

criterion_group!(gcd_benches, bench_binary_gcd, bench_euclidean_gcd);
criterion_main!(gcd_benches);
