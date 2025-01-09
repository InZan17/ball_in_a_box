#![cfg_attr(
    all(
        target_os = "windows",
        not(debug_assertions),
    ),
    windows_subsystem = "windows"
)]
  
use std::{
    f32::consts::PI,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use assets::{find_pack, GameAssets};
use ball::Ball;
use circular_buffer::CircularBuffer;
use conf::{Icon, Platform};
use error_log::ErrorLogs;
use macroquad::{audio::set_sound_volume, prelude::*, rand};
use miniquad::*;
use settings::{read_settings_file, write_settings_file, Settings};
use sounds::{find_sounds, get_random_sounds};
use textures::{find_texture, get_random_texture};
use tutorial::{render_menu_tutorial, render_mouse_tutorial};
use ui::{SettingsState, UiRenderer, MENU_SIZE};
use window::{
    get_window_position, set_mouse_cursor, set_swap_interval, set_window_position, set_window_size,
};

pub mod assets;
pub mod ball;
pub mod error_log;
pub mod settings;
pub mod sounds;
pub mod textures;
pub mod tutorial;
pub mod ui;

include!(concat!(env!("OUT_DIR"), "/icon_data.rs"));

const FPS_LIMIT: u32 = 500;

const BACKSPACES_BEFORE_MISSING: u8 = 7;

const MOUSE_TUTORIAL_WAIT: f32 = 7.25;
const WINDOW_DISTANCE_BEFORE_UNDERSTAND: f32 = 100.0;

const MENU_TUTORIAL_WAIT: f32 = 7.;

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
    fn from_i32_tuple(tuple: (i32, i32)) -> Self;
}

impl FromTuple for Vec2 {
    fn from_i32_tuple(tuple: (i32, i32)) -> Self {
        Vec2::new(tuple.0 as f32, tuple.1 as f32)
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    {
        let start = SystemTime::now();
        let seed = start
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|err| err.duration())
            .as_nanos() as u64;

        rand::srand(seed);
    }

    let mut error_logs = ErrorLogs::new();

    let mut settings = read_settings_file().unwrap_or_else(|| {
        let settings = Settings::default();
        write_settings_file(&settings);
        settings
    });

    let missing_texture = Texture2D::from_rgba8(
        2,
        2,
        &[
            255, 0, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 0, 255, 255,
        ],
    );
    missing_texture.set_filter(macroquad::texture::FilterMode::Nearest);

    let pack_path = if !settings.last_asset_pack.is_empty() {
        if let Some((_, pack_path)) = find_pack(&settings.last_asset_pack, &mut error_logs) {
            Some(pack_path)
        } else {
            None
        }
    } else {
        None
    };

    let mut game_assets = GameAssets::new(pack_path, missing_texture, &mut error_logs);

