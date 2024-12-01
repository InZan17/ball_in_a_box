use core::str;
use std::{
    f32::consts::PI,
    fs::OpenOptions,
    io::Write,
    panic,
    time::{SystemTime, UNIX_EPOCH},
};

use ball::Ball;
use conf::Icon;
use loop_array::LoopArray;
use macroquad::{audio::set_sound_volume, prelude::*, rand};
use miniquad::*;
use settings::{read_settings_file, write_settings_file, Settings};
use sounds::{find_sounds, get_random_sounds};
use textures::{find_texture, get_random_texture};
use ui::{SettingsState, UiRenderer, MENU_SIZE};
use window::{set_window_position, set_window_size};

pub mod ball;
pub mod loop_array;
pub mod settings;
pub mod sounds;
pub mod textures;
pub mod ui;

include!(concat!(env!("OUT_DIR"), "/icon_data.rs"));

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
        icon: Some(Icon {
            small: ICON_SMALL,
            medium: ICON_MEDIUM,
            big: ICON_BIG,
        }),
        ..Default::default()
    }
}

// Whenever the delta time is less than this, it will try to smooth out the window position over the duration of MIN_DELTA_TIME.
const MIN_DELTA_TIME: f32 = 1.0 / 60.0;

pub trait FromTuple {
    fn from_u32_tuple(tuple: (u32, u32)) -> Self;
    fn from_i32_tuple(tuple: (i32, i32)) -> Self;
}

impl FromTuple for Vec2 {
    fn from_u32_tuple(tuple: (u32, u32)) -> Self {
        Vec2::new(tuple.0 as f32, tuple.1 as f32)
    }

