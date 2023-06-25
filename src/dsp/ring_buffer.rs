use super::MonoProcessor;
use num_traits::FromPrimitive;
use std::ops::Index;

#[derive(Debug, Clone)]
pub(super) struct RingBuffer<const N: usize> {
    buffer: [f32; N],
    write_head: usize,
}

impl<const N: usize> RingBuffer<N> {
    pub(super) fn new() -> Self {
        Self {
            buffer: [0.0; N],
            write_head: 0,
        }
    }

    pub(super) fn push(&mut self, x: f32) -> &mut Self {
        self.write_head += 1;
        self.write_head %= N;
        self.buffer[self.write_head] = x;
        self
    }

    pub(super) fn tap(&self, delay: usize) -> f32 {
        assert!(delay < N);
        let idx = N + self.write_head - delay;
        let idx = idx % N;
        self.buffer[idx]
    }

    #[inline]
    pub(super) fn len(&self) -> usize {
        N
    }

    pub(super) fn reset(&mut self) {
        for s in &mut self.buffer {
            *s = 0.0;
        }
    }
}

impl<'a, const N: usize> IntoIterator for &'a RingBuffer<N> {
    type Item = f32;

    type IntoIter = RingBufferIter<'a, N>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            ring_buffer: self,
            index: 0,
        }
    }
}

pub struct RingBufferIter<'a, const N: usize> {
    ring_buffer: &'a RingBuffer<N>,
    index: usize,
}

impl<'a, const N: usize> Iterator for RingBufferIter<'a, N> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.ring_buffer.tap(self.index);
        self.index += 1;
        if self.index >= N {
            None
        } else {
            Some(ret)
        }
    }
}

/// a ring buffer that can store at most 255 values, this specific size is
/// for optimization reasons
///
/// the internal size is fixed to 256, this makes it so that wrapping around
/// the buffer is reduced to truncating the higher bytes, which is done in 2
/// assembly instructions, also checking for division by 0 is elided
///
/// the capacity is 255 and not 256 because one slot is always lost when pushing
// TODO: profile against RingBuffer
#[derive(Debug, Clone)]
pub struct SmallRingBuffer {
    buffer: [f32; 256],
    write_head: usize,
}

impl SmallRingBuffer {
    pub(super) fn new() -> Self {
        Self {
            buffer: [0.0; 256],
            write_head: 0,
        }
    }

    pub(super) fn push(&mut self, x: f32) -> &mut Self {
        self.write_head += 1;
        self.write_head &= 255;
        unsafe { *self.buffer.get_unchecked_mut(self.write_head) = x };
        self
    }

    pub(super) fn tap(&self, delay: usize) -> f32 {
        assert!(delay < 255);
        let idx = 256 + self.write_head - delay;
        let idx = idx & 255;
        unsafe { *self.buffer.get_unchecked(idx) }
    }

    #[inline]
    pub(super) fn len(&self) -> usize {
        255
    }

    pub(super) fn reset(&mut self) {
        for s in &mut self.buffer {
            *s = 0.0;
        }
    }
}
