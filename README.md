# Clip

## Building

> Note: right now some nightly features are required, so you need to switch to the nightly toolchain to build: `rustup default nightly`

After installing [Rust](https://rustup.rs/), you can compile Clip as follows:

```shell
./build.py bunlde --release

# or
cargo xtask bundle clip --release
```

## Optimization

The benchmark is running 5 instances on ALSA driver, with 1024 sample buffer, at 48000 Hz, no GUI. Note that GUI uses more than half of the CPU, but usally only one GUI is open per session, so it's not that important. It should probably be optimized a bit though **(TODO)**

Deadline: 21.333 ms

Note that max load and jitter figures are not very useful because capturing performance increases jitter considerably.

**Before any optimization:**
- average load: 7.566 ms (~1.5 ms per instance)
- max load: 32.767 ms
- jitter: 5.82 %

Flamegraph suggests that these functions are the hottest:
- `dsp::ring_buffer::RingBuffer::tap` (by a pretty big margin)
- `dsp::OversampleX4::step_down_2x`
- `dsp::OversampleX4::step_up_2x`
- `dsp::OversampleX4::step_up_4x`
- `f32::powf` (as a result of calling `dsp::var_hard_clip`)

It seems that it's worth optimizing away some `powf()` in `dsp::var_hard_clip()` and the oversampling logic. Also these functions are worth inlining.

**opt 1: inlining**

Inlining made it ever so slightly worse, not interesting.

**opt 2: fastapprox::fast::powf**

Slightly worse, not interesting.

**opt 3: fastapprox::faster::powf**

Also slightly worse, not interesting.


## Optimization 2

ALSA driver, sample rate 48kHz, 1024 buffer size (21.333 ms deadline)

Before bufferization: ~2 ms

Basic bufferization: ~1.8 ms

Fast oversample: ~1.7 ms


## Licensing

Inherits the GPLv3 license from [NIH-plug](https://github.com/robbert-vdh/nih-plug), which in turn inherits it from the VST3 bindings.

Some code is directly derived from other open source projects, with permission according to their respective licenses, namely:
- `crate::dsp::Oversample` is based on [FunDSP](https://github.com/SamiPerttu/fundsp)
