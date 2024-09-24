use core::str;
use std::{f32::consts::PI, fs};

use ball::Ball;
use macroquad::{
    audio::set_sound_volume,
    prelude::*,
    ui::{root_ui, Skin},
};
use miniquad::*;
use nanoserde::{DeJson, SerJson};
use ui::{render_ui, SettingsState, MENU_SIZE};
use window::set_window_position;

pub mod ball;
pub mod sounds;
pub mod textures;
pub mod ui;

const WIDTH: i32 = 640;
const HEIGHT: i32 = 480;

const WIDTH_F: f32 = WIDTH as f32;
const HEIGHT_F: f32 = HEIGHT as f32;

pub const WALL_THICKNESS: f32 = 20.;
pub const WALL_DEPTH: f32 = 20.;
pub const WALL_OFFSET: f32 = WALL_THICKNESS + WALL_DEPTH;

pub fn window_conf() -> Conf {
    Conf {
        window_title: "Ball in a Box".to_string(),
        window_width: WIDTH,
        window_height: HEIGHT,
        high_dpi: true,
        borderless: true,
        fullscreen: false,
        window_resizable: false,
        sample_count: 0,
        ..Default::default()
    }
}

const BALL_FRAGMENT_SHADER: &'static str = include_str!("../assets/ball.frag");
const SHADOW_FRAGMENT_SHADER: &'static str = include_str!("../assets/shadow.frag");
const VERTEX_SHADER: &'static str = include_str!("../assets/ball.vert");

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

#[derive(Debug, SerJson, DeJson, Clone)]
pub struct Settings {
    gravity_strength: f32,
    air_friction: f32,
    bounciness: f32,
    terminal_velocity: f32,
    ball_radius: f32,
    audio_volume: f32,
    shadow_size: f32,
    shadow_distance_strength: f32,
    last_ball: String,
    last_sounds: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            gravity_strength: 3.,
            air_friction: 0.17,
            bounciness: 0.9,
            terminal_velocity: 100.,
            ball_radius: 90.,
            audio_volume: 0.6,
            shadow_size: 1.2,
            shadow_distance_strength: 50.,
            last_ball: "".to_string(),
            last_sounds: "".to_string(),
        }
    }
}

fn read_settings_file() -> Option<Settings> {
    let result = fs::read("./settings_in_a.json");
    let Ok(bytes) = result else {
        return None;
    };

    let Ok(string) = str::from_utf8(&bytes) else {
        return None;
    };
    return Settings::deserialize_json(string).ok();
}

fn write_settings_file(settings: &Settings) {
    let _ = fs::write("./settings_in_a.json", settings.serialize_json());
}

