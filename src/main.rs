use macroquad::{prelude::*, time};
use miniquad::*;
use window::set_window_position;

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
    let mut delta_window_position = Vec2::ZERO;

    let gravity_strength = 1000.;
    let bounciness = 0.9;
    let mut ball_position = Vec2::ZERO;
    let mut ball_velocity = Vec2::ZERO;
    let terminal_velocity = 8000.;


    loop {
        if is_mouse_button_down(MouseButton::Left) {
            delta_window_position = mouse_delta_position() * Vec2::new(400., 300.) + delta_window_position;
            last_window_position -= delta_window_position;
            set_window_position(last_window_position.x as u32, last_window_position.y as u32);
        } else {
            let current_window_position = Vec2::from_u32_tuple(miniquad::window::get_window_position());
            delta_window_position = last_window_position - current_window_position;
            last_window_position = current_window_position;
        }

        clear_background(LIGHTGRAY);

        ball_velocity += Vec2::new(0., gravity_strength * get_frame_time());

        let total_velocity = if time::get_time() > 1. {
            ball_velocity + (delta_window_position / get_frame_time()) * 2.
        } else {
            ball_velocity
        };

        ball_position += total_velocity * get_frame_time();

        set_camera(&Camera2D {
            zoom: vec2(1. / 800., 1. / 600.),
            ..Default::default()
        });

        if ball_position.y > 600. {
            ball_position.y = 600.;
            ball_velocity.y = -total_velocity.y * bounciness;
        } else if ball_position.y < -600. {
            ball_position.y = -600.;
            ball_velocity.y = -total_velocity.y * bounciness;
        }


        if ball_velocity.length() > terminal_velocity {
            ball_velocity = ball_velocity.normalize() * terminal_velocity;
        }

        draw_circle(ball_position.x, ball_position.y, 40., BLUE);

        next_frame().await
    }
}
