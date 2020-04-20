use crate::fastmath::{exp2, parabolic_sine, wrap01};

const OSC_MAX_FREQ: f64 = 20480.0;

const BATCH_SIZE: usize = 64;
type BatchData = [f64; BATCH_SIZE];

struct OscillatorHelper {
    pub sample_rate: f64,
    pub input_frequency: f64,
    pub octave_offset: f64,
    pub semitone_offset: f64,
    pub cent_offset: f64,
    pub amplitude: f64,

    pub input_frequency_mod_ratio: BatchData,
    pub phase_mod: BatchData,
    pub frequency_mod: BatchData,

    pub modulo: BatchData,
    pub wrap_modulo: [bool; BATCH_SIZE],
    pub amplitude_mod: BatchData,

    last_modulo: f64,
}

impl OscillatorHelper {
    fn new(sample_rate: f64) -> Self {
        OscillatorHelper {
            sample_rate,
            input_frequency: 0.0,
            octave_offset: 0.0,
            semitone_offset: 0.0,
            cent_offset: 0.0,
            amplitude: 1.0,
            input_frequency_mod_ratio: [1.0; BATCH_SIZE],
            frequency_mod: [0.0; BATCH_SIZE],
            phase_mod: [0.0; BATCH_SIZE],
            amplitude_mod: [1.0; BATCH_SIZE],
            modulo: [0.0; BATCH_SIZE],
            wrap_modulo: [false; BATCH_SIZE],
            last_modulo: 0.0,
        }
    }

    fn update(&mut self) {
        let const_offset =
            self.octave_offset * 12.0 + self.semitone_offset + self.cent_offset / 100.0;

        for (((out_modulo, out_wrap_modulo), input_frequency_mod_ratio), frequency_mod) in self
            .modulo
            .iter_mut()
            .zip(self.wrap_modulo.iter_mut())
            .zip(self.input_frequency_mod_ratio.iter())
            .zip(self.frequency_mod.iter())
        {
            let frequency = (self.input_frequency
                * input_frequency_mod_ratio
                * exp2(frequency_mod + const_offset))
            .max(-OSC_MAX_FREQ)
            .min(OSC_MAX_FREQ);

            let phase_incr = frequency / self.sample_rate;

            let wrap = if phase_incr > 0.0 && self.last_modulo >= 1.0 {
                self.last_modulo -= 1.0;
                true
            } else if phase_incr < 0.0 && self.last_modulo <= 0.0 {
                self.last_modulo += 1.0;
                true
            } else {
                false
            };

            *out_modulo = self.last_modulo;
            *out_wrap_modulo = wrap;

            self.last_modulo += phase_incr;
        }
    }
}

struct BandLimitedOscillator {
    helper: OscillatorHelper,
    output: BatchData,
}

impl BandLimitedOscillator {
    fn new(sample_rate: f64) -> Self {
        BandLimitedOscillator {
            helper: OscillatorHelper::new(sample_rate),
            output: [0.0; BATCH_SIZE],
        }
    }

    fn render(&mut self) {
        self.helper.update();

        for (((output, modulo), phase_mod), amplitude_mod) in self
            .output
            .iter_mut()
            .zip(self.helper.modulo.iter())
            .zip(self.helper.phase_mod.iter())
            .zip(self.helper.amplitude_mod.iter())
        {
            let modulo = wrap01(modulo + phase_mod);
            let angle = modulo * 2.0 * std::f64::consts::PI - std::f64::consts::PI;
            *output = parabolic_sine(-angle) * self.helper.amplitude * amplitude_mod;
        }
    }
}

struct LFO {
    helper: OscillatorHelper,
    output: BatchData,
    quad_output: BatchData,
}

impl LFO {
    fn new(sample_rate: f64) -> Self {
        LFO {
            helper: OscillatorHelper::new(sample_rate),
            output: [0.0; BATCH_SIZE],
            quad_output: [0.0; BATCH_SIZE],
        }
    }

    fn render(&mut self) {
        self.helper.update();

        for (((output, quad_output), modulo), amplitude_mod) in self
            .output
            .iter_mut()
            .zip(self.quad_output.iter_mut())
            .zip(self.helper.modulo.iter())
            .zip(self.helper.amplitude_mod.iter())
        {
            let angle = modulo * 2.0 * std::f64::consts::PI - std::f64::consts::PI;
            *output = parabolic_sine(-angle) * self.helper.amplitude * amplitude_mod;

            let quad_angle = (modulo + 0.25) * 2.0 * std::f64::consts::PI - std::f64::consts::PI;
            *quad_output = parabolic_sine(-quad_angle) * self.helper.amplitude * amplitude_mod;
        }
    }
}

pub struct Synth {
    osc1: BandLimitedOscillator,
    osc2: BandLimitedOscillator,
    lfo: LFO,
}

impl Synth {
    pub fn new(sample_rate: f64) -> Self {
        let mut synth = Synth {
            osc1: BandLimitedOscillator::new(sample_rate),
            osc2: BandLimitedOscillator::new(sample_rate),
            lfo: LFO::new(sample_rate),
        };

        synth.osc1.helper.input_frequency = 440.0;
        synth.osc2.helper.input_frequency = 440.0;

        synth.osc2.helper.cent_offset = 2.5;

        synth.lfo.helper.input_frequency = 0.5;

        synth
    }

    pub fn render(&mut self, buffer: &mut [f64]) {
        for output_batch in buffer.chunks_exact_mut(BATCH_SIZE) {
            self.lfo.render();

            self.osc1.helper.frequency_mod = self.lfo.output;
            self.osc2.helper.frequency_mod = self.lfo.output;

            self.osc1.render();
            self.osc2.render();

            for ((output, osc1_out), osc2_out) in output_batch
                .iter_mut()
                .zip(self.osc1.output.iter())
                .zip(self.osc2.output.iter())
            {
                *output = 0.5 * osc1_out + 0.5 * osc2_out;
            }
        }
    }
}
