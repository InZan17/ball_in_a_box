use core::str;
use std::{
    f32::consts::PI,
    fs::OpenOptions,
    io::Write,
    panic, thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use assets::{find_pack, GameAssets};
use ball::Ball;
use circular_buffer::CircularBuffer;
use conf::{Icon, Platform};
use macroquad::{audio::set_sound_volume, prelude::*, rand};
use miniquad::*;
use settings::{read_settings_file, write_settings_file, Settings};
use sounds::{find_sounds, get_random_sounds};
use textures::{find_texture, get_random_texture};
use ui::{SettingsState, UiRenderer, MENU_SIZE};
use window::{get_window_position, set_swap_interval, set_window_position, set_window_size};

pub mod assets;
pub mod ball;
pub mod settings;
pub mod sounds;
pub mod textures;
pub mod ui;

include!(concat!(env!("OUT_DIR"), "/icon_data.rs"));

const FPS_LIMIT: u32 = 500;

const BACKSPACES_BEFORE_MISSING: u8 = 7;

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
        platform: Platform {
            swap_interval: Some(if settings.vsync { 1 } else { 0 }),
            ..Default::default()
        },
        ..Default::default()
    }
}

// https://theswissbay.ch/pdf/Gentoomen%20Library/Game%20Development/Programming/Game%20Programming%20Gems%204.pdf
// 1.10
pub fn smooth_vec2_critically_damped(
    current: Vec2,
    new: Vec2,
    velocity: &mut Vec2,
    smoothness: f32,
    delta_time: f32,
) -> Vec2 {
    if smoothness == 0.0 {
        if delta_time != 0.0 {
            *velocity = (new - current) / delta_time;
        }
        return new;
    }

    let omega = 2.0 / smoothness;
    let x = omega * delta_time;
    let exp = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x);
    let delta_pos = current - new;
    let temp = (*velocity + omega * delta_pos) * delta_time;
    *velocity = (*velocity - omega * temp) * exp;
    return new + (delta_pos + temp) * exp;
}

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

pub fn log_panic(message: &str) {
    if let Ok(mut log_file) = OpenOptions::new()
        .create(true)
        .write(true)
        .open("crash_info.txt")
    {
        println!("aw");
        let _ = log_file.write(message.as_bytes());
    };
    panic!("{}", message)
}

