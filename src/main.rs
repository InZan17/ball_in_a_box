use std::f32::consts::PI;

use macroquad::{prelude::*, time};
use miniquad::*;
use window::set_window_position;

const WIDTH: i32 = 640;
const HEIGHT: i32 = 480;

const WIDTH_F: f32 = WIDTH as f32;
const HEIGHT_F: f32 = HEIGHT as f32;

pub fn window_conf() -> Conf {
    Conf {
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

const BALL_FRAGMENT_SHADER: &'static str = include_str!("../assets/ball.frag");
const SHADOW_FRAGMENT_SHADER: &'static str = include_str!("../assets/shadow.frag");
const VERTEX_SHADER: &'static str = include_str!("../assets/ball.vert");

const EARTH_BALL_TEXTURE_BYTES: &[u8] = include_bytes!("../assets/earth.png");
const WHITE_BALL_TEXTURE_BYTES: &[u8] = include_bytes!("../assets/white.png");
const DISTRESS_BALL_TEXTURE_BYTES: &[u8] = include_bytes!("../assets/distress.png");
const GRINNING_BALL_TEXTURE_BYTES: &[u8] = include_bytes!("../assets/grinning.png");

const BACKGROUND_TEXTURE_BYTES: &[u8] = include_bytes!("../assets/background.png");
const SIDE_TEXTURE_BYTES: &[u8] = include_bytes!("../assets/cardboardsidebottom.png");

pub trait FromTuple {
    fn from_u32_tuple(tuple: (u32, u32)) -> Self;
    fn from_i32_tuple(tuple: (i32, i32)) -> Self;
    fn from_f32_tuple(tuple: (f32, f32)) -> Self;
}

impl FromTuple for Vec2 {
    fn from_u32_tuple(tuple: (u32, u32)) -> Self {
        Vec2::new(tuple.0 as f32, tuple.1 as f32)
    }

    fn from_i32_tuple(tuple: (i32, i32)) -> Self {
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
    set_window_position((1920-WIDTH as u32)/2, (1080-HEIGHT  as u32)/2);
    next_frame().await;
    
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
    let wall_thickness = 20.;
    let wall_depth = 20.;

    let wall_offset = radius + wall_thickness + wall_depth;
    let shadow_size = 1.2;
    let shadow_distance_strength = 50.;

    let mut ball_position = Vec2::ZERO;
    let mut ball_velocity = Vec2::ZERO;
    let mut ball_rotation = 0.;
    let mut ball_rotation_velocity = 0.;

    let ball_material = load_material(
        ShaderSource::Glsl {
            vertex: VERTEX_SHADER,
            fragment: BALL_FRAGMENT_SHADER,
        },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("rotation", UniformType::Float1),
                UniformDesc::new("ceil_distance", UniformType::Float1),
                UniformDesc::new("floor_distance", UniformType::Float1),
                UniformDesc::new("left_distance", UniformType::Float1),
                UniformDesc::new("right_distance", UniformType::Float1),
            ],
            pipeline_params: PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("Failed to load ball material.");

    let shadow_material = load_material(
        ShaderSource::Glsl {
            vertex: VERTEX_SHADER,
            fragment: SHADOW_FRAGMENT_SHADER,
        },
        MaterialParams {
            uniforms: vec![UniformDesc::new("in_shadow", UniformType::Float1)],
            pipeline_params: PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("Failed to load shadow material.");

    let ball_textures = [
        (
            "earth",
            Texture2D::from_file_with_format(EARTH_BALL_TEXTURE_BYTES, None),
        ),
        (
            "distress",
            Texture2D::from_file_with_format(DISTRESS_BALL_TEXTURE_BYTES, None),
        ),
        (
            "grinning",
            Texture2D::from_file_with_format(GRINNING_BALL_TEXTURE_BYTES, None),
        ),
        (
            "white",
            Texture2D::from_file_with_format(WHITE_BALL_TEXTURE_BYTES, None),
        ),
    ];

    let max_len = {
        let mut max_len = 0;

        for (name, _) in ball_textures.iter() {
            max_len = max_len.max(name.len());
        }
        max_len
    };

    let mut ball_texture = ball_textures[0].1.clone();

    let mut text_input = String::new();

    let background_texture = Texture2D::from_file_with_format(BACKGROUND_TEXTURE_BYTES, None);
    let side_texture = Texture2D::from_file_with_format(SIDE_TEXTURE_BYTES, None);

    loop {
        let mut pressed = false;

        while let Some(character) = get_char_pressed() {
            text_input.push(character.to_ascii_lowercase());
            pressed = true;
        }

        if pressed {
            for (name, texture) in ball_textures.iter() {
                if text_input.contains(name) {
                    ball_texture = texture.clone();
                    text_input.clear();
                    break;
                }
            }
        }

        if text_input.len() > max_len {
            let remove = text_input.len() - max_len;
            text_input = text_input[remove..].to_string();
        }

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

        let mut distance_to_floor = HEIGHT_F - wall_offset - ball_position.y;
        if distance_to_floor <= 0. {
            // Floor
            distance_to_floor = 0.;
            ball_position.y = HEIGHT_F - wall_offset;
            ball_velocity.y = -smoothed_total_velocity.y * bounciness;

            (ball_rotation_velocity, ball_velocity.x) = calculate_bounce_spin(
                ball_velocity.x,
                maxed_delta.x,
                ball_rotation_velocity,
                radius,
                false,
            );
        }

        let mut distance_to_ceiling = ball_position.y + HEIGHT_F - wall_offset;
        if distance_to_ceiling <= 0. {
            // Ceiling
            distance_to_ceiling = 0.;
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
        let mut distance_to_right_wall = WIDTH_F - wall_offset - ball_position.x;
        if distance_to_right_wall <= 0. {
            // Right
            distance_to_right_wall = 0.;
            ball_position.x = WIDTH_F - wall_offset;
            ball_velocity.x = -smoothed_total_velocity.x * bounciness;

            (ball_rotation_velocity, ball_velocity.y) = calculate_bounce_spin(
                ball_velocity.y,
                maxed_delta.y,
                ball_rotation_velocity,
                radius,
                true,
            );
        }

        let mut distance_to_left_wall = ball_position.x + WIDTH_F - wall_offset;
        if distance_to_left_wall <= 0. {
            // Left
            distance_to_left_wall = 0.;
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
            Color::from_hex(0x999999),
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
            Color::from_hex(0xb0b0b0),
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
            Color::from_hex(0xbababa),
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
            Color::from_hex(0xe0e0e0),
            DrawTextureParams {
                rotation: PI * 2.0,
                dest_size: Some(vec2(WIDTH_F * 2., wall_thickness)),
                ..Default::default()
            },
        );

        gl_use_material(&shadow_material);

        shadow_material.set_uniform("in_shadow", distance_to_floor / shadow_distance_strength);
        draw_rectangle(
            ball_position.x - radius * shadow_size,
            HEIGHT_F - wall_offset + radius - wall_depth,
            radius * shadow_size * 2.,
            wall_depth * 2.,
            WHITE,
        );

        shadow_material.set_uniform("in_shadow", distance_to_ceiling / shadow_distance_strength);
        draw_rectangle(
            ball_position.x - radius * shadow_size,
            -HEIGHT_F + wall_thickness,
            radius * shadow_size * 2.,
            wall_depth * 2.,
            WHITE,
        );

        shadow_material.set_uniform(
            "in_shadow",
            distance_to_right_wall / shadow_distance_strength,
        );
        draw_rectangle(
            WIDTH_F - wall_offset + radius - wall_depth,
            ball_position.y - radius * shadow_size,
            wall_depth * 2.,
            radius * shadow_size * 2.,
            WHITE,
        );

        shadow_material.set_uniform(
            "in_shadow",
            distance_to_left_wall / shadow_distance_strength,
        );
        draw_rectangle(
            -WIDTH_F + wall_thickness,
            ball_position.y - radius * shadow_size,
            wall_depth * 2.,
            radius * shadow_size * 2.,
            WHITE,
        );

        ball_material.set_uniform("rotation", ball_rotation);
        ball_material.set_uniform(
            "floor_distance",
            distance_to_floor / shadow_distance_strength,
        );
        ball_material.set_uniform(
            "ceil_distance",
            distance_to_ceiling / shadow_distance_strength,
        );
        ball_material.set_uniform(
            "left_distance",
            distance_to_left_wall / shadow_distance_strength,
        );
        ball_material.set_uniform(
            "right_distance",
            distance_to_right_wall / shadow_distance_strength,
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
