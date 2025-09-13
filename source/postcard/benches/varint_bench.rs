use criterion::criterion_main;
#[cfg(feature = "bench_private")]
mod bench {
    use criterion::{criterion_group, Criterion};
    use postcard::varint::{
        varint_max, varint_u128, varint_u16, varint_u32, varint_u64, varint_usize,
    };
    use std::hint::black_box;

    // usize benchmarks
    fn bench_varint_usize_small(c: &mut Criterion) {
        let mut out = [0u8; varint_max::<usize>()];
        c.bench_function("varint_usize_small", |b| {
            b.iter(|| {
                let _ = varint_usize(black_box(42), black_box(&mut out));
            })
        });
    }

    fn bench_varint_usize_medium(c: &mut Criterion) {
        let mut out = [0u8; varint_max::<usize>()];
        c.bench_function("varint_usize_medium", |b| {
            b.iter(|| {
                let _ = varint_usize(black_box(123456), black_box(&mut out));
            })
        });
    }

    fn bench_varint_usize_large(c: &mut Criterion) {
        let mut out = [0u8; varint_max::<usize>()];
        c.bench_function("varint_usize_large", |b| {
            b.iter(|| {
                let _ = varint_usize(black_box(usize::MAX), black_box(&mut out));
            })
        });
    }

    fn bench_varint_usize_zero(c: &mut Criterion) {
        let mut out = [0u8; varint_max::<usize>()];
        c.bench_function("varint_usize_zero", |b| {
            b.iter(|| {
                let _ = varint_usize(black_box(0), black_box(&mut out));
            })
        });
    }

    // u16 benchmarks
    fn bench_varint_u16_small(c: &mut Criterion) {
        let mut out = [0u8; varint_max::<u16>()];
        c.bench_function("varint_u16_small", |b| {
            b.iter(|| {
                let _ = varint_u16(black_box(42), black_box(&mut out));
            })
        });
    }

    fn bench_varint_u16_medium(c: &mut Criterion) {
        let mut out = [0u8; varint_max::<u16>()];
        c.bench_function("varint_u16_medium", |b| {
            b.iter(|| {
                let _ = varint_u16(black_box(1234), black_box(&mut out));
            })
        });
    }

    fn bench_varint_u16_large(c: &mut Criterion) {
        let mut out = [0u8; varint_max::<u16>()];
        c.bench_function("varint_u16_large", |b| {
            b.iter(|| {
                let _ = varint_u16(black_box(u16::MAX), black_box(&mut out));
            })
        });
    }

    fn bench_varint_u16_zero(c: &mut Criterion) {
        let mut out = [0u8; varint_max::<u16>()];
        c.bench_function("varint_u16_zero", |b| {
            b.iter(|| {
                let _ = varint_u16(black_box(0), black_box(&mut out));
            })
        });
    }

    // u32 benchmarks
    fn bench_varint_u32_small(c: &mut Criterion) {
        let mut out = [0u8; varint_max::<u32>()];
        c.bench_function("varint_u32_small", |b| {
            b.iter(|| {
                let _ = varint_u32(black_box(42), black_box(&mut out));
            })
        });
    }

    fn bench_varint_u32_medium(c: &mut Criterion) {
        let mut out = [0u8; varint_max::<u32>()];
        c.bench_function("varint_u32_medium", |b| {
            b.iter(|| {
                let _ = varint_u32(black_box(123456), black_box(&mut out));
            })
        });
    }

    fn bench_varint_u32_large(c: &mut Criterion) {
        let mut out = [0u8; varint_max::<u32>()];
        c.bench_function("varint_u32_large", |b| {
            b.iter(|| {
                let _ = varint_u32(black_box(u32::MAX), black_box(&mut out));
            })
        });
    }

    fn bench_varint_u32_zero(c: &mut Criterion) {
        let mut out = [0u8; varint_max::<u32>()];
        c.bench_function("varint_u32_zero", |b| {
            b.iter(|| {
                let _ = varint_u32(black_box(0), black_box(&mut out));
            })
        });
    }

    // u64 benchmarks
    fn bench_varint_u64_small(c: &mut Criterion) {
        let mut out = [0u8; varint_max::<u64>()];
        c.bench_function("varint_u64_small", |b| {
            b.iter(|| {
                let _ = varint_u64(black_box(42), black_box(&mut out));
            })
        });
    }

    fn bench_varint_u64_medium(c: &mut Criterion) {
        let mut out = [0u8; varint_max::<u64>()];
        c.bench_function("varint_u64_medium", |b| {
            b.iter(|| {
                let _ = varint_u64(black_box(123456), black_box(&mut out));
            })
        });
    }

    fn bench_varint_u64_large(c: &mut Criterion) {
        let mut out = [0u8; varint_max::<u64>()];
        c.bench_function("varint_u64_large", |b| {
            b.iter(|| {
                let _ = varint_u64(black_box(u64::MAX), black_box(&mut out));
            })
        });
    }

    fn bench_varint_u64_zero(c: &mut Criterion) {
        let mut out = [0u8; varint_max::<u64>()];
        c.bench_function("varint_u64_zero", |b| {
            b.iter(|| {
                let _ = varint_u64(black_box(0), black_box(&mut out));
            })
        });
    }

    // u128 benchmarks
    fn bench_varint_u128_small(c: &mut Criterion) {
        let mut out = [0u8; varint_max::<u128>()];
        c.bench_function("varint_u128_small", |b| {
            b.iter(|| {
                let _ = varint_u128(black_box(42), black_box(&mut out));
            })
        });
    }

    fn bench_varint_u128_medium(c: &mut Criterion) {
        let mut out = [0u8; varint_max::<u128>()];
        c.bench_function("varint_u128_medium", |b| {
            b.iter(|| {
                let _ = varint_u128(black_box(12345678901234567890), black_box(&mut out));
            })
        });
    }

    fn bench_varint_u128_large(c: &mut Criterion) {
        let mut out = [0u8; varint_max::<u128>()];
        c.bench_function("varint_u128_large", |b| {
            b.iter(|| {
                let _ = varint_u128(black_box(u128::MAX), black_box(&mut out));
            })
        });
    }

    fn bench_varint_u128_zero(c: &mut Criterion) {
        let mut out = [0u8; varint_max::<u128>()];
        c.bench_function("varint_u128_zero", |b| {
            b.iter(|| {
                let _ = varint_u128(black_box(0), black_box(&mut out));
            })
        });
    }

    criterion_group!(
        benches,
        bench_varint_usize_small,
        bench_varint_usize_medium,
        bench_varint_usize_large,
        bench_varint_usize_zero,
        bench_varint_u16_small,
        bench_varint_u16_medium,
        bench_varint_u16_large,
        bench_varint_u16_zero,
        bench_varint_u32_small,
        bench_varint_u32_medium,
        bench_varint_u32_large,
        bench_varint_u32_zero,
        bench_varint_u64_small,
        bench_varint_u64_medium,
        bench_varint_u64_large,
        bench_varint_u64_zero,
        bench_varint_u128_small,
        bench_varint_u128_medium,
        bench_varint_u128_large,
        bench_varint_u128_zero
    );
}

#[cfg(not(feature = "bench_private"))]
mod bench {
    use criterion::{criterion_group, Criterion};
    fn dummy(_: &mut Criterion) {}
    criterion_group!(benches, dummy);
}

use bench::benches;

criterion_main!(benches);