    let mut ball = {
        let option_sounds = find_sounds(&settings.last_sounds, &mut error_logs).await;

        let sounds = if let Some(sounds) = option_sounds {
            sounds
        } else {
            get_random_sounds(&mut error_logs)
                .await
                .unwrap_or_else(|| (settings.last_sounds.clone(), Vec::new()))
        };

        Ball::new(
            find_texture(&settings.last_ball, &mut error_logs)
                .unwrap_or_else(|| {
                    get_random_texture(&mut error_logs).unwrap_or_else(|| {
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

    let mut do_drag = false;
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
    let mut time_since_start = 0.;

    let mut total_window_distance = 0.;
    let mut time_of_understanding_move = if settings.understands_moving {
        Some(0.0)
    } else {
        None
    };

    let mut times_clicked_backspace: u8 = 0;

    let mut last_left_button_is_down = false;
    let mut last_right_button_is_down = false;
    let mut last_click = 0.0;

    let mut clicked_mouse_position = Vec2::ZERO;
    let mut moved_during_hold = false;

    loop {
        clear_background(DARKGRAY);

        let delta_time;
        let real_delta_time = get_frame_time();

        // First frame loads everything, second frame will have a high delta time because of loading a lot the previous frame.
        // Delay the actual delta time until after that so the user can see the ball spawn in middle and bounce.
        if frames_after_start >= 2 {
            delta_time = real_delta_time * settings.speed_mul
        } else {
            frames_after_start += 1;
            delta_time = 0.0
        }

        time_since_start += delta_time;

        let box_thickness = settings.box_thickness as f32;

        // Handle controls

        let left_button_is_down = is_mouse_button_down(MouseButton::Left);
        let right_button_is_down = is_mouse_button_down(MouseButton::Right);

        // When the user clicks on the UI and makes the mouse exit the screen, it will still think its being presed.
        // When the user clicks again on a valid spot, it still thinks it's from the click on the UI and doesn't move the window.
        // This fixes that.
        if last_left_button_is_down && is_mouse_button_pressed(MouseButton::Left) {
            last_left_button_is_down = false;
        }

        if last_right_button_is_down && is_mouse_button_pressed(MouseButton::Right) {
            last_right_button_is_down = false;
        }

        let last_button_is_down = last_left_button_is_down || last_right_button_is_down;

        let button_is_down = left_button_is_down || right_button_is_down;

        let button_pressed = !last_button_is_down && button_is_down;
        let button_released = last_button_is_down && !button_is_down;

        last_left_button_is_down = left_button_is_down;
        last_right_button_is_down = right_button_is_down;

        let open_menu = button_pressed && last_click > 0.0 || is_key_pressed(KeyCode::Escape);

        let current_mouse_position = Vec2::from_i32_tuple(window::get_screen_mouse_position());

        if button_pressed {
            last_click = 0.4;
            clicked_mouse_position = current_mouse_position;
            if !do_drag {
                moved_during_hold = false;
            } else {
                moved_during_hold = true;
            }
        } else {
            last_click -= real_delta_time;
        }

        if !settings.click_to_drag {
            // Quick way to disable the click to drag feature, since click to drag only works when you click while not moving.
            moved_during_hold = true;
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

        let delta_clicked_mouse_pos = clicked_mouse_position - current_mouse_position;

        const MOUSE_MOVEMENT_LEEWAY: f32 = 2.0;
        if delta_clicked_mouse_pos.length() > MOUSE_MOVEMENT_LEEWAY {
            last_click = 0.0;
            if button_is_down {
                moved_during_hold = true;
            }
        }

        let local_mouse_pos = if let Some(mouse_pos) = mouse_offset {
            -mouse_pos
        } else {
            (current_mouse_position - Vec2::from_i32_tuple(get_window_position()))
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

            if let Some((ball_name, texture)) = find_texture(&text_input, &mut error_logs) {
                ball.texture = texture;
                settings.last_ball = ball_name.clone();
                editing_settings.last_ball = ball_name;
                write_settings_file(&settings);
            }

            if let Some((sounds_name, sounds)) = find_sounds(&text_input, &mut error_logs).await {
                ball.sounds = sounds.clone();
                settings.last_sounds = sounds_name.clone();
                editing_settings.last_sounds = sounds_name;
                write_settings_file(&settings);
            }

            if let Some((pack_name, pack_path)) = find_pack(&text_input, &mut error_logs) {
                settings.last_asset_pack = pack_name.clone();
                editing_settings.last_asset_pack = pack_name;
                write_settings_file(&settings);
                game_assets = GameAssets::new(
                    Some(pack_path),
                    game_assets.missing_texture,
                    &mut error_logs,
                )
            } else if (text_input.ends_with("none") || text_input.ends_with("box")) && !settings.last_asset_pack.is_empty() {
                settings.last_asset_pack = String::new();
                editing_settings.last_asset_pack = String::new();
                write_settings_file(&settings);
                game_assets = GameAssets::new(None, game_assets.missing_texture, &mut error_logs)
            }
        }
        if is_key_pressed(KeyCode::Backspace) {
            times_clicked_backspace = times_clicked_backspace.saturating_add(1);
            text_input.clear();
            if ui_renderer.user_input.pop().is_none() {
                ui_renderer.reset_field = true;
            }
        }

        let hovering_menu = {
            let abs_mouse_pos_from_center = (local_mouse_pos - box_size / 2.).abs();
            abs_mouse_pos_from_center.x < MENU_SIZE.x / 2. * ui_renderer.mult
                && abs_mouse_pos_from_center.y < MENU_SIZE.y / 2. * ui_renderer.mult
        };

        // Don't move window if overlapping with menu.
        if button_pressed && (!is_menu_open || !hovering_menu) {
            do_drag = true
        } else if button_released && moved_during_hold {
            do_drag = false
        }

        if (!get_keys_pressed().is_empty() && !is_key_pressed(KeyCode::Backspace)) || do_drag {
            times_clicked_backspace = 0
        }

        if times_clicked_backspace >= BACKSPACES_BEFORE_MISSING {
            ball.texture = game_assets.missing_texture.clone();
        }

        let mouse_offset_was_some = mouse_offset.is_some();

        // Update internal / visual window position and get delta position of window.
        let visual_delta_pos = if do_drag {
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
            if mouse_offset_was_some && do_drag {
                mouse_deltas.push_back(visual_delta_pos);
            }
        }

        if do_drag {
            let mut new_pos = old_visual_window_position;

            for delta in mouse_deltas.iter() {
                new_pos += *delta;
            }
            set_window_position(new_pos.x as i32, new_pos.y as i32);
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

                set_window_position(new_pos.x as i32, new_pos.y as i32);
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

        // Update distance and check if it has traveled far enough for the person to understand the tutorial.
        // This will fail if the person accidentally does a "click-to-drag" and is confused as to why the window is now following the cursor.
        // Idk how I would go about detecting that tho.
        total_window_distance += visual_delta_pos.length();

        if time_of_understanding_move.is_none()
            && total_window_distance > WINDOW_DISTANCE_BEFORE_UNDERSTAND
        {
            settings.understands_moving = true;
            editing_settings.understands_moving = true;
            write_settings_file(&settings);
            time_of_understanding_move = Some(time_since_start);
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

        if hovering_menu && settings_state.is_open() {
            set_mouse_cursor(CursorIcon::Default);
        } else if do_drag {
            set_mouse_cursor(CursorIcon::Move);
        } else {
            set_mouse_cursor(CursorIcon::Pointer);
        }

        // Tutorial
        if time_since_start > MOUSE_TUTORIAL_WAIT {
            render_mouse_tutorial(
                &game_assets,
                time_since_start - MOUSE_TUTORIAL_WAIT,
                time_of_understanding_move.and_then(|time| Some(time - MOUSE_TUTORIAL_WAIT)),
                box_size,
            );
        }

        if !settings.understands_menu {
            if let Some(time_of_understanding_move) = time_of_understanding_move {
                if time_since_start - time_of_understanding_move > MENU_TUTORIAL_WAIT {
                    render_menu_tutorial(
                        &game_assets,
                        time_since_start - time_of_understanding_move - MENU_TUTORIAL_WAIT,
                    )
                }
            }
        }

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
            let change_assets = editing_settings.last_asset_pack != settings.last_asset_pack;
            settings = editing_settings.clone();
            write_settings_file(&settings);
            for sound in ball.sounds.iter() {
                set_sound_volume(sound, settings.audio_volume);
            }
            ball.radius = settings.ball_radius as f32;
            let new_box_size = vec2(settings.box_width as f32, settings.box_height as f32);
            let box_size_difference = new_box_size - box_size;
            let new_window_position =
                Vec2::from_i32_tuple(get_window_position()) - (box_size_difference / 2.).round();

            let window_rect = Rect::new(
                new_window_position.x,
                new_window_position.y,
                new_box_size.x,
                new_box_size.y,
            );

            let window_position_offset = vec2(
                (current_mouse_position.x - window_rect.left() - 1.0).min(0.0)
                    + (current_mouse_position.x - window_rect.right() + 1.0).max(0.0),
                (current_mouse_position.y - window_rect.top() - 1.0).min(0.0)
                    + (current_mouse_position.y - window_rect.bottom() + 1.0).max(0.0),
            );

            let new_window_position = new_window_position + window_position_offset;

            set_window_position(new_window_position.x as _, new_window_position.y as _);
            set_window_size(settings.box_width, settings.box_height);

            box_size = new_box_size;

            set_camera(&Camera2D {
                zoom: vec2(1. / box_size.x, 1. / box_size.y),
                ..Default::default()
            });
            set_swap_interval(if settings.vsync { 1 } else { 0 });
            if change_ball {
                if let Some((_, texture)) = find_texture(&settings.last_ball, &mut error_logs) {
                    ball.texture = texture
                }
            }

            if change_sounds {
                if let Some((_, sounds)) = find_sounds(&settings.last_sounds, &mut error_logs).await
                {
                    ball.sounds = sounds;
                }
            }

            if change_assets {
                let pack_path = if !settings.last_asset_pack.is_empty() {
                    if let Some((_, pack_path)) =
                        find_pack(&settings.last_asset_pack, &mut error_logs)
                    {
                        Some(pack_path)
                    } else {
                        None
                    }
                } else {
                    None
                };

                game_assets =
                    GameAssets::new(pack_path, game_assets.missing_texture, &mut error_logs)
            }
        }

        let ui_interacted = ui_renderer.did_interact();

        // The reason we open it at the end of everything is so that if someone double clicks to open the menu, they wont accidentally click a button.
        if ui_interacted {
            last_click = 0.0;
        } else if open_menu {
            let activated_with_double_click = button_pressed;

            last_click = 0.0;
            if settings_state != SettingsState::Closed {
                settings_state = SettingsState::Closed;

                if activated_with_double_click {
                    // When double clicking to close, it may end up being in drag mode, which feels a bit weird.
                    moved_during_hold = true;
                    do_drag = true;
                }
            } else {
                if !settings.understands_menu {
                    settings.understands_menu = true;
                    editing_settings.understands_menu = true;
                    write_settings_file(&settings);
                }
                settings_state = SettingsState::Open;
                ui_renderer.reset_focused();

                if hovering_menu {
                    do_drag = false;
                }

                if activated_with_double_click {
                    // Even when the mouse is in a valid spot to drag, it feels a bit weird for the mouse to still be dragging when opening the menu.
                    moved_during_hold = true;
                    do_drag = true;
                }
            }
        }

        error_logs.render_errors(-box_size, box_size.x * 2.);

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
