mod window;

use std::env;
use claxon::FlacReader;

const GAIN : i32 = 1;
const WAVEFORM_SECONDS : usize = 60;

fn main() {
    // Find file
    let args : Vec<String> = env::args().collect();
    let path_to_open = &args[1];

    // Open the file
    let mut flac_reader = FlacReader::open(path_to_open).expect("Failed to open FLAC stream.");
    let stream_info = flac_reader.streaminfo();
    // dbg!(&stream_info);

    let mut count = 0;
    let mut samples = Vec::with_capacity(stream_info.sample_rate as usize * WAVEFORM_SECONDS);

    // Read the file contents
    for sample in flac_reader.samples() {
        if count >= stream_info.sample_rate * WAVEFORM_SECONDS as u32 {
            break;
        }

        count += 1;
        let actual_sample = sample.expect("Sample is invalid.");
        samples.push(actual_sample);
    }
    // dbg!(&samples);

    // // Create frequency spectrum
    // let mut planner = FftPlanner::new();
    // let mut buffer = samples.iter().map(|x| Complex::new(*x as f32, 0f32)).collect::<Vec<Complex<f32>>>();
    // let fft = planner.plan_fft_forward(6);
    // fft.process(&mut buffer);
    //
    // dbg!(buffer);

    // Create a window
    let mut window = window::Window::new();
    window.render(&samples);

    loop {
        window.update();
    }
}