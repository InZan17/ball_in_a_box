use core::str;
use std::{
    f32::consts::PI,
    fs::{self, OpenOptions},
    io::Write,
    panic,
    time::{SystemTime, UNIX_EPOCH},
};

use ball::Ball;
use macroquad::{audio::set_sound_volume, prelude::*, rand, ui::root_ui};
use miniquad::*;
use nanoserde::{DeJson, SerJson};
use sounds::{find_sounds, get_random_sounds};
use textures::{find_texture, get_random_texture};
use ui::{create_skin, render_ui, SettingsState, UiRenderer, MENU_SIZE};
use window::set_window_position;

pub mod ball;
pub mod sounds;
pub mod textures;
pub mod ui;

pub const WALL_THICKNESS: f32 = 20.;
pub const WALL_DEPTH: f32 = 20.;
pub const WALL_OFFSET: f32 = WALL_THICKNESS + WALL_DEPTH;

pub fn window_conf() -> Conf {
    let settings = read_settings_file().unwrap_or_default();

    Conf {
        window_title: "Ball in a Box".to_string(),
        window_width: settings.box_width as i32,
        window_height: settings.box_height as i32,
        high_dpi: true,
        borderless: true,
        fullscreen: false,
        window_resizable: false,
        sample_count: 0,
        ..Default::default()
    }
}

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
    terminal_velocity: f32,
    gravity_strength: f32,
    air_friction: f32,
    ball_bounciness: f32,
    ball_radius: f32,
    ball_weight: f32,
    ball_friction: f32,
    audio_volume: f32,
    shadow_size: f32,
    shadow_distance_strength: f32,
    box_width: f32,
    box_height: f32,
    last_ball: String,
    last_sounds: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            gravity_strength: 3.,
            air_friction: 0.17,
            terminal_velocity: 100.,
            ball_bounciness: 0.9,
            ball_radius: 90.,
            ball_weight: 0.65,
            ball_friction: 0.75,
            audio_volume: 0.6,
            shadow_size: 1.2,
            shadow_distance_strength: 0.55,
            box_width: 640.,
            box_height: 480.,
            last_ball: "grinning".to_string(),
            last_sounds: "thud".to_string(),
        }
    }
}

fn read_settings_file() -> Option<Settings> {
    let bytes = fs::read("./settings_in_a.json").ok()?;
    let string = str::from_utf8(&bytes).ok()?;
    let mut settings = Settings::deserialize_json(string).ok()?;

    // Fix values so it doesn't break when creating windows.
    settings.box_width = settings.box_width.floor().max(1.0);
    settings.box_height = settings.box_height.floor().max(1.0);

    return Some(settings);
}

