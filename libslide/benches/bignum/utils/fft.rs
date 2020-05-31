#[macro_use]
extern crate criterion;
extern crate libslide;

use criterion::Criterion;
use libslide::_fft;

macro_rules! bench_fft {
    ($($name: ident: $size: expr)*)=> {
        $(
        fn $name(c: &mut Criterion) -> () {
            c.bench_function(concat!("Bignum_", $size, "_utils_fft"), |b| {
                b.iter(|| {
                    _fft(vec![9; $size]);
                })
            });
        }
    )*
    }
}

bench_fft! {
    size_1024: 1024
    size_2048: 2048
    size_4096: 4096
}

criterion_group!(fft_benches, size_1024, size_2048, size_4096);
criterion_main!(fft_benches);