#[macroquad::main(window_conf)]
async fn main() {
    set_window_position((1920 - WIDTH as u32) / 2, (1080 - HEIGHT as u32) / 2);
    next_frame().await;

    let mut settings = read_settings_file().unwrap_or_else(|| {
        let settings = Settings::default();
        write_settings_file(&settings);
        settings
    });

    let mut editing_settings = settings.clone();

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

    let ball_textures = textures::BallTextures::new();

    let ball_sounds = sounds::BallSounds::new().await;

    let max_string_len = 100;

    let mut text_input = String::new();

    let background_texture =
        Texture2D::from_file_with_format(include_bytes!("../assets/background.png"), None);

    let side_texture =
        Texture2D::from_file_with_format(include_bytes!("../assets/cardboardsidebottom.png"), None);

    let font = load_ttf_font_from_bytes(include_bytes!("../assets/FrederickatheGreat-Regular.ttf"))
        .unwrap();

    let window_style = root_ui()
        .style_builder()
        .background(
            Image::from_file_with_format(include_bytes!("../assets/main_background.png"), None)
                .unwrap(),
        )
        .build();

    let button_style = root_ui()
        .style_builder()
        .background(
            Image::from_file_with_format(include_bytes!("../assets/cardboard_button.png"), None)
                .unwrap(),
        )
        .with_font(&font)
        .unwrap()
        .font_size(28)
        .text_color(Color::new(0.05, 0., 0.1, 1.))
        .color_hovered(Color::new(0.90, 0.90, 0.90, 1.0))
        .build();

    let label_style = root_ui()
        .style_builder()
        .with_font(&font)
        .unwrap()
        .font_size(24)
        .text_color(Color::new(0.05, 0., 0.1, 1.))
        .margin(RectOffset::new(0., 0., 10., 0.))
        .build();

    let editbox_style = root_ui()
        .style_builder()
        .with_font(&font)
        .unwrap()
        .font_size(16)
        .text_color(Color::new(0., 0., 0., 1.))
        .color(Color::new(0.0, 0.90, 0.90, 0.0))
        .color_selected(Color::new(0.0, 0.90, 0.90, 0.0))
        .color_clicked(Color::new(0.0, 0.90, 0.90, 0.0))
        .build();

    let checkbox_style = root_ui()
        .style_builder()
        .font_size(18)
        .color(Color::from_rgba(222, 185, 140, 255))
        .color_hovered(Color::from_rgba(138, 101, 56, 255))
        .color_clicked(Color::from_rgba(112, 77, 35, 255))
        .build();

    let group_style = root_ui()
        .style_builder()
        .color(Color::new(0., 0., 0., 0.))
        .build();

    let skin = Skin {
        window_style,
        button_style,
        label_style,
        editbox_style,
        checkbox_style,
        group_style,
        ..root_ui().default_skin()
    };

    root_ui().push_skin(&skin);

    let mut is_in_settings = false;
    let mut settings_state = SettingsState::Closed;
    let mut interacting_with_ui = false;

    let mut last_mouse_position = Vec2::from_i32_tuple(window::get_screen_mouse_position());

    let mut mouse_offset = Vec2::ZERO;

    let mut smoothed_delta = Vec2::ZERO;
    let mut smoothed_magnitude = 0.;

    let mut ball = Ball::new(
        ball_textures.get_texture(&settings.last_ball),
        ball_material,
        shadow_material,
        ball_sounds
            .find(&settings.last_sounds)
            .unwrap_or_else(|| ball_sounds.get_first())
            .1,
    );

    set_camera(&Camera2D {
        zoom: vec2(1. / WIDTH_F, 1. / HEIGHT_F),
        ..Default::default()
    });

    loop {
        clear_background(DARKGRAY);

        if is_key_pressed(KeyCode::Escape) {
            if settings_state != SettingsState::Closed {
                settings_state = SettingsState::Closed
            } else {
                settings_state = SettingsState::Open
            }
        }

        if let SettingsState::Settings(_) = settings_state {
            if !is_in_settings {
                editing_settings = settings.clone();
                is_in_settings = true
            }
        } else {
            is_in_settings = false
        }

        let is_menu_open = settings_state.is_open();

        let save = render_ui(&mut editing_settings, &mut settings_state);
        if save {
            settings = editing_settings.clone();
            write_settings_file(&settings);
            for sound in ball.sounds.iter() {
                set_sound_volume(sound, settings.audio_volume);
            }
        }

        while let Some(character) = get_char_pressed() {
            text_input.push(character.to_ascii_lowercase());

            if let Some((ball_name, texture)) = ball_textures.find_custom(&text_input) {
                ball.texture = Texture2D::from_file_with_format(&texture, None);
                settings.last_ball = ball_name.clone();
                editing_settings.last_ball = ball_name;
                write_settings_file(&settings);
            } else if let Some((ball_name, texture)) = ball_textures.find(&text_input) {
                ball.texture = Texture2D::from_file_with_format(texture, None);
                settings.last_ball = ball_name.to_string();
                editing_settings.last_ball = ball_name.to_string();
                write_settings_file(&settings);
            }

            if let Some((sounds_name, sounds)) = ball_sounds.find(&text_input) {
                ball.sounds = sounds.clone();
                settings.last_sounds = sounds_name.clone();
                editing_settings.last_sounds = sounds_name;
                write_settings_file(&settings);
            }
        }

        if text_input.len() > max_string_len {
            let remove = text_input.len() - max_string_len;
            text_input = text_input[remove..].to_string();
        }

        let current_mouse_position = Vec2::from_i32_tuple(window::get_screen_mouse_position());
        let delta_mouse_position = current_mouse_position - last_mouse_position;
        last_mouse_position = current_mouse_position;

        if is_mouse_button_pressed(MouseButton::Left) && is_menu_open {
            let abs_mouse_pos_from_center =
                (Vec2::from_f32_tuple(mouse_position()) - vec2(WIDTH_F, HEIGHT_F) / 2.).abs();
            if abs_mouse_pos_from_center.x < MENU_SIZE.x / 2.
                && abs_mouse_pos_from_center.y < MENU_SIZE.y / 2.
            {
                interacting_with_ui = true
            }
        }
        if is_mouse_button_released(MouseButton::Left) {
            interacting_with_ui = false
        }

        let delta_pos = if interacting_with_ui {
            Vec2::ZERO
        } else if is_mouse_button_down(MouseButton::Left) {
            if is_mouse_button_pressed(MouseButton::Left) {
                let current_window_position =
                    Vec2::from_u32_tuple(miniquad::window::get_window_position());
                mouse_offset = current_window_position - current_mouse_position;
            }
            let new_pos = current_mouse_position + mouse_offset;
            set_window_position(new_pos.x as u32, new_pos.y as u32);
            -delta_mouse_position
        } else {
            Vec2::ZERO
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

        draw_texture_ex(
            &background_texture,
            -WIDTH_F + WALL_THICKNESS,
            -HEIGHT_F + WALL_THICKNESS,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    (WIDTH_F - WALL_THICKNESS) * 2.,
                    (HEIGHT_F - WALL_THICKNESS) * 2.,
                )),
                ..Default::default()
            },
        );

        draw_texture_ex(
            &side_texture,
            -WIDTH_F * 2. + WALL_THICKNESS / 2.,
            0.,
            Color::from_hex(0x999999),
            DrawTextureParams {
                rotation: PI * 0.5,
                dest_size: Some(vec2(WIDTH_F * 2., WALL_THICKNESS)),
                ..Default::default()
            },
        );

        draw_texture_ex(
            &side_texture,
            -WALL_THICKNESS / 2.,
            0.,
            Color::from_hex(0xb0b0b0),
            DrawTextureParams {
                rotation: PI * 1.5,
                dest_size: Some(vec2(WIDTH_F * 2., WALL_THICKNESS)),
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
                dest_size: Some(vec2(WIDTH_F * 2., WALL_THICKNESS)),
                ..Default::default()
            },
        );

        draw_texture_ex(
            &side_texture,
            -WIDTH_F,
            HEIGHT_F - WALL_THICKNESS,
            Color::from_hex(0xe0e0e0),
            DrawTextureParams {
                rotation: PI * 2.0,
                dest_size: Some(vec2(WIDTH_F * 2., WALL_THICKNESS)),
                ..Default::default()
            },
        );

        ball.step_and_render(get_frame_time(), &settings, delta_pos, maxed_delta);

        if is_menu_open {
            draw_rectangle(
                -WIDTH_F,
                -HEIGHT_F,
                WIDTH_F * 2.,
                HEIGHT_F * 2.,
                Color::from_rgba(0, 0, 0, 100),
            );
        }

        next_frame().await
    }
}
