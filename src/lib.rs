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