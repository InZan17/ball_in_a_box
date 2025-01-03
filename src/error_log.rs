use std::{fs::OpenOptions, io::Write};

use circular_buffer::CircularBuffer;
use macroquad::{
    color::Color,
    math::{Rect, Vec2},
    shapes::draw_rectangle,
    text::{draw_text_ex, TextParams},
    time::get_time,
};

const ERROR_HEIGHT: f32 = 120.0;
const ERROR_PADDING: f32 = 10.0;
const ERROR_MAX_COUNT: usize = 10;

const ERROR_ALPHA: f32 = 0.8;
const ERROR_FONT_SIZE: u16 = 22;
const ERROR_FONT_SIZE_F32: f32 = ERROR_FONT_SIZE as f32;

const ERROR_START_DECAY: f64 = 3.0;
const ERROR_DECAY_DURATION: f64 = 2.0;

pub struct ErrorLogs(CircularBuffer<ERROR_MAX_COUNT, (f64, String)>);

impl ErrorLogs {
    pub fn new() -> Self {
        Self(CircularBuffer::new())
    }
    /// Adds a error to the log file aswell as displaying it inside the game.
    pub fn display_error(&mut self, error: String) {
        self.add_error(&error);
        let time = get_time();
        self.0.push_front((time, error));
    }
    /// Only adds a error to the log file.
    pub fn add_error(&self, error: &str) {
        if let Ok(mut log_file) = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open("error_log.txt")
        {
            let mut bytes = error.as_bytes().to_vec();
            bytes.extend("\n".as_bytes());
            let _ = log_file.write(&bytes);
        };
    }
    /// Renders the errors to the screen.
    pub fn render_errors(&self, top_left_corner: Vec2, width: f32) {
        let time = get_time();
        let start_decay_time = time - ERROR_START_DECAY;
        for (i, (error_time, error)) in self.0.iter().enumerate() {
            let decay_value = (start_decay_time - *error_time).max(0.0) / ERROR_DECAY_DURATION;
            let alpha = (1.0 - decay_value).max(0.0);

            let rect = Rect::new(
                top_left_corner.x,
                top_left_corner.y + ERROR_PADDING + i as f32 * (ERROR_HEIGHT + ERROR_PADDING),
                width,
                ERROR_HEIGHT,
            );

            draw_rectangle(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                Color::new(0.2, 0.0, 0.0, alpha as f32 * ERROR_ALPHA),
            );
            draw_text_ex(
                error,
                rect.x + 10.0,
                rect.y + (rect.h + ERROR_FONT_SIZE_F32) / 2.,
                TextParams {
                    font: None,
                    font_size: 22,
                    font_scale: 2.,
                    color: Color::new(1.0, 1.0, 1.0, alpha as f32),
                    ..Default::default()
                },
            );
        }
    }
}
