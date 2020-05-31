#[macro_use]
extern crate criterion;
extern crate libslide;

use criterion::Criterion;
use libslide::Bignum;

macro_rules! bench_bignum_constructor {
    ($($name: ident: $size: expr)*)=> {
        $(
        fn $name(c: &mut Criterion) -> () {
            c.bench_function(concat!("Bignum_", $size, "_constructor"), |b| {
                b.iter(|| {
                    let u = String::from_utf8(vec![b'9'; $size]).unwrap();
                    Bignum::new(u.to_string()).unwrap();
                })
            });
        }
    )*
    }
}

bench_bignum_constructor! {
    size_1024: 1024
    size_2048: 2048
    size_4096: 4096
}

criterion_group!(ctor_benches, size_1024, size_2048, size_4096);
criterion_main!(ctor_benches);
