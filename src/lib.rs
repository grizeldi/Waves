use spectrum_analyzer::FrequencySpectrum;

pub const GAIN : i32 = 2;
pub const WAVEFORM_SECONDS : usize = 60;
pub const FFT_WINDOW_SIZE : u32 = 2048;

// Band limits
const LOW_MID_THRESHOLD: f32 = 400.0;
const MID_HIGH_THRESHOLD: f32 = 3900.0;

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

pub fn calculate_float_rms(samples : &[f32]) -> f32 {
    let mut sqr_sum:i64 = 0;
    for sample in samples {
        let var = (*sample * i32::MAX as f32) as i32;
        sqr_sum += (var * var) as i64;
    }
    (sqr_sum / samples.len() as i64) as f32 / i32::MAX as f32
}

/// Calculates percentages of low/mid/high bands in a give spectrum
pub fn separate_spectre_into_bands(spectrum : &FrequencySpectrum) -> (f32, f32, f32) {
    let mut low = 0.0;
    let mut mid = 0.0;
    let mut high = 0.0;

    for (frequency, value) in spectrum.data() {
        if frequency.val() < LOW_MID_THRESHOLD {
            low += value.val();
        } else if frequency.val() > MID_HIGH_THRESHOLD {
            high += value.val();
        } else {
            mid += value.val();
        }
    }

    let total = low + mid + high;
    (low / total, mid / total, high / total)
}