fn write_settings_file(settings: &Settings) {
    let _ = fs::write("./settings_in_a.json", settings.serialize_json());
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut settings = read_settings_file().unwrap_or_else(|| {
        let settings = Settings::default();
        write_settings_file(&settings);
        settings
    });

    let mut editing_settings = settings.clone();

    set_window_position(
        (1920 - settings.box_width as u32) / 2,
        (1080 - settings.box_height as u32) / 2,
    );
    next_frame().await;

    panic::set_hook(Box::new(|info| {
        let Ok(mut log_file) = OpenOptions::new()
            .create(true)
            .write(true)
            .open("crash_info.txt")
        else {
            return;
        };

        let panic_message = if let Some(s) = info.payload().downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.to_string()
        } else {
            "Unknown panic message".to_string()
        };

        let _ = log_file.write(format!("{panic_message}").as_bytes());
    }));

    {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        rand::srand(since_the_epoch.as_nanos() as u64);
    }

    let background_texture = Texture2D::from_file_with_format(
        &load_file("./assets/background.png")
            .await
            .expect("Couldn't find the assets/background.png file"),
        None,
    );

    let side_texture = Texture2D::from_file_with_format(
        &load_file("./assets/cardboardside.png")
            .await
            .expect("Couldn't find the assets/cardboardside.png file"),
        None,
    );

    let default_vert = load_string("assets/default.vert")
        .await
        .expect("Couldn't find the assets/default.vert file");

    let ball_material = load_material(
        ShaderSource::Glsl {
            vertex: &default_vert,
            fragment: &load_string("assets/ball.frag")
                .await
                .expect("Couldn't find the assets/ball.frag file"),
        },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("rotation", UniformType::Float1),
                UniformDesc::new("ceil_distance", UniformType::Float1),
                UniformDesc::new("floor_distance", UniformType::Float1),
                UniformDesc::new("left_distance", UniformType::Float1),
                UniformDesc::new("right_distance", UniformType::Float1),
                UniformDesc::new("ball_radius", UniformType::Float1),
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
    .expect("Failed to load ball material");

    let shadow_material = load_material(
        ShaderSource::Glsl {
            vertex: &default_vert,
            fragment: &load_string("assets/shadow.frag")
                .await
                .expect("Couldn't find the assets/shadow.frag file"),
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
    .expect("Failed to load shadow material");

    drop(default_vert);

    let max_string_len = 100;

    let mut text_input = String::new();

    let skin = create_skin().await;

    root_ui().push_skin(&skin);

    let mut ui_renderer = UiRenderer::new().await;

    let mut is_in_settings = false;
    let mut settings_state = SettingsState::Closed;
    let mut interacting_with_ui = false;

    let mut last_mouse_position = Vec2::from_i32_tuple(window::get_screen_mouse_position());

    let mut mouse_offset: Option<Vec2> = None;

    let mut smoothed_delta = Vec2::ZERO;
    let mut smoothed_magnitude = 0.;

    let mut ball = {
        let option_sounds = find_sounds(&settings.last_sounds).await;

        let sounds = if let Some(sounds) = option_sounds {
            sounds
        } else {
            get_random_sounds().await
        };

        Ball::new(
            find_texture(&settings.last_ball)
                .unwrap_or_else(|| get_random_texture())
                .1,
            ball_material,
            shadow_material,
            sounds.1,
        )
    };

    set_camera(&Camera2D {
        zoom: vec2(1. / settings.box_width, 1. / settings.box_height),
        ..Default::default()
    });

    let max_axis = settings.box_width.max(settings.box_height);

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

        let mouse_pos = if let Some(mouse_pos) = mouse_offset {
            -mouse_pos
        } else {
            Vec2::from_f32_tuple(mouse_position()) * screen_dpi_scale()
        };

        while let Some(character) = get_char_pressed() {
            text_input.push(character.to_ascii_lowercase());

            if let Some((ball_name, texture)) = find_texture(&text_input) {
                ball.texture = texture;
                settings.last_ball = ball_name.clone();
                editing_settings.last_ball = ball_name;
                write_settings_file(&settings);
            }

            if let Some((sounds_name, sounds)) = find_sounds(&text_input).await {
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
                (mouse_pos - vec2(settings.box_width, settings.box_height) / 2.).abs();
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
            let mouse_offset = match mouse_offset {
                Some(mouse_offset) => mouse_offset,
                None => {
                    mouse_offset = Some(-mouse_pos);
                    -mouse_pos
                }
            };
            let new_pos = current_mouse_position + mouse_offset;
            set_window_position(new_pos.x as u32, new_pos.y as u32);
            -delta_mouse_position
        } else {
            mouse_offset = None;
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
            -settings.box_width + WALL_THICKNESS,
            -settings.box_height + WALL_THICKNESS,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    (settings.box_width - WALL_THICKNESS) * 2.,
                    (settings.box_height - WALL_THICKNESS) * 2.,
                )),
                ..Default::default()
            },
        );

        // Left
        draw_texture_ex(
            &side_texture,
            -settings.box_width - max_axis + WALL_THICKNESS / 2.,
            0.,
            Color::from_hex(0x999999),
            DrawTextureParams {
                rotation: PI * 0.5,
                dest_size: Some(vec2(max_axis * 2., WALL_THICKNESS)),
                ..Default::default()
            },
        );

        // Right
        draw_texture_ex(
            &side_texture,
            -WALL_THICKNESS / 2. - max_axis + settings.box_width,
            0.,
            Color::from_hex(0xb0b0b0),
            DrawTextureParams {
                rotation: PI * 1.5,
                dest_size: Some(vec2(max_axis * 2., WALL_THICKNESS)),
                ..Default::default()
            },
        );

        // Top
        draw_texture_ex(
            &side_texture,
            -settings.box_width,
            -settings.box_height,
            Color::from_hex(0xbababa),
            DrawTextureParams {
                rotation: PI * 1.0,
                dest_size: Some(vec2(max_axis * 2., WALL_THICKNESS)),
                ..Default::default()
            },
        );

        // Bottom
        draw_texture_ex(
            &side_texture,
            -settings.box_width,
            settings.box_height - WALL_THICKNESS,
            Color::from_hex(0xe0e0e0),
            DrawTextureParams {
                rotation: PI * 2.0,
                dest_size: Some(vec2(max_axis * 2., WALL_THICKNESS)),
                ..Default::default()
            },
        );

        let wall_velocity = delta_pos / get_frame_time();

        let mut remaining_dt = get_frame_time();

        let mut steps = 0;
        let mut wall_hits = [0, 0];

        while remaining_dt > 0.00001 && steps < 10 {
            steps += 1;
            remaining_dt = ball.step(
                remaining_dt,
                &settings,
                wall_velocity,
                maxed_delta,
                &mut wall_hits,
            );
        }

        ball.render(&settings);

        let old_ui = false;

        let save = if !old_ui {
            ui_renderer.render_ui(
                &mut editing_settings,
                &mut settings_state,
                mouse_pos,
                (settings.box_width, settings.box_height),
            )
        } else {
            render_ui(
                &mut editing_settings,
                &mut settings_state,
                (settings.box_width, settings.box_height),
            )
        };
        if save {
            settings = editing_settings.clone();
            write_settings_file(&settings);
            for sound in ball.sounds.iter() {
                set_sound_volume(sound, settings.audio_volume);
            }
        }

        next_frame().await
    }
}
