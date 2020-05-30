#[macro_use]
extern crate criterion;
extern crate libslide;

use criterion::Criterion;
use libslide::{Bignum, _add};

macro_rules! bench_add {
    ($($name: ident: $size: expr)*)=> {
        $(
        fn $name(c: &mut Criterion) -> () {
            c.bench_function(concat!("add_", $size), |b| {
                b.iter(|| {
                    let u = String::from_utf8(vec![b'9'; $size]).unwrap();
                    let v = String::from_utf8(vec![b'5'; $size]).unwrap();
                    _add(Bignum::new(u).unwrap(), Bignum::new(v).unwrap());
                })
            });
        }
    )*
    }
}

bench_add! {
    size_1024: 1024
    size_2048: 2048
    size_4096: 4096
}

criterion_group!(add_benches, size_1024, size_2048, size_4096);
criterion_main!(add_benches);
