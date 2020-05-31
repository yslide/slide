#[macro_use]
extern crate criterion;
extern crate libslide;

use criterion::Criterion;
use libslide::{Bignum, _mul};

macro_rules! bench_mul {
    ($($name: ident: $size: expr)*)=> {
        $(
        fn $name(c: &mut Criterion) -> () {
            let mut group = c.benchmark_group("mul");
            group.sample_size(10);
            group.bench_function(concat!("Bignum_", $size, "_mul"), |b| {
                b.iter(|| {
                    let u = String::from_utf8(vec![b'9'; $size]).unwrap();
                    let v = String::from_utf8(vec![b'5'; $size]).unwrap();
                    _mul(Bignum::new(u).unwrap(), Bignum::new(v).unwrap());
                })
            });
            group.finish();
        }
    )*
    }
}

bench_mul! {
    size_1024: 1024
    size_2048: 2048
    size_4096: 4096
}

criterion_group!(bench_mul, size_1024, size_2048, size_4096);
criterion_main!(bench_mul);
