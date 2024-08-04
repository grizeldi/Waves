mod window;

use std::env;
use std::str;
use std::process::Command;
use claxon::FlacReader;

const GAIN : i32 = 1;
const WAVEFORM_SECONDS : usize = 60;
const FILENAME_LOW_BAND : &str = "/tmp/waves_low.flac";
const FILENAME_MID_BAND : &str = "/tmp/waves_mid.flac";
const FILENAME_HIGH_BAND : &str = "/tmp/waves_high.flac";

fn main() {
    // Find file
    let args : Vec<String> = env::args().collect();
    let path_to_open = &args[1];
    separate_bands(path_to_open);

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

    while window.is_window_open() {
        window.update();
    }
}

fn separate_bands(filename : &String) {
    // Generate the filtered versions
    /*let output = */Command::new("ffmpeg")
        .arg("-i")
        .arg(filename)
        .arg("-af")
        .arg("lowpass=f=100")
        .arg(FILENAME_LOW_BAND)
        .output()
        .expect("Failed to run ffmpeg");
    // println!("{}", str::from_utf8(output.stdout.as_slice()).expect("Failed to format output"));
    // println!("{}", str::from_utf8(output.stderr.as_slice()).expect("Failed to format stderr"));

    Command::new("ffmpeg")
        .arg("-i")
        .arg(filename)
        .arg("-af")
        .arg("highpass=f=10000")
        .arg(FILENAME_HIGH_BAND)
        .output()
        .expect("Failed to run ffmpeg");

    Command::new("ffmpeg")
        .arg("-i")
        .arg(filename)
        .arg("-af")
        .arg("bandpass=f=1000")
        .arg(FILENAME_MID_BAND)
        .output()
        .expect("Failed to run ffmpeg");
}