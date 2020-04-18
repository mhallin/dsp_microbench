use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn dsp_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Rendering");
    for size in &[
        64usize, 128, 150, 256, 350, 512, 800, 1024, 1800, 2048, 4096,
    ] {
        group.bench_with_input(
            BenchmarkId::new("One frame per call", *size),
            size,
            |b, size| {
                b.iter_with_setup(
                    || {
                        (
                            vec![0.0f64; *size],
                            dsp_perf::one_frame_per_call::Synth::new(44100.0),
                        )
                    },
                    |(mut data, mut synth)| {
                        synth.render(&mut data);
                        data
                    },
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Fixed batch size (64 frames)", *size),
            size,
            |b, size| {
                b.iter_with_setup(
                    || {
                        (
                            vec![0.0f64; *size],
                            dsp_perf::fixed_batch_size::Synth::new(44100.0),
                        )
                    },
                    |(mut data, mut synth)| {
                        synth.render(&mut data);
                        data
                    },
                );
            },
        );
    }
    group.finish();
}

criterion_group!(benches, dsp_bench);
criterion_main!(benches);
