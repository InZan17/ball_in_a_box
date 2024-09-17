use std::f32::consts::PI;

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
        sample_count: 8,
        window_resizable: false,
        ..Default::default()
    }
}

const FRAGMENT_SHADER: &'static str = include_str!("../assets/ball.frag");
const VERTEX_SHADER: &'static str = include_str!("../assets/ball.vert");

const BALL_TEXTURE_BYTES: &[u8] = include_bytes!("../assets/ball.png");
const BACKGROUND_TEXTURE_BYTES: &[u8] = include_bytes!("../assets/background.png");
const SIDE_TEXTURE_BYTES: &[u8] = include_bytes!("../assets/cardboardsidebottom.png");

pub trait FromTuple {
    fn from_u32_tuple(tuple: (u32, u32)) -> Self;
    fn from_f32_tuple(tuple: (f32, f32)) -> Self;
}

impl FromTuple for Vec2 {
    fn from_u32_tuple(tuple: (u32, u32)) -> Self {
        Vec2::new(tuple.0 as f32, tuple.1 as f32)
    }

    fn from_f32_tuple(tuple: (f32, f32)) -> Self {
        Vec2::new(tuple.0, tuple.1)
    }
}

pub fn calculate_bounce_spin(
    ball_velocity: f32,
    window_velocity: f32,
    ball_rotation_velocity: f32,
    ball_radius: f32,
    inverted: bool,
) -> (f32, f32) {
    let total_velocity = if inverted {
        -(ball_velocity + window_velocity)
    } else {
        ball_velocity + window_velocity
    };
    let rotation_velocity_from_velocity = total_velocity / ball_radius;
    let delta_rotation_velocity = rotation_velocity_from_velocity.lerp(ball_rotation_velocity, 0.5);
    let current_rotation_direction_velocity = if inverted {
        -(delta_rotation_velocity * ball_radius)
    } else {
        delta_rotation_velocity * ball_radius
    };
    let new_rotation_velocity = delta_rotation_velocity.lerp(rotation_velocity_from_velocity, 0.5);
    return (
        new_rotation_velocity,
        current_rotation_direction_velocity - window_velocity,
    );
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut last_window_position = Vec2::from_u32_tuple(miniquad::window::get_window_position());
    let mut last_mouse_position = Vec2::ZERO;

    let mut mouse_offset = Vec2::ZERO;

    let mut smoothed_delta = Vec2::ZERO;
    let mut smoothed_magnitude = 0.;

    let gravity_strength = 3000.;
    let air_friction = 0.17;
    let bounciness = 0.9;
    let terminal_velocity = 10000.;
    let radius = 90.;
    let wall_thickness = 25.;
    let mut ball_position = Vec2::ZERO;
    let mut ball_velocity = Vec2::ZERO;
    let mut ball_rotation = 0.;
    let mut ball_rotation_velocity = 0.;

    let wall_offset = radius + wall_thickness + 20.;

    let ball_material = load_material(
        ShaderSource::Glsl {
            vertex: VERTEX_SHADER,
            fragment: FRAGMENT_SHADER,
        },
        MaterialParams::default(),
    )
    .expect("Failed to load material.");

    let ball_texture = Texture2D::from_file_with_format(BALL_TEXTURE_BYTES, None);
    let background_texture = Texture2D::from_file_with_format(BACKGROUND_TEXTURE_BYTES, None);
    let side_texture = Texture2D::from_file_with_format(SIDE_TEXTURE_BYTES, None);

    loop {
        let current_window_position = Vec2::from_u32_tuple(miniquad::window::get_window_position());
        let delta_window_position = last_window_position - current_window_position;
        last_window_position = current_window_position;

        let current_mouse_position = Vec2::from_u32_tuple(other::screen_mouse_position());
        let delta_mouse_position = current_mouse_position - last_mouse_position;
        last_mouse_position = current_mouse_position;

        let delta_pos = if is_mouse_button_down(MouseButton::Left) {
            if is_mouse_button_pressed(MouseButton::Left) {
                mouse_offset = current_window_position - current_mouse_position;
            }
            let new_pos = current_mouse_position + mouse_offset;
            set_window_position(new_pos.x as u32, new_pos.y as u32);
            -delta_mouse_position
        } else {
            delta_window_position
        };

        smoothed_delta = smoothed_delta.lerp(delta_pos, 0.5);
        smoothed_magnitude = smoothed_magnitude
            .lerp(smoothed_delta.length(), 0.15)
            .min(smoothed_delta.length());

        let smoothed_delta = if smoothed_delta.length() != 0. {
            smoothed_delta.normalize() * smoothed_magnitude
        } else {
            smoothed_delta
        };

        let maxed_delta = smoothed_delta.max(delta_pos) / get_frame_time() * 2.;

        clear_background(LIGHTGRAY);

        ball_velocity += Vec2::new(0., gravity_strength * get_frame_time());

        ball_velocity *= 1. - (air_friction * get_frame_time().clamp(0., 1.));

        let total_velocity = if time::get_time() > 1. {
            ball_velocity + (delta_pos / get_frame_time()) * 2.
        } else {
            ball_velocity
        };

        let smoothed_total_velocity = if time::get_time() > 1. {
            ball_velocity + maxed_delta
        } else {
            ball_velocity
        };

        ball_position += total_velocity * get_frame_time();

        ball_rotation += ball_rotation_velocity * get_frame_time();

        set_camera(&Camera2D {
            zoom: vec2(1. / WIDTH_F, 1. / HEIGHT_F),
            ..Default::default()
        });

        if ball_position.y > HEIGHT_F - wall_offset {
            // Floor
            ball_position.y = HEIGHT_F - wall_offset;
            ball_velocity.y = -smoothed_total_velocity.y * bounciness;

            (ball_rotation_velocity, ball_velocity.x) = calculate_bounce_spin(
                ball_velocity.x,
                maxed_delta.x,
                ball_rotation_velocity,
                radius,
                false,
            );
        } else if ball_position.y < -HEIGHT_F + wall_offset {
            // Ceiling
            ball_position.y = -HEIGHT_F + wall_offset;
            ball_velocity.y = -smoothed_total_velocity.y * bounciness;

            (ball_rotation_velocity, ball_velocity.x) = calculate_bounce_spin(
                ball_velocity.x,
                maxed_delta.x,
                ball_rotation_velocity,
                radius,
                true,
            );
        }

        if ball_position.x > WIDTH_F - wall_offset {
            // Right
            ball_position.x = WIDTH_F - wall_offset;
            ball_velocity.x = -smoothed_total_velocity.x * bounciness;

            (ball_rotation_velocity, ball_velocity.y) = calculate_bounce_spin(
                ball_velocity.y,
                maxed_delta.y,
                ball_rotation_velocity,
                radius,
                true,
            );
        } else if ball_position.x < -WIDTH_F + wall_offset {
            // Left
            ball_position.x = -WIDTH_F + wall_offset;
            ball_velocity.x = -smoothed_total_velocity.x * bounciness;

            (ball_rotation_velocity, ball_velocity.y) = calculate_bounce_spin(
                ball_velocity.y,
                maxed_delta.y,
                ball_rotation_velocity,
                radius,
                false,
            );
        }

        if ball_velocity.length() > terminal_velocity {
            println!("Reached terminal velocity!");
            ball_velocity = ball_velocity.normalize() * terminal_velocity;
        }

        //draw_circle(ball_position.x, ball_position.y, radius, BLUE);

        draw_texture_ex(
            &background_texture,
            -WIDTH_F + wall_thickness,
            -HEIGHT_F + wall_thickness,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    (WIDTH_F - wall_thickness) * 2.,
                    (HEIGHT_F - wall_thickness) * 2.,
                )),
                ..Default::default()
            },
        );

        draw_texture_ex(
            &side_texture,
            -WIDTH_F * 2. + wall_thickness / 2.,
            0.,
            Color::from_hex(0xf1f1f1),
            DrawTextureParams {
                rotation: PI * 0.5,
                dest_size: Some(vec2(WIDTH_F * 2., wall_thickness)),
                ..Default::default()
            },
        );

        draw_texture_ex(
            &side_texture,
            -wall_thickness / 2.,
            0.,
            Color::from_hex(0xf1f1f1),
            DrawTextureParams {
                rotation: PI * 1.5,
                dest_size: Some(vec2(WIDTH_F * 2., wall_thickness)),
                ..Default::default()
            },
        );

        draw_texture_ex(
            &side_texture,
            -WIDTH_F,
            -HEIGHT_F,
            Color::from_hex(0xf1f1f1),
            DrawTextureParams {
                rotation: PI * 1.0,
                dest_size: Some(vec2(WIDTH_F * 2., wall_thickness)),
                ..Default::default()
            },
        );

        draw_texture_ex(
            &side_texture,
            -WIDTH_F,
            HEIGHT_F - wall_thickness,
            Color::from_hex(0xf1f1f1),
            DrawTextureParams {
                rotation: PI * 2.0,
                dest_size: Some(vec2(WIDTH_F * 2., wall_thickness)),
                ..Default::default()
            },
        );

        gl_use_material(&ball_material);

        draw_texture_ex(
            &ball_texture,
            ball_position.x - radius,
            ball_position.y - radius,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(radius * 2., radius * 2.)),
                rotation: ball_rotation,
                ..Default::default()
            },
        );

        gl_use_default_material();

        next_frame().await
    }
}
