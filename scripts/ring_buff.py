"""Prototype for crate::dsp::RingBuffer"""
from typing_extensions import Self


class RingBuffer:
    def __init__(self, max_size: int):
        self.buffer = [0.0 for _ in range(max_size + 1)]
        self.write_head = 0

    def push(self, x: float) -> Self:
        self.write_head += 1
        self.write_head %= len(self.buffer)
        self.buffer[self.write_head] = x
        return self

    def tap(self, delay: int) -> float:
        assert delay < len(self.buffer)
        idx = (self.write_head - delay) % len(self.buffer)
        return self.buffer[idx]

    def __getitem__(self, idx) -> float:
        return self.tap(idx)


class FixedDelay:
    def __init__(self, delay: int):
        self.buffer = RingBuffer(delay + 1)
        self.delay = delay

    def step(self, x: float):
        self.buffer.push(x)
        return self.buffer.tap(self.delay)


if __name__ == "__main__":
    r = RingBuffer(3)
    assert r.push(1.0).tap(3) == 0.0
    assert r.push(2.0).tap(3) == 0.0
    assert r.push(3.0).tap(3) == 0.0
    for i in range(4, 100):
        assert r.push(float(i)).tap(3) == float(i - 3)

    r = FixedDelay(3)
    assert r.step(1.0) == 0.0
    assert r.step(2.0) == 0.0
    assert r.step(3.0) == 0.0
    for i in range(4, 100):
        assert r.step(float(i)) == float(i - 3)
