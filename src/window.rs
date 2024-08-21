use minifb::WindowOptions;
use waves::from_u8_rgb;

const WINDOW_WIDTH: i32 = 1800;
const WINDOW_HEIGHT: i32 = 350;
const GAIN : i32 = 3;

pub const BACKGROUND_GRAY: u32 = from_u8_rgb(30, 30, 30);
pub const LOW_COLOR: u32 = from_u8_rgb(33, 80, 227);
pub const MID_COLOR: u32 = from_u8_rgb(242, 162, 51);
pub const HIGH_COLOR: u32 = from_u8_rgb(245, 234, 214);

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

    pub fn render(&mut self, samples : &Vec<i32>, color : u32) {
        println!("Rendering waveform with color {}", color);
        let stride = samples.len() as i32 / WINDOW_WIDTH;
        for x in 0..WINDOW_WIDTH {
            let upper_bound = if x*stride+stride-1 < samples.len() as i32 {x*stride+stride-1} else {(samples.len()-1) as i32};
            let absolute = (waves::calculate_rms(&samples[(x * stride) as usize..upper_bound as usize]) * GAIN) as f32;
            let fraction = absolute / (i32::MAX as f32);
            let remapped = (fraction * WINDOW_HEIGHT as f32) as i32;
            for y in -remapped..remapped {
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