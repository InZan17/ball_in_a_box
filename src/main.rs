use macroquad::{prelude::*, time};
use miniquad::*;

pub fn window_conf() -> miniquad::conf::Conf {
    miniquad::conf::Conf {
        window_title: "Ball in a Box".to_string(),
        window_width: 800,
        window_height: 600,
        high_dpi: true,
        fullscreen: false,
        sample_count: 4,
        window_resizable: false,
        ..Default::default()
    }
}

pub trait FromTuple {
    fn from_u32_tuple(tuple: (u32, u32)) -> Self;
}

impl FromTuple for Vec2 {
    fn from_u32_tuple(tuple: (u32, u32)) -> Self {
        Vec2::new(tuple.0 as f32, tuple.1 as f32)
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut last_window_position = Vec2::from_u32_tuple(miniquad::window::get_window_position());

    let mut ball_position = Vec2::new(0., 0.);

    loop {
        let current_window_position = Vec2::from_u32_tuple(miniquad::window::get_window_position());
        let delta_position = last_window_position - current_window_position;
        last_window_position = current_window_position;

        println!("{:?}", delta_position);
        clear_background(LIGHTGRAY);

        let ball_velocity = delta_position;

        if time::get_time() > 1. {
            ball_position += ball_velocity * 2.;
        }

        set_camera(&Camera2D {
            zoom: vec2(1. / 800., 1. / 600.),
            ..Default::default()
        });
        draw_circle(ball_position.x, ball_position.y, 40., YELLOW);

        next_frame().await
    }
}
