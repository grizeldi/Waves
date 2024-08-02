use minifb::WindowOptions;
use waves::{FFT_WINDOW_SIZE, from_u8_rgb, GAIN};

const WINDOW_WIDTH: i32 = 1000;
const WINDOW_HEIGHT: i32 = 350;

// Colors
const BACKGROUND_GRAY: u32 = from_u8_rgb(30, 30, 30);
const LOW_COLOR: u32 = from_u8_rgb(33, 80, 227);
const MID_COLOR: u32 = from_u8_rgb(242, 162, 51);
const HIGH_COLOR: u32 = from_u8_rgb(245, 234, 214);

pub struct Window {
    minifb_window: minifb::Window,
    frame_buffer: Vec<u32>
}

impl Window {
    pub fn new() -> Window {
        let mut window = minifb::Window::new("Waves", WINDOW_WIDTH as usize, WINDOW_HEIGHT as usize, WindowOptions {
            resize: false,
            ..WindowOptions::default()
        }).unwrap();
        window.set_target_fps(30);

        Window {
            minifb_window: window,
            frame_buffer: vec![BACKGROUND_GRAY; (WINDOW_HEIGHT * WINDOW_WIDTH) as usize],
        }
    }

    pub fn render(&mut self, samples : &Vec<f32>, spectra : &Vec<(f32, f32, f32)>) {
        for x in 0..WINDOW_WIDTH {
            let upper_bound = if x as u32 * FFT_WINDOW_SIZE + FFT_WINDOW_SIZE-1 < samples.len() as u32 {x as u32 *FFT_WINDOW_SIZE+FFT_WINDOW_SIZE-1} else { (samples.len() - 1) as u32 };
            let rms = waves::calculate_float_rms(&samples[(x * FFT_WINDOW_SIZE as i32) as usize..upper_bound as usize]) * GAIN as f32;
            let remapped = (rms * WINDOW_HEIGHT as f32) as i32;
            for y in -remapped..remapped {
                let color : u32 = {
                    if y.abs() < (spectra[x as usize].0 * remapped as f32) as i32 { LOW_COLOR }
                    else if y.abs() > ((1.0 - spectra[x as usize].2) * remapped as f32) as i32 { HIGH_COLOR }
                    else { MID_COLOR }
                };
                self.frame_buffer[((y + WINDOW_HEIGHT / 2) * WINDOW_WIDTH + x) as usize] = color;
            }
        }
    }

    pub fn update(&mut self) {
        self.minifb_window.update_with_buffer(&self.frame_buffer, WINDOW_WIDTH as usize, WINDOW_HEIGHT as usize).unwrap();
    }

    pub fn is_window_open(&self) -> bool {
        self.minifb_window.is_open()
    }
}