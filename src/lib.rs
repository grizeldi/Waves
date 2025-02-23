use std::process::Command;
use std::thread;
use claxon::FlacReader;
use log::{debug, info};

const WAVEFORM_SECONDS : usize = 40;
pub const FILENAME_LOW_BAND : &str = "/tmp/waves_low.flac";
pub const FILENAME_MID_BAND : &str = "/tmp/waves_mid.flac";
pub const FILENAME_HIGH_BAND : &str = "/tmp/waves_high.flac";

pub fn read_flac(path_to_open : &str) -> Vec<f32> {
    debug!("Reading {} seconds of FLAC at path \"{}\".", WAVEFORM_SECONDS, path_to_open);
    let mut flac_reader = FlacReader::open(path_to_open).expect("Failed to open FLAC stream.");
    let stream_info = flac_reader.streaminfo();

    let mut count = 0;
    let mut samples = Vec::with_capacity(stream_info.sample_rate as usize * WAVEFORM_SECONDS);

    // Read the file contents
    for sample in flac_reader.samples() {
        if count >= stream_info.sample_rate * WAVEFORM_SECONDS as u32 {
            break;
        }

        count += 1;
        let actual_sample = sample.expect("Sample is invalid.");
        samples.push(actual_sample as f32 / 2_i32.pow(stream_info.bits_per_sample - 1) as f32);
    }
    debug!("Done reading FLAC at path \"{}\".", path_to_open);
    samples
}

pub fn separate_audio_file_into_bands(filename : &str) {
    // Generate the filtered versions
    let file = (*filename).to_string();
    let low_band_thread = thread::spawn(move || {
        info!("Generating low band audio file for {}.", file);
        Command::new("ffmpeg")
            .arg("-y")
            .arg("-i")
            .arg(file)
            .arg("-af")
            .arg("lowpass=f=100")
            .arg(FILENAME_LOW_BAND)
            .output()
            .expect("Failed to run ffmpeg");
    });

    let file = (*filename).to_string();
    let mid_band_thread = thread::spawn(move || {
        info!("Generating mid band audio file for {}.", file);
        Command::new("ffmpeg")
            .arg("-y")
            .arg("-i")
            .arg(file)
            .arg("-af")
            .arg("highpass=f=5000")
            .arg(FILENAME_HIGH_BAND)
            .output()
            .expect("Failed to run ffmpeg");
    });

    info!("Generating high band audio file for {}.", filename);
    Command::new("ffmpeg")
        .arg("-y")
        .arg("-i")
        .arg(filename)
        .arg("-af")
        .arg("bandpass=f=1750")//:width=1000:width_type=h")
        .arg(FILENAME_MID_BAND)
        .output()
        .expect("Failed to run ffmpeg");

    low_band_thread.join().unwrap();
    mid_band_thread.join().unwrap();
    info!("All band audio files successfully generated.");
}