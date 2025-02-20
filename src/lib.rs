use claxon::FlacReader;

const WAVEFORM_SECONDS : usize = 40;

pub fn read_flac(path_to_open : &str) -> Vec<f32> {
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
        samples.push(actual_sample as f32 / 2_i32.pow(stream_info.bits_per_sample) as f32);
    }
    samples
}