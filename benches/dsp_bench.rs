use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

#[derive(Copy, Clone)]
struct AOSIncrementerData {
    phase_increment: f64,
    output: f64,
}

struct AOSIncrementer {
    data: [AOSIncrementerData; 64],
    last_phase: f64,
}

impl AOSIncrementer {
    fn new() -> Self {
        AOSIncrementer {
            data: [AOSIncrementerData {
                phase_increment: 0.25,
                output: 0.0,
            }; 64],
            last_phase: 0.0,
        }
    }

    #[inline(never)]
    fn render(&mut self) {
        for data in self.data.iter_mut() {
            let phase = self.last_phase;
            self.last_phase += data.phase_increment;
            data.output = parabolic_sine(-2.0 * phase * std::f64::consts::PI);
        }
    }
}

pub struct AOSTest {
    modulator: AOSIncrementer,
    oscillator: AOSIncrementer,
    base_increment: f64,
}

impl AOSTest {
    fn new() -> Self {
        AOSTest {
            modulator: AOSIncrementer::new(),
            oscillator: AOSIncrementer::new(),
            base_increment: 0.125,
        }
    }

    pub fn render(&mut self, output: &mut [f64]) {
        for batch in output.chunks_exact_mut(64) {
            self.modulator.render();

            for (dest, src) in self
                .oscillator
                .data
                .iter_mut()
                .zip(self.modulator.data.iter())
            {
                dest.phase_increment = self.base_increment * 2.0f64.powf(src.output);
            }

            self.oscillator.render();

            for (dest, src) in batch.iter_mut().zip(self.oscillator.data.iter()) {
                *dest = src.output;
            }
        }
    }
}

struct BatchIncrementer {
    phase_increment: [f64; 64],
    phase: [f64; 64],
    output: [f64; 64],
    last_phase: f64,
}

impl BatchIncrementer {
    fn new() -> Self {
        BatchIncrementer {
            phase_increment: [0.25; 64],
            phase: [0.0; 64],
            output: [0.0; 64],
            last_phase: 0.0,
        }
    }

    fn render_phase_batch(&mut self) {
        for (v, incr) in self.phase.iter_mut().zip(self.phase_increment.iter()) {
            *v = self.last_phase;
            self.last_phase += incr;
            self.last_phase = wrap01(self.last_phase);
        }
    }

    fn render_output(&mut self) {
        for (output, phase) in self.output.iter_mut().zip(self.phase.iter()) {
            *output = parabolic_sine(-2.0 * *phase * std::f64::consts::PI);
        }
    }

    #[inline(never)]
    pub fn render(&mut self) {
        self.render_phase_batch();
        self.render_output();
    }
}

pub struct BatchTest {
    modulator: BatchIncrementer,
    oscillator: BatchIncrementer,
    base_increment: f64,
}

impl BatchTest {
    fn new() -> Self {
        BatchTest {
            modulator: BatchIncrementer::new(),
            oscillator: BatchIncrementer::new(),
            base_increment: 0.125,
        }
    }

    pub fn render(&mut self, data: &mut [f64]) {
        for batch in data.chunks_exact_mut(64) {
            self.modulator.render_output();

            for (dest, src) in self
                .oscillator
                .phase_increment
                .iter_mut()
                .zip(self.modulator.output.iter())
            {
                *dest = self.base_increment * 2.0f64.powf(*src);
            }

            self.oscillator.render();
            batch.copy_from_slice(&self.oscillator.output);
        }
    }
}

struct PerFrameIncrementer {
    last_phase: f64,
    phase_increment: f64,
}

impl PerFrameIncrementer {
    fn new() -> Self {
        PerFrameIncrementer {
            last_phase: 0.0,
            phase_increment: 0.25,
        }
    }

    fn render_phase_value(&mut self) -> f64 {
        let v = self.last_phase;
        self.last_phase += self.phase_increment;
        self.last_phase = wrap01(self.last_phase);
        v
    }

    #[inline(never)]
    fn render(&mut self) -> f64 {
        let phase = self.render_phase_value();
        parabolic_sine(-2.0 * phase * std::f64::consts::PI)
    }
}

pub struct PerFrameTest {
    modulator: PerFrameIncrementer,
    oscillator: PerFrameIncrementer,
    base_increment: f64,
}

impl PerFrameTest {
    fn new() -> Self {
        PerFrameTest {
            modulator: PerFrameIncrementer::new(),
            oscillator: PerFrameIncrementer::new(),
            base_increment: 0.125,
        }
    }

    pub fn render(&mut self, output: &mut [f64]) {
        for output in output.iter_mut() {
            let phase_increment_mod = self.modulator.render();
            self.oscillator.phase_increment =
                self.base_increment * 2.0f64.powf(phase_increment_mod);
            *output = self.oscillator.render();
        }
    }
}

#[inline(never)]
fn parabolic_sine(x: f64) -> f64 {
    use std::f64::consts::PI;

    const B: f64 = 4.0 / PI;
    const C: f64 = -4.0 / (PI * PI);
    const P: f64 = 0.225;
    let mut y = B * x + C * x * x.abs();

    y = P * (y * y.abs() - y) + y;

    y
}

fn wrap01(x: f64) -> f64 {
    if x >= 0.0 && x <= 1.0 {
        x
    } else {
        x.rem_euclid(1.0)
    }
}

fn dsp_mini_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Rendering");
    for size in &[64usize, 256, 1024, 4096] {
        group.bench_with_input(
            BenchmarkId::new("One frame per call", *size),
            size,
            |b, size| {
                b.iter_with_setup(
                    || (vec![0.0f64; *size], PerFrameTest::new()),
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
                    || (vec![0.0f64; *size], BatchTest::new()),
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
                    || (vec![0.0f64; *size], AOSTest::new()),
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

fn dsp_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Rendering (full implementation)");

    for size in &[64usize, 256, 1024, 4096] {
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
                    |(mut data, mut incrementer)| {
                        incrementer.render(&mut data);
                        data
                    },
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Fixed batch size (struct-of-arrays)", *size),
            size,
            |b, size| {
                b.iter_with_setup(
                    || {
                        (
                            vec![0.0f64; *size],
                            dsp_perf::fixed_batch_size::Synth::new(44100.0),
                        )
                    },
                    |(mut data, mut incrementer)| {
                        // synth.render(&mut data)
                        incrementer.render(&mut data);
                        data
                    },
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Fixed batch size (array-of-structs)", *size),
            size,
            |b, size| {
                b.iter_with_setup(
                    || {
                        (
                            vec![0.0f64; *size],
                            dsp_perf::array_of_structs::Synth::new(44100.0),
                        )
                    },
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
criterion_group!(mini_benches, dsp_mini_bench);
criterion_main!(benches, mini_benches);
