pub fn parabolic_sine(x: f64) -> f64 {
    use std::f64::consts::PI;

    const B: f64 = 4.0 / PI;
    const C: f64 = -4.0 / (PI * PI);
    const P: f64 = 0.225;
    let mut y = B * x + C * x * x.abs();

    y = P * (y * y.abs() - y) + y;

    y
}

pub fn wrap01(x: f64) -> f64 {
    if x >= 0.0 && x <= 1.0 {
        x
    } else {
        x.rem_euclid(1.0)
    }
}

// https://github.com/akohlmey/fastermath/blob/master/src/exp.c
pub fn exp2(mut x: f64) -> f64 {
    #[repr(align(32))]
    struct Aligned<T>(T);

    const Q: Aligned<[f64; 2]> = Aligned([2.33184211722314911771e2, 4.36821166879210612817e3]);
    const P: Aligned<[f64; 3]> = Aligned([
        2.30933477057345225087e-2,
        2.02020656693165307700e1,
        1.51390680115615096133e3,
    ]);

    union DoubleManip {
        f: f64,
        s: [i32; 2],
    }

    const DOUBLE_BIAS: i32 = 1023;

    let ipart = (x + 0.5).floor();
    let fpart = x - ipart;

    let epart = DoubleManip {
        s: [0, ((ipart as i32) + DOUBLE_BIAS) << 20],
    };

    x = fpart * fpart;

    let mut px = P.0[0];
    px = px * x + P.0[1];
    let mut qx = x + Q.0[0];
    px = px * x + P.0[2];
    qx = qx * x + Q.0[1];

    px = px * fpart;

    x = 1.0 + 2.0 * (px / (qx - px));

    unsafe { epart.f * x }
}
