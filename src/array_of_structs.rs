use crate::fastmath::{parabolic_sine, wrap01, exp2};

const OSC_MAX_FREQ: f64 = 20480.0;

const BATCH_SIZE: usize = 64;

#[derive(Copy, Clone)]
struct OscillatorAudioRate {
    pub input_frequency_mod_ratio: f64,
    pub phase_mod: f64,
    pub frequency_mod: f64,
    pub modulo: f64,

    pub wrap_modulo: bool,
    pub amplitude_mod: f64,
}

struct OscillatorHelper {
    pub sample_rate: f64,
    pub input_frequency: f64,
    pub octave_offset: f64,
    pub semitone_offset: f64,
    pub cent_offset: f64,
    pub amplitude: f64,

    pub audio_rate: [OscillatorAudioRate; BATCH_SIZE],

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
            audio_rate: [OscillatorAudioRate {
                input_frequency_mod_ratio: 1.0,
                frequency_mod: 0.0,
                phase_mod: 0.0,
                amplitude_mod: 1.0,
                modulo: 0.0,
                wrap_modulo: false,
            }; BATCH_SIZE],
            last_modulo: 0.0,
        }
    }

    fn set_frequency_mod<I: Iterator<Item = f64>>(&mut self, iter: I) {
        self.audio_rate
            .iter_mut()
            .map(|d| &mut d.frequency_mod)
            .zip(iter)
            .for_each(|(dest, src)| *dest = src);
    }

    #[inline(never)]
    fn update(&mut self) {
        let const_offset =
            self.octave_offset * 12.0 + self.semitone_offset + self.cent_offset / 100.0;

        for audio_rate in self.audio_rate.iter_mut() {
            let frequency = (self.input_frequency
                * audio_rate.input_frequency_mod_ratio
                * exp2(audio_rate.frequency_mod + const_offset))
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

            audio_rate.modulo = self.last_modulo;
            audio_rate.wrap_modulo = wrap;

            self.last_modulo += phase_incr;
        }
    }
}

struct BandLimitedOscillator {
    helper: OscillatorHelper,
    output: [f64; BATCH_SIZE],
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

        for (output, audio_rate) in self.output.iter_mut().zip(self.helper.audio_rate.iter()) {
            let modulo = wrap01(audio_rate.modulo + audio_rate.phase_mod);
            let angle = modulo * 2.0 * std::f64::consts::PI - std::f64::consts::PI;
            *output = parabolic_sine(-angle) * self.helper.amplitude * audio_rate.amplitude_mod;
        }
    }
}

struct LFO {
    helper: OscillatorHelper,
    output: [(f64, f64); BATCH_SIZE],
}

impl LFO {
    fn new(sample_rate: f64) -> Self {
        LFO {
            helper: OscillatorHelper::new(sample_rate),
            output: [(0.0, 0.0); BATCH_SIZE],
        }
    }

    fn render(&mut self) {
        self.helper.update();

        for (output, audio_rate) in self.output.iter_mut().zip(self.helper.audio_rate.iter()) {
            let angle = audio_rate.modulo * 2.0 * std::f64::consts::PI - std::f64::consts::PI;
            output.0 = parabolic_sine(-angle) * self.helper.amplitude * audio_rate.amplitude_mod;

            let quad_angle =
                (audio_rate.modulo + 0.25) * 2.0 * std::f64::consts::PI - std::f64::consts::PI;
            output.1 =
                parabolic_sine(-quad_angle) * self.helper.amplitude * audio_rate.amplitude_mod;
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

            self.osc1
                .helper
                .set_frequency_mod(self.lfo.output.iter().map(|(out, _)| *out));
            self.osc2
                .helper
                .set_frequency_mod(self.lfo.output.iter().map(|(out, _)| *out));

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
