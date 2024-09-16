use macroquad::{prelude::*, time};
use miniquad::*;
use window::set_window_position;

const WIDTH: i32 = 640;
const HEIGHT: i32 = 480;

const WIDTH_F: f32 = WIDTH as f32;
const HEIGHT_F: f32 = HEIGHT as f32;

pub fn window_conf() -> miniquad::conf::Conf {
    miniquad::conf::Conf {
        window_title: "Ball in a Box".to_string(),
        window_width: WIDTH,
        window_height: HEIGHT,
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

    let mut smoothed_delta = Vec2::ZERO;
    let mut smoothed_magnitude = 0.;

    let gravity_strength = 3000.;
    let air_friction = 0.17;
    let bounciness = 0.9;
    let terminal_velocity = 10000.;
    let radius = 60.;
    let mut ball_position = Vec2::ZERO;
    let mut ball_velocity = Vec2::ZERO;

    loop {
        if is_mouse_button_down(MouseButton::Left) {
            delta_window_position = mouse_delta_position() * Vec2::new(WIDTH_F / 2., HEIGHT_F / 2.)
                + delta_window_position;
            last_window_position -= delta_window_position;
            set_window_position(last_window_position.x as u32, last_window_position.y as u32);
        } else {
            let current_window_position =
                Vec2::from_u32_tuple(miniquad::window::get_window_position());
            delta_window_position = last_window_position - current_window_position;
            last_window_position = current_window_position;
        }

        smoothed_delta = smoothed_delta.lerp(delta_window_position, 0.25);
        smoothed_magnitude = smoothed_magnitude
            .lerp(smoothed_delta.length(), 0.15)
            .min(smoothed_delta.length());

        let smoothed_delta = if smoothed_delta.length() != 0. {
            smoothed_delta.normalize() * smoothed_magnitude
        } else {
            smoothed_delta
        };

        let maxed_delta = smoothed_delta.max(delta_window_position) / get_frame_time() * 2.;

        clear_background(LIGHTGRAY);

        ball_velocity += Vec2::new(0., gravity_strength * get_frame_time());

        ball_velocity *= 1. - (air_friction * get_frame_time().clamp(0., 1.));

        let total_velocity = if time::get_time() > 1. {
            ball_velocity + (delta_window_position / get_frame_time()) * 2.
        } else {
            ball_velocity
        };

        ball_position += total_velocity * get_frame_time();

        set_camera(&Camera2D {
            zoom: vec2(1. / WIDTH_F, 1. / HEIGHT_F),
            ..Default::default()
        });

        if ball_position.y > HEIGHT_F - radius {
            ball_position.y = HEIGHT_F - radius;
            ball_velocity.y = -(ball_velocity.y + maxed_delta.y) * bounciness;
        } else if ball_position.y < -HEIGHT_F + radius {
            ball_position.y = -HEIGHT_F + radius;
            ball_velocity.y = -(ball_velocity.y + maxed_delta.y) * bounciness;
        }

        if ball_position.x > WIDTH_F - radius {
            ball_position.x = WIDTH_F - radius;
            ball_velocity.x = -(ball_velocity.x + maxed_delta.x) * bounciness;
        } else if ball_position.x < -WIDTH_F + radius {
            ball_position.x = -WIDTH_F + radius;
            ball_velocity.x = -(ball_velocity.x + maxed_delta.x) * bounciness;
        }

        if ball_velocity.length() > terminal_velocity {
            println!("Reached terminal velocity!");
            ball_velocity = ball_velocity.normalize() * terminal_velocity;
        }

        draw_circle(ball_position.x, ball_position.y, radius, BLUE);

        next_frame().await
    }
}
