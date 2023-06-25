use clip::dsp::{
    inline_var_hard_clip_fast, inline_var_hard_clip_fast_simd_4, inline_var_hard_clip_faster,
    inline_var_hard_clip_faster_simd_4, inline_var_hard_clip_fastest,
    inline_var_hard_clip_fastest_simd_4, var_hard_clip, var_hard_clip_fast,
    var_hard_clip_fast_simd_4, var_hard_clip_faster, var_hard_clip_faster_simd_4,
    var_hard_clip_fastest, var_hard_clip_fastest_simd_4, var_hard_clip_simd_4, IdentityProcessor,
    MonoProcessor, OversampleX4,
};
use clip::math_utils::{fast_powf, faster_powf, fastest_powf};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, SamplingMode};
use itertools_num::linspace;
use wide::f32x4;

fn bench_var_hard_clip(c: &mut Criterion) {
    let x = black_box([-0.9, -0.2, 0.3, 0.8]);
    let mut group = c.benchmark_group("var hard clip");
    group.sample_size(100).sampling_mode(SamplingMode::Linear);
    group.bench_function("unopt", |b| {
        b.iter(|| {
            for x in x {
                black_box(var_hard_clip(x, black_box(0.843)));
            }
        })
    });
    group.bench_function("fast", |b| {
        b.iter(|| {
            for x in x {
                black_box(var_hard_clip_fast(x, black_box(0.843)));
            }
        })
    });
    group.bench_function("faster", |b| {
        b.iter(|| {
            for x in x {
                black_box(var_hard_clip_faster(x, black_box(0.843)));
            }
        })
    });
    group.bench_function("fastest", |b| {
        b.iter(|| {
            for x in x {
                black_box(var_hard_clip_fastest(x, black_box(0.843)));
            }
        })
    });
    group.bench_function("inline fast", |b| {
        b.iter(|| {
            for x in x {
                black_box(inline_var_hard_clip_fast(x, black_box(0.843)));
            }
        })
    });
    group.bench_function("inline faster", |b| {
        b.iter(|| {
            for x in x {
                black_box(inline_var_hard_clip_faster(x, black_box(0.843)));
            }
        })
    });
    group.bench_function("inline fastest", |b| {
        b.iter(|| {
            for x in x {
                black_box(inline_var_hard_clip_fastest(x, black_box(0.843)));
            }
        })
    });
    group.bench_function("simd 4", |b| {
        b.iter(|| {
            black_box(var_hard_clip_simd_4(
                black_box(f32x4::new(x)),
                black_box(0.843),
            ))
        });
    });
    group.bench_function("fast simd 4", |b| {
        b.iter(|| {
            black_box(var_hard_clip_fast_simd_4(
                black_box(f32x4::new(x)),
                black_box(0.843),
            ))
        });
    });
    group.bench_function("faster simd 4", |b| {
        b.iter(|| {
            black_box(var_hard_clip_faster_simd_4(
                black_box(f32x4::new(x)),
                black_box(0.843),
            ))
        });
    });
    group.bench_function("fastest simd 4", |b| {
        b.iter(|| {
            black_box(var_hard_clip_fastest_simd_4(
                black_box(f32x4::new(x)),
                black_box(0.843),
            ))
        });
    });
    group.bench_function("inline fast simd 4", |b| {
        b.iter(|| {
            black_box(inline_var_hard_clip_fast_simd_4(
                black_box(f32x4::new(x)),
                black_box(0.843),
            ))
        });
    });
    group.bench_function("inline faster simd 4", |b| {
        b.iter(|| {
            black_box(inline_var_hard_clip_faster_simd_4(
                black_box(f32x4::new(x)),
                black_box(0.843),
            ))
        });
    });
    group.bench_function("inline fastest simd 4", |b| {
        b.iter(|| {
            black_box(inline_var_hard_clip_fastest_simd_4(
                black_box(f32x4::new(x)),
                black_box(0.843),
            ))
        });
    });
    group.finish();
}

fn bench_powf(c: &mut Criterion) {
    let mut group = c.benchmark_group("powf");
    let x = 0.6321f32;
    let y = 21.2345f32;
    group.sample_size(10_0);
    group.bench_function("unopt", |b| {
        b.iter(|| black_box(black_box(x).powf(black_box(y))))
    });
    group.bench_function("fast", |b| {
        b.iter(|| black_box(fast_powf(black_box(x), black_box(y))))
    });
    group.bench_function("faster", |b| {
        b.iter(|| black_box(faster_powf(black_box(x), black_box(y))))
    });
    group.bench_function("fastest", |b| {
        b.iter(|| black_box(fastest_powf(black_box(x), black_box(y))))
    });
    group.finish();
}

fn bench_oversample(c: &mut Criterion) {
    let mut os = OversampleX4::new(IdentityProcessor());
    c.bench_function("oversample", |b| {
        b.iter(|| {
            black_box(os.step(black_box(0.5)));
        })
    });
}

criterion_group!(benches, bench_oversample, bench_var_hard_clip, bench_powf);
criterion_main!(benches);
