use claxon::FlacReader;

const WAVEFORM_SECONDS : usize = 20;

pub const fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}

pub fn calculate_rms(samples : &[i32]) -> i32 {
    let mut sqr_sum:i64 = 0;
    for sample in samples {
        sqr_sum += (*sample * *sample) as i64;
    }
    (sqr_sum / samples.len() as i64) as i32
}

pub fn read_flac(path_to_open : &str) -> Vec<i32> {
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
        samples.push(actual_sample);
    }
    samples
}