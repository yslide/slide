#[macro_use]
extern crate criterion;
extern crate libslide;

use criterion::Criterion;
use libslide::{Bignum, _compare};

const CASES: [&str; 5] = ["eq", "lte", "lt", "gte", "gt"];

macro_rules! bench_bignum_cmp {
    ($($name: ident: $size: expr)*)=> {
        $(
        fn $name(c: &mut Criterion) -> () {
            for item in CASES.iter() {
                c.bench_function(&(concat!("Bignum_", $size, "_cmp_").to_string() + (*item)), |b| {
                    b.iter(|| {
                        let u = String::from_utf8(vec![b'9'; $size]).unwrap();
                        let v = String::from_utf8(vec![b'5'; $size]).unwrap();
                        _compare(Bignum::new(u.to_string()).unwrap(), Bignum::new(v.to_string()).unwrap(), item);
                   })
                });
            }
        }
    )*
    }
}

bench_bignum_cmp! {
    size_1024: 1024
    size_2048: 2048
    size_4096: 4096
}

criterion_group!(bignum_cmp_benches, size_1024, size_2048, size_4096);
criterion_main!(bignum_cmp_benches);
