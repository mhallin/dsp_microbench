// use std::error::Error;

use criterion::black_box;

// use plotters::style::{IntoFont, RED, BLUE, WHITE};
// use plotters::{
//     chart::ChartBuilder,
//     drawing::{BitMapBackend, IntoDrawingArea},
//     series::LineSeries,
// };

// fn main() -> Result<(), Box<dyn Error>> {
//     let root = BitMapBackend::new("target/render_result.png", (640, 480)).into_drawing_area();
//     root.fill(&WHITE)?;

//     let mut chart = ChartBuilder::on(&root)
//         .caption("Benchmark", ("sans-serif", 50).into_font())
//         .margin(5)
//         .x_label_area_size(30)
//         .y_label_area_size(30)
//         .build_ranged(0.0f64..128.0, -1.2f64..1.2)?;

//     chart.configure_mesh().draw()?;

//     {
//         let mut data = vec![0.0f64; 128];
//         let mut synth = dsp_perf::one_frame_per_call::Synth::new(44100.0);
//         synth.render(&mut data);
//         chart
//             .draw_series(LineSeries::new(
//                 data.into_iter().enumerate().map(|(i, y)| (i as f64, y)),
//                 &RED,
//             ))?
//             .label("One frame per call");
//     }

//     {
//         let mut data = vec![0.0f64; 128];
//         let mut synth = dsp_perf::fixed_batch_size::Synth::new(44100.0);
//         synth.render(&mut data);
//         chart
//             .draw_series(LineSeries::new(
//                 data.into_iter().enumerate().map(|(i, y)| (i as f64, y)),
//                 &BLUE,
//             ))?
//             .label("One frame per call");
//     }

//     Ok(())
// }

fn main() {
    let mut data = vec![0.0f64; 4096];
    let mut synth = dsp_perf::one_frame_per_call::Synth::new(44100.0);
    // let mut synth = dsp_perf::fixed_batch_size::Synth::new(44100.0);
    for _ in 0..100000 {
        synth.render(&mut data);
    }
    black_box(data);
}
