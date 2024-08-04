mod window;

use std::env;
use std::str;
use std::process::Command;
use waves::read_flac;

const FILENAME_LOW_BAND : &str = "/tmp/waves_low.flac";
const FILENAME_MID_BAND : &str = "/tmp/waves_mid.flac";
const FILENAME_HIGH_BAND : &str = "/tmp/waves_high.flac";

fn main() {
    // Find file
    let args : Vec<String> = env::args().collect();
    let path_to_open = &args[1];
    separate_bands(path_to_open);

    // Open the file
    let lows = read_flac(FILENAME_LOW_BAND);
    let mids = read_flac(FILENAME_MID_BAND);
    let highs = read_flac(FILENAME_HIGH_BAND);

    // // Create frequency spectrum
    // let mut planner = FftPlanner::new();
    // let mut buffer = samples.iter().map(|x| Complex::new(*x as f32, 0f32)).collect::<Vec<Complex<f32>>>();
    // let fft = planner.plan_fft_forward(6);
    // fft.process(&mut buffer);
    //
    // dbg!(buffer);

    // Create a window
    let mut window = window::Window::new();
    window.render(&lows, window::LOW_COLOR);
    window.render(&mids, window::MID_COLOR);
    window.render(&highs, window::HIGH_COLOR);

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
        .arg("highpass=f=5000")
        .arg(FILENAME_HIGH_BAND)
        .output()
        .expect("Failed to run ffmpeg");

    Command::new("ffmpeg")
        .arg("-i")
        .arg(filename)
        .arg("-af")
        .arg("bandpass=f=1750")//:width=1000:width_type=h")
        .arg(FILENAME_MID_BAND)
        .output()
        .expect("Failed to run ffmpeg");
}