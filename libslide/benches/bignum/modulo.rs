#[macro_use]
extern crate criterion;
extern crate libslide;

use criterion::Criterion;
use libslide::{Bignum, _binary_skip_mod, _single_skip_mod};

macro_rules! bench_mod {
    ($($name: ident: $size: expr, $mode: expr)*) => {
        $(
        fn $name(c: &mut Criterion) -> () {
            match $mode {
                1 => {
                    c.bench_function(concat!("binary_skip_modulo_", $size), |b| {
                        b.iter(|| {
                            let u = String::from_utf8(vec![b'9'; $size]).unwrap();
                            let v = String::from_utf8(vec![b'2'; $size]).unwrap();
                            _binary_skip_mod(Bignum::new(u).unwrap(), Bignum::new(v).unwrap());
                        })
                    });
                    
                }, 
                2 => {
                    c.bench_function(concat!("single_skip_modulo_", $size), |b| {
                        b.iter(|| {
                            let u = String::from_utf8(vec![b'9';$size]).unwrap();
                            let v = String::from_utf8(vec![b'2';$size/10]).unwrap();
                            _single_skip_mod(Bignum::new(u).unwrap(), Bignum::new(v).unwrap());
                        })
                    });
                }, 
                _ => unreachable!(),
            }
        }
    )*
    }
}

bench_mod! {
    size_1024_bin: 1024, 1
    size_2048_bin: 2048, 1
    size_4096_bin: 4096, 1
    size_1024_sin: 1024, 2
    size_2048_sin: 2048, 2
    size_4096_sin: 4096, 2
}

criterion_group!(mod_benches, size_1024_bin, size_2048_bin, size_4096_bin, size_1024_sin, size_2048_sin, size_4096_sin);
criterion_main!(mod_benches);
