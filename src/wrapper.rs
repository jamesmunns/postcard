use core::ops::DerefMut;
use core::ops::Deref;
use serde::{ser, Serialize};
use heapless::{Vec, String, ArrayLength};

/// Newtype until heapless grows Serde support
pub struct HeaplessVec<T, B>(Vec<T, B>)
where
    T: Serialize,
    B: ArrayLength<T>;

impl<T, B> Deref for HeaplessVec<T, B>
where
    T: Serialize,
    B: ArrayLength<T>
{
    type Target = Vec<T, B>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, B> DerefMut for HeaplessVec<T, B>
where
    T: Serialize,
    B: ArrayLength<T>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T, B> HeaplessVec<T, B>
where
    T: Serialize,
    B: ArrayLength<T>
{
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl<B> HeaplessString<B>
where
    B: ArrayLength<u8>
{
    pub fn new() -> Self {
        Self(String::new())
    }
}

/// Newtype until heapless grows Serde support
pub struct HeaplessString<B>(String<B>)
where
    B: ArrayLength<u8>;

impl<B> Deref for HeaplessString<B>
where
    B: ArrayLength<u8>
{
    type Target = String<B>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<B> DerefMut for HeaplessString<B>
where
    B: ArrayLength<u8>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T, B> Serialize for HeaplessVec<T, B>
where
    T: Serialize,
    B: ArrayLength<T>
{
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        use ser::SerializeSeq;
        let deref: &[T] = self.deref();
        let mut seq = serializer.serialize_seq(Some(deref.len()))?;
        deref.iter().try_for_each(|t| seq.serialize_element(t))?;
        seq.end()
    }
}

impl<B> Serialize for HeaplessString<B>
where
    B: ArrayLength<u8>
{
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        use ser::SerializeSeq;
        let deref: &[u8] = self.deref().as_bytes();
        let mut seq = serializer.serialize_seq(Some(deref.len()))?;
        deref.iter().try_for_each(|t| seq.serialize_element(t))?;
        seq.end()
    }
}
