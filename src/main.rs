mod window;

use std::env;
use claxon::FlacReader;
use spectrum_analyzer::{FrequencyLimit, samples_fft_to_spectrum};
use spectrum_analyzer::scaling::divide_by_N_sqrt;
use spectrum_analyzer::windows::hann_window;
use waves::{FFT_WINDOW_SIZE, separate_spectre_into_bands, WAVEFORM_SECONDS};

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
        let actual_sample = sample.expect("Sample is invalid.") as f32 / i32::MAX as f32;
        samples.push(actual_sample);
    }

    // Convert to frequency domain
    count = 0;
    let mut spectrums = Vec::new();
    while count < WAVEFORM_SECONDS as u32 * stream_info.sample_rate - FFT_WINDOW_SIZE {
        let hann_window = hann_window(&samples[count as usize..(count+FFT_WINDOW_SIZE) as usize]);
        // calc spectrum
        let current_spectrum = samples_fft_to_spectrum(
            // (windowed) samples
            &hann_window,
            stream_info.sample_rate,
            FrequencyLimit::All,
            // optional scale
            Some(&divide_by_N_sqrt),
        ).unwrap();
        spectrums.push(separate_spectre_into_bands(&current_spectrum));
        count += FFT_WINDOW_SIZE;
    }

    // Create a window
    let mut window = window::Window::new();
    window.render(&samples, &spectrums);

    while window.is_window_open() {
        window.update();
    }
}