    fn from_i32_tuple(tuple: (i32, i32)) -> Self {
        Vec2::new(tuple.0 as f32, tuple.1 as f32)
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut settings = read_settings_file().unwrap_or_else(|| {
        let settings = Settings::default();
        write_settings_file(&settings);
        settings
    });

    let mut box_size = vec2(settings.box_width as f32, settings.box_height as f32);

    let mut editing_settings = settings.clone();

    panic::set_hook(Box::new(|info| {
        let Ok(mut log_file) = OpenOptions::new()
            .create(true)
            .write(true)
            .open("crash_info.txt")
        else {
            return;
        };

        let panic_message = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.to_string()
        } else {
            "Unknown crash.".to_string()
        };

        let _ = log_file.write(panic_message.as_bytes());
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
                UniformDesc::new("ambient_occlusion_focus", UniformType::Float1),
                UniformDesc::new("ambient_occlusion_strength", UniformType::Float1),
                UniformDesc::new("ambient_light", UniformType::Float1),
                UniformDesc::new("specular_focus", UniformType::Float1),
                UniformDesc::new("specular_strength", UniformType::Float1),
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
            uniforms: vec![
                UniformDesc::new("in_shadow", UniformType::Float1),
                UniformDesc::new("shadow_strength", UniformType::Float1),
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
    .expect("Failed to load shadow material");

    drop(default_vert);

    let max_string_len = 100;

    let mut text_input = String::new();

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
            settings.ball_radius as f32,
            sounds.1,
        )
    };

    set_camera(&Camera2D {
        zoom: vec2(1. / box_size.x, 1. / box_size.y),
        ..Default::default()
    });

    let mut box_deltas: LoopArray<(f32, Vec2), 10> = LoopArray::new();

    loop {
        clear_background(DARKGRAY);

        let delta_time = get_frame_time();

        let box_thickness = settings.box_thickness as f32;

        if is_key_pressed(KeyCode::Escape) || is_mouse_button_pressed(MouseButton::Right) {
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
            Vec2::from(mouse_position()) * screen_dpi_scale()
        };

        while let Some(character) = get_char_pressed() {
            if character.is_control() {
                continue;
            }
            ui_renderer.user_input.push(character);
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
        if is_key_pressed(KeyCode::Backspace) {
            ui_renderer.user_input.pop();
        }

        if text_input.len() > max_string_len {
            let remove = text_input.len() - max_string_len;
            text_input = text_input[remove..].to_string();
        }

        let current_mouse_position = Vec2::from_i32_tuple(window::get_screen_mouse_position());
        let delta_mouse_position = current_mouse_position - last_mouse_position;
        last_mouse_position = current_mouse_position;

        if is_mouse_button_pressed(MouseButton::Left) && is_menu_open {
            let abs_mouse_pos_from_center = (mouse_pos - box_size / 2.).abs();
            if abs_mouse_pos_from_center.x < MENU_SIZE.x / 2. * ui_renderer.mult
                && abs_mouse_pos_from_center.y < MENU_SIZE.y / 2. * ui_renderer.mult
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

        let mut restricted_delta_pos = if delta_time < MIN_DELTA_TIME && delta_pos != Vec2::ZERO {
            box_deltas.push((MIN_DELTA_TIME, delta_pos));
            Vec2::ZERO
        } else {
            delta_pos
        };

        let mut discard_amount = 0;

        for i in 0..box_deltas.len() {
            let (time_left, smear_delta_pos) = box_deltas.get_mut(i);
            if *time_left <= delta_time {
                let pct = *time_left / MIN_DELTA_TIME;
                restricted_delta_pos += *smear_delta_pos * pct;
                discard_amount += 1;
            } else {
                let pct = delta_time / MIN_DELTA_TIME;
                restricted_delta_pos += *smear_delta_pos * pct;
                *time_left -= delta_time;
            }
        }

        box_deltas.remove_amount(discard_amount);

        smoothed_delta = smoothed_delta.lerp(restricted_delta_pos, 0.5);
        smoothed_magnitude = smoothed_magnitude
            .lerp(smoothed_delta.length(), 0.15)
            .min(smoothed_delta.length());

        let smoothed_delta = if smoothed_delta.length() != 0. {
            smoothed_delta.normalize() * smoothed_magnitude
        } else {
            smoothed_delta
        };

        let maxed_delta = vec2(
            if smoothed_delta.x.abs() > restricted_delta_pos.x.abs() {
                smoothed_delta.x
            } else {
                restricted_delta_pos.x
            },
            if smoothed_delta.y.abs() > restricted_delta_pos.y.abs() {
                smoothed_delta.y
            } else {
                restricted_delta_pos.y
            },
        );

        let smoothed_wall_velocity = maxed_delta / delta_time * 2.;

        draw_texture_ex(
            &background_texture,
            -box_size.x + box_thickness,
            -box_size.y + box_thickness,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    (box_size.x - box_thickness) * 2.,
                    (box_size.y - box_thickness) * 2.,
                )),
                ..Default::default()
            },
        );

        let max_axis = box_size.max_element();

        // Left
        draw_texture_ex(
            &side_texture,
            -box_size.x - max_axis + box_thickness / 2.,
            0.,
            Color::from_hex(0x999999),
            DrawTextureParams {
                rotation: PI * 0.5,
                dest_size: Some(vec2(max_axis * 2., box_thickness)),
                ..Default::default()
            },
        );

        // Right
        draw_texture_ex(
            &side_texture,
            -box_thickness / 2. - max_axis + box_size.x,
            0.,
            Color::from_hex(0xb0b0b0),
            DrawTextureParams {
                rotation: PI * 1.5,
                dest_size: Some(vec2(max_axis * 2., box_thickness)),
                ..Default::default()
            },
        );

        // Top
        draw_texture_ex(
            &side_texture,
            -box_size.x,
            -box_size.y,
            Color::from_hex(0xbababa),
            DrawTextureParams {
                rotation: PI * 1.0,
                dest_size: Some(vec2(max_axis * 2., box_thickness)),
                ..Default::default()
            },
        );

        // Bottom
        draw_texture_ex(
            &side_texture,
            -box_size.x,
            box_size.y - box_thickness,
            Color::from_hex(0xe0e0e0),
            DrawTextureParams {
                rotation: PI * 2.0,
                dest_size: Some(vec2(max_axis * 2., box_thickness)),
                ..Default::default()
            },
        );

        let wall_velocity = delta_pos / delta_time;

        let mut remaining_dt = delta_time;

        let mut steps = 0;
        let mut wall_hits = [0, 0];

        while remaining_dt > 0.00001 && steps < 10 {
            steps += 1;
            remaining_dt = ball.step(
                remaining_dt,
                &settings,
                wall_velocity,
                smoothed_wall_velocity,
                &mut wall_hits,
                box_size,
            );
        }

        ball.render(&settings, box_size);

        let save = ui_renderer.render_ui(
            &mut editing_settings,
            &settings,
            &mut settings_state,
            mouse_pos,
            box_size,
        );

        if save {
            settings = editing_settings.clone();
            write_settings_file(&settings);
            for sound in ball.sounds.iter() {
                set_sound_volume(sound, settings.audio_volume);
            }
            ball.radius = settings.ball_radius as f32;
            set_window_size(settings.box_width, settings.box_height);
            box_size = vec2(settings.box_width as f32, settings.box_height as f32);
            set_camera(&Camera2D {
                zoom: vec2(1. / box_size.x, 1. / box_size.y),
                ..Default::default()
            });
        }

        next_frame().await
    }
}
