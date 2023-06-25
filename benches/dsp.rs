use clip::dsp::{
    var_hard_clip, var_hard_clip_simd_4, IdentityProcessor, MonoProcessor, OversampleX4,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use itertools_num::linspace;
use wide::f32x4;

fn bench_var_hard_clip(c: &mut Criterion) {
    let x = black_box([-0.9, -0.2, 0.3, 0.8]);
    let mut group = c.benchmark_group("var hard clip");
    group.bench_function("unopt", |b| {
        b.iter(|| {
            for x in x {
                black_box(var_hard_clip(x, black_box(0.843)));
            }
        })
    });
    group.bench_function("simd 4", |b| {
        b.iter(|| black_box(var_hard_clip_simd_4(f32x4::new(x), 0.843)));
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

criterion_group!(benches, bench_var_hard_clip, bench_oversample);
criterion_main!(benches);
