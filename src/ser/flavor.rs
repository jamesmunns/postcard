use core::ops::Index;
use cobs::{EncoderState, PushResult};
use heapless::{ArrayLength, Vec};
use core::ops::IndexMut;

pub trait SerFlavor {
    type Output;

    fn try_extend(&mut self, data: &[u8]) -> core::result::Result<(), ()> {
        data.iter()
            .try_for_each(|d| self.try_push(*d))
            .map_err(|_| ())
    }
    fn try_push(&mut self, data: u8) -> core::result::Result<(), ()>;
    fn release(self) -> core::result::Result<Self::Output, ()>;
}

pub struct Slice<'a> {
    pub(crate) buf: &'a mut [u8],
    pub(crate) idx: usize,
}



impl<'a> SerFlavor for Slice<'a> {
    type Output = Self;

    #[inline(always)]
    fn try_extend(&mut self, data: &[u8]) -> core::result::Result<(), ()> {
        let len = data.len();

        if (len + self.idx) > self.buf.len() {
            return Err(());
        }

        self.buf[self.idx..self.idx + len]
            .copy_from_slice(data);

        self.idx += len;

        Ok(())
    }

    #[inline(always)]
    fn try_push(&mut self, data: u8) -> core::result::Result<(), ()> {
        if self.idx >= self.buf.len() {
            return Err(());
        }

        self.buf[self.idx] = data;
        self.idx += 1;

        Ok(())
    }

    fn release(self) -> core::result::Result<Self::Output, ()> {
        Ok(self)
    }
}

pub struct Vanilla<B: ArrayLength<u8>>(Vec<u8, B>);

impl<B: ArrayLength<u8>> Index<usize> for Vanilla<B> {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        &self.0[idx]
    }
}

impl<B: ArrayLength<u8>> IndexMut<usize> for Vanilla<B> {
    fn index_mut(&mut self, idx: usize) -> &mut u8 {
        &mut self.0[idx]
    }
}

impl<'a> Index<usize> for Slice<'a> {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        &self.buf[idx]
    }
}

impl<'a> IndexMut<usize> for Slice<'a> {
    fn index_mut(&mut self, idx: usize) -> &mut u8 {
        &mut self.buf[idx]
    }
}

impl<B: ArrayLength<u8>> Default for Vanilla<B> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<'a, B> SerFlavor for Vanilla<B>
where
    B: ArrayLength<u8>,
{
    type Output = Vec<u8, B>;

    #[inline(always)]
    fn try_extend(&mut self, data: &[u8]) -> core::result::Result<(), ()> {
        self.0.extend_from_slice(data)
    }

    #[inline(always)]
    fn try_push(&mut self, data: u8) -> core::result::Result<(), ()> {
        self.0.push(data).map_err(|_| ())
    }

    fn release(self) -> core::result::Result<Vec<u8, B>, ()> {
        Ok(self.0)
    }
}

pub struct Cobs<B>
where
    B: SerFlavor + IndexMut<usize, Output = u8>,
{
    flav: B,
    cobs: EncoderState,
}

impl<B> Cobs<B>
where
    B: SerFlavor + IndexMut<usize, Output = u8>,
{
    pub(crate) fn new(mut bee: B) -> Self {
        bee.try_push(0).unwrap();
        Self {
            flav: bee,
            cobs: EncoderState::default(),
        }
    }
}

impl<'a, B> SerFlavor for Cobs<B>
where
    B: SerFlavor + IndexMut<usize, Output = u8>,
{
    type Output = <B as SerFlavor>::Output;

    #[inline(always)]
    fn try_push(&mut self, data: u8) -> core::result::Result<(), ()> {
        use PushResult::*;
        match self.cobs.push(data) {
            AddSingle(n) => self.flav.try_push(n),
            ModifyFromStartAndSkip((idx, mval)) => {
                self.flav[idx] = mval;
                self.flav.try_push(0)
            }
            ModifyFromStartAndPushAndSkip((idx, mval, nval)) => {
                self.flav[idx] = mval;
                self.flav.try_push(nval)?;
                self.flav.try_push(0)
            }
        }
    }

    fn release(mut self) -> core::result::Result<Self::Output, ()> {
        let (idx, mval) = self.cobs.finalize();
        self.flav[idx] = mval;
        self.flav.try_push(0)?;
        self.flav.release()
    }
}
