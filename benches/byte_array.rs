use criterion::{black_box, criterion_group, criterion_main, Criterion};
use postcard::FixedSizeByteArray;
use serde::{Deserialize, Serialize};
use serde_byte_array::ByteArray;
use serde_bytes::{Bytes, ByteBuf};

fn serialize<const N: usize, const B: usize>(c: &mut Criterion)
where
    [u8; N]: Serialize
{
    let own: &_ = &FixedSizeByteArray::from([0; N]);
    let bytes: &_ = Bytes::new(&[0; N]);
    let byte_array: &_ = &ByteArray::new([0; N]);
    let big_array: &_ = &serde_big_array::Array([0; N]);
    let fixed: &_ = &[0; N];
    let variable: &[u8] = &[0; N];
    let mut buf = [0; B];
    let mut group = c.benchmark_group(format!("serialize{}", N));
    group.bench_function("own", |b| b.iter(|| {
        let _ = black_box(postcard::to_slice(black_box(own), &mut buf).unwrap());
    }));
    group.bench_function("bytes", |b| b.iter(|| {
        let _ = black_box(postcard::to_slice(black_box(bytes), &mut buf).unwrap());
    }));
    group.bench_function("byte_array", |b| b.iter(|| {
        let _ = black_box(postcard::to_slice(black_box(byte_array), &mut buf).unwrap());
    }));
    group.bench_function("big_array", |b| b.iter(|| {
        let _ = black_box(postcard::to_slice(black_box(big_array), &mut buf).unwrap());
    }));
    group.bench_function("fixed_size", |b| b.iter(|| {
        let _ = black_box(postcard::to_slice(black_box(fixed), &mut buf).unwrap());
    }));
    group.bench_function("variable_size", |b| b.iter(|| {
        let _ = black_box(postcard::to_slice(black_box(variable), &mut buf).unwrap());
    }));
    group.finish();
}

fn deserialize<const N: usize, const SN: usize>(c: &mut Criterion)
where
    [u8; N]: Deserialize<'static>
{
    let data = &[0; N];
    let mut data_prefixed = [0; SN];
    data_prefixed[0] = N as u8;
    let data_prefixed = &data_prefixed;

    let mut group = c.benchmark_group(format!("deserialize{}", N));
    group.bench_function("own", |b| b.iter(|| {
        let _: FixedSizeByteArray<N> = black_box(postcard::from_bytes(black_box(data)).unwrap());
    }));
    group.bench_function("bytes", |b| b.iter(|| {
        let _: ByteBuf = black_box(postcard::from_bytes(black_box(data_prefixed)).unwrap());
    }));
    group.bench_function("byte_array", |b| b.iter(|| {
        let _: ByteArray<N> = black_box(postcard::from_bytes(black_box(data_prefixed)).unwrap());
    }));
    group.bench_function("big_array", |b| b.iter(|| {
        let _: serde_big_array::Array<u8, N> = black_box(postcard::from_bytes(black_box(data)).unwrap());
    }));
    group.bench_function("fixed_size", |b| b.iter(|| {
        let _: [u8; N] = black_box(postcard::from_bytes(black_box(data)).unwrap());
    }));
    group.bench_function("variable_size", |b| b.iter(|| {
        let _: Vec<u8> = black_box(postcard::from_bytes(black_box(data_prefixed)).unwrap());
    }));
    group.finish();
}

fn serialize0(c: &mut Criterion) { serialize::<0, 64>(c) }
fn serialize1(c: &mut Criterion) { serialize::<1, 64>(c) }
fn serialize2(c: &mut Criterion) { serialize::<2, 64>(c) }
fn serialize4(c: &mut Criterion) { serialize::<4, 64>(c) }
fn serialize8(c: &mut Criterion) { serialize::<8, 64>(c) }
fn serialize16(c: &mut Criterion) { serialize::<16, 64>(c) }
fn serialize32(c: &mut Criterion) { serialize::<32, 64>(c) }

fn deserialize0(c: &mut Criterion) { deserialize::<0, 1>(c) }
fn deserialize1(c: &mut Criterion) { deserialize::<1, 2>(c) }
fn deserialize2(c: &mut Criterion) { deserialize::<2, 3>(c) }
fn deserialize4(c: &mut Criterion) { deserialize::<4, 5>(c) }
fn deserialize8(c: &mut Criterion) { deserialize::<8, 9>(c) }
fn deserialize16(c: &mut Criterion) { deserialize::<16, 17>(c) }
fn deserialize32(c: &mut Criterion) { deserialize::<32, 33>(c) }

criterion_group!(byte_array,
    serialize0,
    serialize1,
    serialize2,
    serialize4,
    serialize8,
    serialize16,
    serialize32,

    deserialize0,
    deserialize1,
    deserialize2,
    deserialize4,
    deserialize8,
    deserialize16,
    deserialize32,
);

criterion_main!(byte_array);
