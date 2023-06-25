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
