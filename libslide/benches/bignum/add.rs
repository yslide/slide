#[macro_use]
extern crate criterion;
extern crate libslide;
extern crate lazy_static;

use criterion::Criterion;
use libslide::{Bignum, _add};
use lazy_static::lazy_static;

lazy_static! {
    static ref INPUT: [(Bignum, Bignum); 4] = 
        [(Bignum::new("99999999999999999999999999999999".to_string()).unwrap(), 
            Bignum::new("999999999999999999999".to_string()).unwrap()), 
        (Bignum::new("0.555555555555555555555555555".to_string()).unwrap(), 
            Bignum::new("0.555555555555555555".to_string()).unwrap()),
        (Bignum::new("99999999999999.999999999999".to_string()).unwrap(),
            Bignum::new("99999999999.99999".to_string()).unwrap()), 
        (Bignum::new("-99999999999999.999999999999".to_string()).unwrap(),
            Bignum::new("-99999999999.99999".to_string()).unwrap())];
}

fn bench_add(c: &mut Criterion) {
    c.bench_function("add", |b| {
        b.iter(|| {
            for (u, v) in INPUT.iter() {
                _add(u.to_owned(), v.to_owned());
            }
        })
    });
}

criterion_group!(add_benches, bench_add);
criterion_main!(add_benches);
