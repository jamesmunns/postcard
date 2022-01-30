use core::{
    num::{NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize, NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize},
    marker::PhantomData,
};

/// This trait is used to enforce the maximum size required to
/// store the serialization of a given type.
pub trait SerializeMaxSize {
    /// The maximum possible size that the serialization of this
    /// type can have, in bytes.
    const MAX_SIZE: usize;
}

impl SerializeMaxSize for bool {
    const MAX_SIZE: usize = 1;
}

impl SerializeMaxSize for i8 {
    const MAX_SIZE: usize = 1;
}

impl SerializeMaxSize for i16 {
    const MAX_SIZE: usize = 2;
}

impl SerializeMaxSize for i32 {
    const MAX_SIZE: usize = 4;
}

impl SerializeMaxSize for i64 {
    const MAX_SIZE: usize = 8;
}

impl SerializeMaxSize for i128 {
    const MAX_SIZE: usize = 8;
}

impl SerializeMaxSize for isize {
    const MAX_SIZE: usize = i64::MAX_SIZE;
}

impl SerializeMaxSize for u8 {
    const MAX_SIZE: usize = 1;
}

impl SerializeMaxSize for u16 {
    const MAX_SIZE: usize = 2;
}

impl SerializeMaxSize for u32 {
    const MAX_SIZE: usize = 4;
}

impl SerializeMaxSize for u64 {
    const MAX_SIZE: usize = 8;
}

impl SerializeMaxSize for u128 {
    const MAX_SIZE: usize = 8;
}

impl SerializeMaxSize for usize {
    const MAX_SIZE: usize = u64::MAX_SIZE;
}

impl SerializeMaxSize for f32 {
    const MAX_SIZE: usize = 4;
}

impl SerializeMaxSize for f64 {
    const MAX_SIZE: usize = 8;
}

impl SerializeMaxSize for char {
    const MAX_SIZE: usize = 5;
}

impl<T: SerializeMaxSize> SerializeMaxSize for Option<T> {
    const MAX_SIZE: usize = T::MAX_SIZE + 1;
}

impl<T: SerializeMaxSize, E: SerializeMaxSize> SerializeMaxSize for Result<T, E> {
    const MAX_SIZE: usize = max(T::MAX_SIZE, E::MAX_SIZE) + 1;
}

impl SerializeMaxSize for () {
    const MAX_SIZE: usize = 0;
}

impl<T: SerializeMaxSize, const N: usize> SerializeMaxSize for [T; N] {
    const MAX_SIZE: usize = T::MAX_SIZE * N;
}

impl<T: SerializeMaxSize> SerializeMaxSize for &'_ T {
    const MAX_SIZE: usize = T::MAX_SIZE;
}

impl<T: SerializeMaxSize> SerializeMaxSize for &'_ mut T {
    const MAX_SIZE: usize = T::MAX_SIZE;
}

impl SerializeMaxSize for NonZeroI8 {
    const MAX_SIZE: usize = i8::MAX_SIZE;
}

impl SerializeMaxSize for NonZeroI16 {
    const MAX_SIZE: usize = i16::MAX_SIZE;
}

impl SerializeMaxSize for NonZeroI32 {
    const MAX_SIZE: usize = i32::MAX_SIZE;
}

impl SerializeMaxSize for NonZeroI64 {
    const MAX_SIZE: usize = i64::MAX_SIZE;
}

impl SerializeMaxSize for NonZeroI128 {
    const MAX_SIZE: usize = i128::MAX_SIZE;
}

impl SerializeMaxSize for NonZeroIsize {
    const MAX_SIZE: usize = isize::MAX_SIZE;
}

impl SerializeMaxSize for NonZeroU8 {
    const MAX_SIZE: usize = u8::MAX_SIZE;
}

impl SerializeMaxSize for NonZeroU16 {
    const MAX_SIZE: usize = u16::MAX_SIZE;
}

impl SerializeMaxSize for NonZeroU32 {
    const MAX_SIZE: usize = u32::MAX_SIZE;
}

impl SerializeMaxSize for NonZeroU64 {
    const MAX_SIZE: usize = u64::MAX_SIZE;
}

impl SerializeMaxSize for NonZeroU128 {
    const MAX_SIZE: usize = u128::MAX_SIZE;
}

impl SerializeMaxSize for NonZeroUsize {
    const MAX_SIZE: usize = usize::MAX_SIZE;
}

impl<T: SerializeMaxSize> SerializeMaxSize for PhantomData<T> {
    const MAX_SIZE: usize = 0;
}

impl<A: SerializeMaxSize> SerializeMaxSize for (A,) {
    const MAX_SIZE: usize = A::MAX_SIZE;
}

impl<A: SerializeMaxSize, B: SerializeMaxSize> SerializeMaxSize for (A, B) {
    const MAX_SIZE: usize = A::MAX_SIZE + B::MAX_SIZE;
}

impl<A: SerializeMaxSize, B: SerializeMaxSize, C: SerializeMaxSize> SerializeMaxSize for (A, B, C) {
    const MAX_SIZE: usize = A::MAX_SIZE + B::MAX_SIZE + C::MAX_SIZE;
}

impl<A: SerializeMaxSize, B: SerializeMaxSize, C: SerializeMaxSize, D: SerializeMaxSize> SerializeMaxSize for (A, B, C, D) {
    const MAX_SIZE: usize = A::MAX_SIZE + B::MAX_SIZE + C::MAX_SIZE + D::MAX_SIZE;
}

impl<A: SerializeMaxSize, B: SerializeMaxSize, C: SerializeMaxSize, D: SerializeMaxSize, E: SerializeMaxSize> SerializeMaxSize for (A, B, C, D, E) {
    const MAX_SIZE: usize = A::MAX_SIZE + B::MAX_SIZE + C::MAX_SIZE + D::MAX_SIZE + E::MAX_SIZE;
}

impl<A: SerializeMaxSize, B: SerializeMaxSize, C: SerializeMaxSize, D: SerializeMaxSize, E: SerializeMaxSize, F: SerializeMaxSize> SerializeMaxSize for (A, B, C, D, E, F) {
    const MAX_SIZE: usize = A::MAX_SIZE + B::MAX_SIZE + C::MAX_SIZE + D::MAX_SIZE + E::MAX_SIZE + F::MAX_SIZE;
}

#[cfg(feature = "heapless")]
impl<T: SerializeMaxSize, const N: usize> SerializeMaxSize for heapless::Vec<T, N> {
    const MAX_SIZE: usize = <[T; N]>::MAX_SIZE + varint_size(N);
}

#[cfg(feature = "heapless")]
impl<const N: usize> SerializeMaxSize for heapless::String<N> {
    const MAX_SIZE: usize = <[u8; N]>::MAX_SIZE + varint_size(N);
}

const fn varint_size(max_n: usize) -> usize {
    const BITS_PER_BYTE: usize = 8;
    const BITS_PER_VARINT_BYTE: usize = 7;

    // How many data bits do we need for `max_n`.
    let bits = core::mem::size_of::<usize>() * BITS_PER_BYTE - max_n.leading_zeros() as usize;

    // We add (BITS_PER_BYTE - 1), to ensure any integer divisions
    // with a remainder will always add exactly one full byte, but
    // an evenly divided number of bits will be the same
    let roundup_bits = bits + (BITS_PER_BYTE - 1);

    // Apply division, using normal "round down" integer division
    roundup_bits / BITS_PER_VARINT_BYTE
}

const fn max(lhs: usize, rhs: usize) -> usize {
    if lhs > rhs {
        lhs
    } else {
        rhs
    }
}