use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

#[derive(Copy, Clone)]
struct AOSIncrementerData {
    phase: f64,
    output: f64,
}

pub struct AOSIncrementer {
    data: [AOSIncrementerData; 64],
    last_phase: f64,
}

impl AOSIncrementer {
    fn new() -> Self {
        AOSIncrementer {
            data: [AOSIncrementerData {
                phase: 0.0,
                output: 0.0,
            }; 64],
            last_phase: 0.0,
        }
    }

    #[inline(never)]
    fn render_batch(&mut self) {
        for data in self.data.iter_mut() {
            data.phase = self.last_phase;
            data.output = parabolic_sine(-2.0 * data.phase * std::f64::consts::PI);
            self.last_phase += 0.25;
        }
    }

    pub fn render(&mut self, output: &mut [f64]) {
        for batch in output.chunks_exact_mut(64) {
            self.render_batch();

            for (output, data) in batch.iter_mut().zip(self.data.iter()) {
                *output = data.output;
            }
        }
    }
}

pub struct BatchIncrementer {
    phase: [f64; 64],
    output: [f64; 64],
    last_phase: f64,
}

impl BatchIncrementer {
    fn new() -> Self {
        BatchIncrementer {
            phase: [0.0; 64],
            output: [0.0; 64],
            last_phase: 0.0,
        }
    }

    #[inline(never)]
    fn render_phase_batch(&mut self) {
        for v in self.phase.iter_mut() {
            *v = self.last_phase;
            self.last_phase += 0.25;
            self.last_phase = self.last_phase.rem_euclid(1.0);
        }
    }

    #[inline(never)]
    fn render_output(&mut self) {
        for (output, phase) in self.output.iter_mut().zip(self.phase.iter()) {
            *output = parabolic_sine(-2.0 * *phase * std::f64::consts::PI);
        }
    }

    pub fn render(&mut self, data: &mut [f64]) {
        for batch in data.chunks_exact_mut(64) {
            self.render_phase_batch();
            self.render_output();
            batch.copy_from_slice(&self.output);
        }
    }
}

pub struct Incrementer {
    last_phase: f64,
}

impl Incrementer {
    fn new() -> Self {
        Incrementer { last_phase: 0.0 }
    }

    #[inline(never)]
    fn render_phase_value(&mut self) -> f64 {
        let v = self.last_phase;
        self.last_phase += 0.25;
        self.last_phase = self.last_phase.rem_euclid(1.0);
        v
    }

    pub fn render(&mut self, data: &mut [f64]) {
        for v in data.iter_mut() {
            let phase = self.render_phase_value();
            *v = parabolic_sine(-2.0 * phase * std::f64::consts::PI);
        }
    }
}

// #[inline(never)]
fn parabolic_sine(x: f64) -> f64 {
    use std::f64::consts::PI;

    const B: f64 = 4.0 / PI;
    const C: f64 = -4.0 / (PI * PI);
    const P: f64 = 0.225;
    let mut y = B * x + C * x * x.abs();

    y = P * (y * y.abs() - y) + y;

    y
}

fn dsp_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Rendering");
    for size in &[64usize, 256, 1024, 4096] {
        group.bench_with_input(
            BenchmarkId::new("One frame per call", *size),
            size,
            |b, size| {
                b.iter_with_setup(
                    || (vec![0.0f64; *size], Incrementer::new()),
                    |(mut data, mut incrementer)| {
                        incrementer.render(&mut data);
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
                    || (vec![0.0f64; *size], BatchIncrementer::new()),
                    |(mut data, mut incrementer)| {
                        // synth.render(&mut data)
                        incrementer.render(&mut data);
                        data
                    },
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Array-of-structs (64 frames batch)", *size),
            size,
            |b, size| {
                b.iter_with_setup(
                    || (vec![0.0f64; *size], AOSIncrementer::new()),
                    |(mut data, mut incrementer)| {
                        // synth.render(&mut data)
                        incrementer.render(&mut data);
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
