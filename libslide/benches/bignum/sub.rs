#[macro_use]
extern crate criterion;
extern crate libslide;

use criterion::Criterion;
use libslide::{Bignum, _sub};

macro_rules! bench_sub {
    ($($name: ident: $size: expr)*)=> {
        $(
        fn $name(c: &mut Criterion) -> () {
            c.bench_function(concat!("Bignum_", $size, "_sub"), |b| {
                b.iter(|| {
                    let v = String::from_utf8(vec![b'9'; $size]).unwrap();
                    let u = String::from_utf8(vec![b'1'; $size]).unwrap();
                    _sub(Bignum::new(u).unwrap(), Bignum::new(v).unwrap());
                })
            });
        }
    )*
    }
}

bench_sub! {
    size_1024: 1024
    size_2048: 2048
    size_4096: 4096
}

criterion_group!(sub_benches, size_1024, size_2048, size_4096);
criterion_main!(sub_benches);