#[macroquad::main(window_conf)]
async fn main() {
    {
        let start = SystemTime::now();
        let seed = start
            .duration_since(UNIX_EPOCH)
            .and_then(|duration| Ok(duration.as_nanos() as u64))
            .unwrap_or(0);

        rand::srand(seed);
    }

    let mut settings = read_settings_file().unwrap_or_else(|| {
        let settings = Settings::default();
        write_settings_file(&settings);
        settings
    });

    let missing_texture = Texture2D::from_rgba8(
        2,
        2,
        &[
            255, 0, 220, 255, 1, 0, 1, 255, 1, 0, 1, 255, 255, 0, 220, 255,
        ],
    );
    missing_texture.set_filter(macroquad::texture::FilterMode::Nearest);

    let pack_path = if let Some(last_asset_pack) = &settings.last_asset_pack {
        if let Some((_, pack_path)) = find_pack(last_asset_pack) {
            Some(pack_path)
        } else {
            None
        }
    } else {
        None
    };

    let mut game_assets = GameAssets::new(pack_path, missing_texture);

    let mut ball = {
        let option_sounds = find_sounds(&settings.last_sounds).await;

        let sounds = if let Some(sounds) = option_sounds {
            sounds
        } else {
            get_random_sounds()
                .await
                .unwrap_or_else(|| (settings.last_sounds.clone(), Vec::new()))
        };

        Ball::new(
            find_texture(&settings.last_ball)
                .unwrap_or_else(|| {
                    get_random_texture().unwrap_or_else(|| {
                        (
                            settings.last_ball.clone(),
                            game_assets.missing_texture.clone(),
                        )
                    })
                })
                .1,
            settings.ball_radius as f32,
            sounds.1,
        )
    };

    let mut box_size = vec2(settings.box_width as f32, settings.box_height as f32);

    set_camera(&Camera2D {
        zoom: vec2(1. / box_size.x, 1. / box_size.y),
        ..Default::default()
    });

    const MAX_INPUT_LEN: usize = 100;
    let mut text_input = String::with_capacity(MAX_INPUT_LEN);

    let mut ui_renderer = UiRenderer::new().await;

    let mut moved_since_right_click = false;
    let mut interacting_with_ui = false;
    let mut is_in_settings = false;
    let mut settings_state = SettingsState::Closed;

    let mut editing_settings = settings.clone();

    let mut mouse_offset: Option<Vec2> = None;
    let mut mouse_deltas: CircularBuffer<10, Vec2> = CircularBuffer::new();

    let mut old_visual_window_position = Vec2::ZERO;
    let mut old_internal_window_position = Vec2::ZERO;
    let mut window_velocity = Vec2::ZERO;

    let mut frames_after_start: u8 = 0;
    let mut prev_render_time = get_time();

    let mut times_clicked_backspace: u8 = 0;

    loop {
        clear_background(DARKGRAY);

        let delta_time;

        // First frame loads everything, second frame will have a high delta time because of loading a lot the previous frame.
        // Delay the actual delta time until after that so the user can see the ball spawn in middle and bounce.
        if frames_after_start >= 2 {
            delta_time = get_frame_time() * settings.speed_mul
        } else {
            frames_after_start += 1;
            delta_time = 0.0
        }

        let box_thickness = settings.box_thickness as f32;

        // Handle controls

        if is_mouse_button_pressed(MouseButton::Right) {
            moved_since_right_click = false;
        }

        if is_key_pressed(KeyCode::Escape)
            || (is_mouse_button_released(MouseButton::Right) && !moved_since_right_click)
        {
            if settings_state != SettingsState::Closed {
                settings_state = SettingsState::Closed
            } else {
                settings_state = SettingsState::Open
            }
        }

        if settings_state.is_settings() {
            if !is_in_settings {
                editing_settings = settings.clone();
                is_in_settings = true
            }
        } else {
            is_in_settings = false
        }

        let is_menu_open = settings_state.is_open();

        let current_mouse_position = Vec2::from_i32_tuple(window::get_screen_mouse_position());

        let local_mouse_pos = if let Some(mouse_pos) = mouse_offset {
            -mouse_pos
        } else {
            (current_mouse_position - Vec2::from_u32_tuple(get_window_position()))
                .clamp(Vec2::ZERO, box_size - 1.0)
        };

        // Handle typing
        while let Some(character) = get_char_pressed() {
            if character.is_control() {
                continue;
            }
            ui_renderer.user_input.push(character);

            if text_input.len() >= MAX_INPUT_LEN {
                text_input.remove(0);
            }

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

            if let Some((pack_name, pack_path)) = find_pack(&text_input) {
                settings.last_asset_pack = Some(pack_name.clone());
                editing_settings.last_asset_pack = Some(pack_name);
                write_settings_file(&settings);
                game_assets = GameAssets::new(Some(pack_path), game_assets.missing_texture)
            } else if text_input.ends_with("none") && settings.last_asset_pack.is_some() {
                settings.last_asset_pack = None;
                editing_settings.last_asset_pack = None;
                write_settings_file(&settings);
                game_assets = GameAssets::new(None, game_assets.missing_texture)
            }
        }
        if is_key_pressed(KeyCode::Backspace) {
            times_clicked_backspace = times_clicked_backspace.saturating_add(1);
            text_input.clear();
            if ui_renderer.user_input.pop().is_none() {
                ui_renderer.reset_field = true;
            }
        }

        // Don't move window if overlapping with menu.
        if (is_mouse_button_pressed(MouseButton::Left)
            || is_mouse_button_pressed(MouseButton::Right))
            && is_menu_open
        {
            let abs_mouse_pos_from_center = (local_mouse_pos - box_size / 2.).abs();
            if abs_mouse_pos_from_center.x < MENU_SIZE.x / 2. * ui_renderer.mult
                && abs_mouse_pos_from_center.y < MENU_SIZE.y / 2. * ui_renderer.mult
            {
                interacting_with_ui = true
            }
        }

        let button_is_down =
            is_mouse_button_down(MouseButton::Left) || is_mouse_button_down(MouseButton::Right);

        if !button_is_down {
            interacting_with_ui = false
        }

        if (!get_keys_pressed().is_empty() && !is_key_pressed(KeyCode::Backspace)) || button_is_down
        {
            times_clicked_backspace = 0
        }

        if times_clicked_backspace >= BACKSPACES_BEFORE_MISSING {
            ball.texture = game_assets.missing_texture.clone();
        }

        let mouse_offset_was_some = mouse_offset.is_some();

        // Update internal / visual window position and get delta position of window.
        let visual_delta_pos = if !interacting_with_ui && button_is_down {
            let mouse_offset = match mouse_offset {
                Some(mouse_offset) => mouse_offset,
                None => {
                    mouse_offset = Some(-local_mouse_pos);
                    window_velocity = Vec2::ZERO;
                    old_internal_window_position = current_mouse_position - local_mouse_pos;
                    old_visual_window_position = old_internal_window_position;
                    -local_mouse_pos
                }
            };

            let new_pos = current_mouse_position + mouse_offset;
            let new_internal_window_pos = smooth_vec2_critically_damped(
                old_internal_window_position,
                new_pos,
                &mut window_velocity,
                settings.box_weight,
                delta_time,
            );

            let new_visual_window_pos = if settings.hide_smoothing {
                new_pos
            } else {
                new_internal_window_pos
            };

            let visual_delta_pos = new_visual_window_pos - old_visual_window_position;

            if visual_delta_pos != Vec2::ZERO {
                moved_since_right_click = true
            }

            old_internal_window_position = new_internal_window_pos;
            old_visual_window_position = new_visual_window_pos;
            -visual_delta_pos
        } else {
            window_velocity = Vec2::ZERO;
            mouse_offset = None;
            Vec2::ZERO
        };

        // Position window and handle delay frames
        let delay_frames = settings.delay_frames as usize;
        if delay_frames != 0 {
            while mouse_deltas.len() >= delay_frames {
                mouse_deltas.pop_front();
            }
            if mouse_offset_was_some && button_is_down {
                mouse_deltas.push_back(visual_delta_pos);
            }
        }

        if !interacting_with_ui && button_is_down {
            let mut new_pos = old_visual_window_position;

            for delta in mouse_deltas.iter() {
                new_pos += *delta;
            }
            set_window_position(new_pos.x as u32, new_pos.y as u32);
        } else {
            if mouse_deltas.len() > 0 {
                mouse_deltas.push_back(Vec2::ZERO);
                let mut new_pos = old_visual_window_position;

                let mut delayed_delta_pos = Vec2::ZERO;

                for delta in mouse_deltas.iter() {
                    delayed_delta_pos += *delta;
                }

                if delayed_delta_pos == Vec2::ZERO {
                    mouse_deltas.clear();
                }

                new_pos += delayed_delta_pos;

                set_window_position(new_pos.x as u32, new_pos.y as u32);
            };
        }

        // Adjust velocity
        if settings.quick_turn {
            let offset_mouse_pos = current_mouse_position + mouse_offset.unwrap_or(Vec2::ZERO);

            if offset_mouse_pos.x > old_visual_window_position.x {
                window_velocity.x = window_velocity.x.max(0.0)
            } else if offset_mouse_pos.x < old_visual_window_position.x {
                window_velocity.x = window_velocity.x.min(0.0)
            }

            if offset_mouse_pos.y > old_visual_window_position.y {
                window_velocity.y = window_velocity.y.max(0.0)
            } else if offset_mouse_pos.y < old_visual_window_position.y {
                window_velocity.y = window_velocity.y.min(0.0)
            }
        }

        let visual_window_velocity = if delta_time == 0.0 {
            Vec2::ZERO
        } else {
            visual_delta_pos / delta_time
        };

        // Ball physics
        let mut remaining_dt = delta_time;

        let mut steps = 0;
        let mut wall_hits = [0, 0];

        while remaining_dt > 0.00001 && steps < 10 {
            steps += 1;
            remaining_dt = ball.step(
                remaining_dt,
                &settings,
                visual_window_velocity * 2.,
                -window_velocity * 2.,
                &mut wall_hits,
                box_size,
            );
        }

        // Render

        // Background
        draw_texture_ex(
            &game_assets.box_background_texture,
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
            &game_assets.box_side_texture,
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
            &game_assets.box_side_texture,
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
            &game_assets.box_side_texture,
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
            &game_assets.box_side_texture,
            -box_size.x,
            box_size.y - box_thickness,
            Color::from_hex(0xe0e0e0),
            DrawTextureParams {
                rotation: PI * 2.0,
                dest_size: Some(vec2(max_axis * 2., box_thickness)),
                ..Default::default()
            },
        );

        // Ball
        ball.render(&game_assets, &settings, box_size);

        // Settings
        let save = ui_renderer.render_ui(
            &game_assets,
            &mut editing_settings,
            &settings,
            &mut settings_state,
            local_mouse_pos,
            box_size,
        );

        if save {
            let change_ball = editing_settings.last_ball != settings.last_ball;
            let change_sounds = editing_settings.last_sounds != settings.last_sounds;
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
            set_swap_interval(if settings.vsync { 1 } else { 0 });
            if change_ball {
                if let Some((_, texture)) = find_texture(&settings.last_ball) {
                    ball.texture = texture
                }
            }

            if change_sounds {
                if let Some((_, sounds)) = find_sounds(&settings.last_sounds).await {
                    ball.sounds = sounds;
                }
            }
        }

        if settings.max_fps < FPS_LIMIT {
            let min_fps_delta = 1. / settings.max_fps as f64;

            let time_now = get_time();

            let time_difference = time_now - prev_render_time;

            if time_difference < min_fps_delta {
                let duration = min_fps_delta - time_difference;
                thread::sleep(Duration::from_secs_f64(duration));
                prev_render_time = time_now + duration;
            } else {
                let offset = (time_difference - min_fps_delta) % min_fps_delta;
                prev_render_time = time_now - offset;
            }
        } else {
            prev_render_time = get_time();
        }

        next_frame().await
    }
}
