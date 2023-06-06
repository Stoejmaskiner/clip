# Clip

## Building

> Note: right now some nightly features are required, so you need to switch to the nightly toolchain to build: `rustup default nightly`

After installing [Rust](https://rustup.rs/), you can compile Clip as follows:

```shell
./build.py bunlde --release

# or
cargo xtask bundle clip --release
```

## Licensing

Inherits the GPLv3 license from [NIH-plug](https://github.com/robbert-vdh/nih-plug), which in turn inherits it from the VST3 bindings.

Some code is directly derived from other open source projects, with permission according to their respective licenses, namely:
- `crate::dsp::Oversample` is based on [FunDSP](https://github.com/SamiPerttu/fundsp)
