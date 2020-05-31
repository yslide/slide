#[macro_use]
extern crate criterion;
extern crate libslide;

use criterion::Criterion;
use libslide::{Bignum, _binary_skip_mod, _single_skip_mod};

const CASES: [&str; 2] = ["binary_skip", "single_skip"];

macro_rules! bench_mod {
    ($($name: ident: $size: expr)*) => {
        $(
        fn $name(c: &mut Criterion) -> () {
            let mut group = c.benchmark_group("mod");
            group.sample_size(10);
            for item in CASES.iter() {
                group.bench_function(&(concat!("Bignum_", $size, "_modulo_").to_string() + (*item)), |b| {
                    b.iter(|| {
                        let u = String::from_utf8(vec![b'9'; $size]).unwrap();
                        let v = String::from_utf8(vec![b'5'; $size]).unwrap();
                        match *item {
                            "binary_skip" => _binary_skip_mod(Bignum::new(u.to_string()).unwrap(), Bignum::new(v.to_string()).unwrap()),
                            "single_skip" => _single_skip_mod(Bignum::new(u.to_string()).unwrap(), Bignum::new(v.to_string()).unwrap()),
                            _ => unreachable!(),
                        }
                    })
                });
            }
            group.finish();
        }
    )*
    }
}

bench_mod! {
    size_1024: 1024
    size_2048: 2048
    size_4096: 4096
}

criterion_group!(mod_benches, size_1024, size_2048, size_4096);
criterion_main!(mod_benches);
