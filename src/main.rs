use std::env;

use claxon::FlacReader;
use minifb::{Window, WindowOptions};

const GAIN : i32 = 1;
const WAVEFORM_SECONDS : usize = 60;
const WINDOW_WIDTH: i32 = 800;
const WINDOW_HEIGHT: i32 = 350;
const BACKGROUND_GRAY: u32 = from_u8_rgb(30, 30, 30);

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
    let mut frame_buffer: Vec<u32> = vec![BACKGROUND_GRAY; (WINDOW_HEIGHT * WINDOW_WIDTH) as usize];
    render(&mut frame_buffer, &samples);

    let mut window = Window::new("Waves", WINDOW_WIDTH as usize, WINDOW_HEIGHT as usize, WindowOptions {
        resize: false,
        ..WindowOptions::default()
    }).unwrap();

    window.update_with_buffer(&frame_buffer, WINDOW_WIDTH as usize, WINDOW_HEIGHT as usize).unwrap();
    loop {
        window.update_with_buffer(&frame_buffer, WINDOW_WIDTH as usize, WINDOW_HEIGHT as usize).unwrap();
    }
}

const fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}

fn calculate_rms(samples : &[i32]) -> i32 {
    let mut sqr_sum:i64 = 0;
    for sample in samples {
        sqr_sum += (*sample * *sample) as i64;
    }
    (sqr_sum / samples.len() as i64) as i32
}

fn render(frame_buffer : &mut Vec<u32>, samples : &Vec<i32>) {
    let stride = samples.len() as i32 / WINDOW_WIDTH;
    for x in 0..WINDOW_WIDTH {
        let upper_bound = if x*stride+stride-1 < samples.len() as i32 {x*stride+stride-1} else {(samples.len()-1) as i32};
        let absolute = (calculate_rms(&samples[(x * stride) as usize..upper_bound as usize]) * GAIN) as f32;
        let fraction = absolute / (i32::MAX as f32);
        let remapped = (fraction * WINDOW_HEIGHT as f32) as i32;
        for y in -remapped..remapped {
            frame_buffer[((y + WINDOW_HEIGHT / 2) * WINDOW_WIDTH + x) as usize] = from_u8_rgb(255, 255, 255);
        }
    }
}