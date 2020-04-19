const OSC_MAX_FREQ: f64 = 20480.0;

struct OscillatorHelper {
    pub sample_rate: f64,
    pub input_frequency: f64,
    pub octave_offset: f64,
    pub semitone_offset: f64,
    pub cent_offset: f64,
    pub amplitude: f64,

    pub input_frequency_mod_ratio: f64,
    pub phase_mod: f64,
    pub amplitude_mod: f64,
    pub frequency_mod: f64,

    computed_frequency: f64,
    phase_increment: f64,
    modulo: f64,
}

impl OscillatorHelper {
    fn new(sample_rate: f64) -> Self {
        OscillatorHelper {
            sample_rate,
            input_frequency: 0.0,
            input_frequency_mod_ratio: 1.0,
            octave_offset: 0.0,
            semitone_offset: 0.0,
            cent_offset: 0.0,
            frequency_mod: 0.0,
            amplitude: 1.0,
            amplitude_mod: 1.0,
            phase_mod: 0.0,
            computed_frequency: 0.0,
            phase_increment: 0.0,
            modulo: 0.0,
        }
    }

    // #[inline(never)]
    fn update(&mut self) {
        self.computed_frequency = self.input_frequency
            * self.input_frequency_mod_ratio
            * pitch_shift_multiplier(
                self.frequency_mod
                    + self.octave_offset * 12.0
                    + self.semitone_offset
                    + self.cent_offset / 100.0,
            );

        if self.computed_frequency > OSC_MAX_FREQ {
            self.computed_frequency = OSC_MAX_FREQ
        } else if self.computed_frequency < -OSC_MAX_FREQ {
            self.computed_frequency = -OSC_MAX_FREQ;
        }

        self.phase_increment = self.computed_frequency / self.sample_rate;
    }

    fn check_wrap_modulo(&mut self) -> bool {
        if self.phase_increment > 0.0 && self.modulo >= 1.0 {
            self.modulo -= 1.0;
            true
        } else if self.phase_increment < 0.0 && self.modulo <= 0.0 {
            self.modulo += 1.0;
            true
        } else {
            false
        }
    }

    fn increment_modulo(&mut self) {
        self.modulo += self.phase_increment;
    }
}

struct BandLimitedOscillator {
    helper: OscillatorHelper,
}

impl BandLimitedOscillator {
    fn new(sample_rate: f64) -> Self {
        BandLimitedOscillator {
            helper: OscillatorHelper::new(sample_rate),
        }
    }

    // #[inline(never)]
    fn update(&mut self) {
        self.helper.update();
    }

    // #[inline(never)]
    fn render(&mut self) -> f64 {
        self.helper.check_wrap_modulo();

        let modulo = wrap_modulo(self.helper.modulo + self.helper.phase_mod);

        let out: f64;

        {
            let angle = modulo * 2.0 * std::f64::consts::PI - std::f64::consts::PI;
            out = parabolic_sine(-angle);
        }

        self.helper.increment_modulo();

        out * self.helper.amplitude * self.helper.amplitude_mod
    }
}

struct LFO {
    helper: OscillatorHelper,
}

impl LFO {
    fn new(sample_rate: f64) -> Self {
        LFO {
            helper: OscillatorHelper::new(sample_rate),
        }
    }

    // #[inline(never)]
    fn update(&mut self) {
        self.helper.update();
    }

    // #[inline(never)]
    fn render(&mut self) -> (f64, f64) {
        self.helper.check_wrap_modulo();

        let quad_modulo = {
            let quad_modulo = self.helper.modulo + 0.25;
            if quad_modulo >= 1.0 {
                quad_modulo - 1.0
            } else {
                quad_modulo
            }
        };

        let out: f64;
        let quad_out: f64;

        {
            let angle = self.helper.modulo * 2.0 * std::f64::consts::PI - std::f64::consts::PI;
            out = parabolic_sine(-angle);

            let quad_angle = quad_modulo * 2.0 * std::f64::consts::PI - std::f64::consts::PI;
            quad_out = parabolic_sine(-quad_angle);
        }

        self.helper.increment_modulo();

        (out, quad_out)
    }
}

fn pitch_shift_multiplier(x: f64) -> f64 {
    2.0f64.powf(x / 12.0)
}

fn wrap_modulo(x: f64) -> f64 {
    x.rem_euclid(1.0)
}

fn parabolic_sine(x: f64) -> f64 {
    use std::f64::consts::PI;

    const B: f64 = 4.0 / PI;
    const C: f64 = -4.0 / (PI * PI);
    const P: f64 = 0.225;
    let mut y = B * x + C * x * x.abs();

    y = P * (y * y.abs() - y) + y;

    y
}

pub struct Synth {
    osc1: BandLimitedOscillator,
    // osc2: BandLimitedOscillator,
    // lfo: LFO,
}

impl Synth {
    pub fn new(sample_rate: f64) -> Self {
        let mut synth = Synth {
            osc1: BandLimitedOscillator::new(sample_rate),
            // osc2: BandLimitedOscillator::new(sample_rate),
            // lfo: LFO::new(sample_rate),
        };

        synth.osc1.helper.input_frequency = 440.0;
        // synth.osc2.helper.input_frequency = 440.0;
        
        // synth.osc2.helper.cent_offset = 2.5;

        // synth.lfo.helper.input_frequency = 0.5;

        synth
    }

    #[inline(never)]
    pub fn render(&mut self, buffer: &mut [f64]) {        
        for output in buffer {
            // self.lfo.update();
            // let (lfo_out, _) = self.lfo.render();

            // self.osc1.helper.frequency_mod = lfo_out;
            self.osc1.update();

            // self.osc2.helper.frequency_mod = lfo_out;
            // self.osc2.update();

            let osc1_out = self.osc1.render();
            // let osc2_out = self.osc2.render();
            // *output = 0.5 * osc1_out + 0.5 * osc2_out;
            *output = 0.5 * osc1_out;
        }
    }
}
