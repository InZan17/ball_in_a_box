use core::str;
use std::{
    f32::consts::{E, PI},
    fs,
};

use macroquad::{
    audio::{load_sound_from_bytes, play_sound, PlaySoundParams},
    prelude::*,
    time,
    ui::{hash, root_ui, widgets, Skin},
};
use miniquad::*;
use nanoserde::{DeJson, SerJson};
use window::{order_quit, set_window_position};

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

#[derive(Debug, SerJson, DeJson, Clone)]
struct Settings {
    gravity_strength: f32,
    air_friction: f32,
    bounciness: f32,
    terminal_velocity: f32,
    ball_radius: f32,
    audio_volume: f32,
    shadow_size: f32,
    shadow_distance_strength: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            gravity_strength: 3.,
            air_friction: 0.17,
            bounciness: 0.9,
            terminal_velocity: 100.,
            ball_radius: 90.,
            audio_volume: 0.75,
            shadow_size: 1.2,
            shadow_distance_strength: 50.,
        }
    }
}

pub fn calculate_bounce_spin(
    ball_velocity: f32,
    window_velocity: f32,
    ball_rotation_velocity: f32,
    mut ball_radius: f32,
    inverted: bool,
) -> (f32, f32) {
    ball_radius = ball_radius.max(0.001);

    let total_velocity = if inverted {
        -(ball_velocity + window_velocity)
    } else {
        ball_velocity + window_velocity
    };
    let rotation_velocity_from_velocity = total_velocity / ball_radius;
    let middle_rotation_velocity =
        rotation_velocity_from_velocity.lerp(ball_rotation_velocity, 0.5);
    let current_rotation_direction_velocity = if inverted {
        -middle_rotation_velocity * ball_radius
    } else {
        middle_rotation_velocity * ball_radius
    };
    let new_rotation_velocity = ball_rotation_velocity.lerp(rotation_velocity_from_velocity, 0.75);
    return (
        new_rotation_velocity,
        current_rotation_direction_velocity - window_velocity,
    );
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

    let wall_thickness = 20.;
    let wall_depth = 20.;

    let mut is_menu_open = false;
    let mut is_in_settings = false;
    let last_page = 1;
    let mut settings_page = 0_u8;
    let mut close_menu = false;
    let mut interacting_with_ui = false;

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

    let bonk_sounds = [
        load_sound_from_bytes(include_bytes!("../assets/bonk2.wav"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/bonk3.wav"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/bonk4.wav"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/bonk5.wav"))
            .await
            .unwrap(),
        load_sound_from_bytes(include_bytes!("../assets/bonk6.wav"))
            .await
            .unwrap(),
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

    let window_style = root_ui()
        .style_builder()
        .background(
            Image::from_file_with_format(include_bytes!("../assets/main_background.png"), None)
                .unwrap(),
        )
        //.background_margin(RectOffset::new(20.0, 20.0, 10.0, 10.0))
        //.margin(RectOffset::new(-20.0, -30.0, 0.0, 0.0))
        .build();

    let button_style = root_ui()
        .style_builder()
        .background(
            Image::from_file_with_format(include_bytes!("../assets/cardboard_button.png"), None)
                .unwrap(),
        )
        .font(include_bytes!("../assets/FrederickatheGreat-Regular.ttf"))
        .unwrap()
        .font_size(28)
        .text_color(Color::new(0.05, 0., 0.1, 1.))
        .color_hovered(Color::new(0.90, 0.90, 0.90, 1.0))
        .build();

    let label_style = root_ui()
        .style_builder()
        .font(include_bytes!("../assets/FrederickatheGreat-Regular.ttf"))
        .unwrap()
        .font_size(24)
        .text_color(Color::new(0.05, 0., 0.1, 1.))
        .margin(RectOffset::new(0., 0., 10., 0.))
        .build();

    let editbox_style = root_ui()
        .style_builder()
        .font(include_bytes!("../assets/FrederickatheGreat-Regular.ttf"))
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
        //tabbar_style,
        //scrollbar_handle_style,
        //scrollbar_style,
        //combobox_style,
        editbox_style,
        //window_titlebar_style,
        checkbox_style,
        group_style,
        ..root_ui().default_skin()
    };

    root_ui().push_skin(&skin);

    let mut last_mouse_position = Vec2::from_i32_tuple(window::get_screen_mouse_position());

    let mut mouse_offset = Vec2::ZERO;

    let mut smoothed_delta = Vec2::ZERO;
    let mut smoothed_magnitude = 0.;

    let mut ball_position = Vec2::ZERO;
    let mut ball_velocity = Vec2::ZERO;
    let mut ball_rotation = 0.;
    let mut ball_rotation_velocity = 0.;

    let mut hit_wall_speed: f32;
    let mut previous_hit_wall_speed = 0.;

    loop {
        hit_wall_speed = 0.;
        if is_key_pressed(KeyCode::Escape) {
            if is_menu_open {
                is_menu_open = false;
                is_in_settings = false;
            } else {
                is_menu_open = true;
            }
        }

        const MENU_SIZE: Vec2 = vec2(310., 400.);
        const BUTTON_SIZE: Vec2 = vec2(160., 75.);
        const BUTTONS_MARGIN: f32 = 20.;
        if is_menu_open {
            if is_in_settings {
                const MENU_PADDING: f32 = 10.;
                const SMALL_BUTTON_DIV: f32 = 1.5;
                const SMALLER_BUTTON_DIV: f32 = 1.75;
                root_ui().window(
                    hash!(),
                    vec2(WIDTH_F - MENU_SIZE.x, HEIGHT_F - MENU_SIZE.y) / 2.,
                    MENU_SIZE,
                    |ui| {
                        let mut top_position = vec2(
                            (MENU_SIZE.x - BUTTON_SIZE.x) / 2.,
                            MENU_PADDING + BUTTONS_MARGIN,
                        );

                        let last_settings_page = settings_page;

                        if last_settings_page > 0 {
                            if widgets::Button::new("Prev")
                                .position(vec2(
                                    top_position.x + BUTTON_SIZE.x / 2.
                                        - BUTTON_SIZE.x / SMALLER_BUTTON_DIV
                                        - BUTTONS_MARGIN / 2.,
                                    top_position.y,
                                ))
                                .size(BUTTON_SIZE / SMALLER_BUTTON_DIV)
                                .ui(ui)
                            {
                                settings_page -= 1;
                            }
                        }

                        if last_settings_page < last_page {
                            if widgets::Button::new("Next")
                                .position(vec2(
                                    top_position.x + BUTTON_SIZE.x / 2. + BUTTONS_MARGIN / 2.,
                                    top_position.y,
                                ))
                                .size(BUTTON_SIZE / SMALLER_BUTTON_DIV)
                                .ui(ui)
                            {
                                settings_page += 1;
                            }
                        }

                        const GROUP_OFFSET: Vec2 = vec2(50., 30.);

                        let group = widgets::Group::new(
                            hash!(),
                            MENU_SIZE - GROUP_OFFSET
                                + vec2(40., -BUTTON_SIZE.y / SMALLER_BUTTON_DIV),
                        )
                        .position(GROUP_OFFSET + vec2(0., BUTTON_SIZE.y / SMALLER_BUTTON_DIV))
                        .begin(ui);

                        match last_settings_page {
                            0 => {
                                widgets::Label::new("Audio volume").ui(ui);

                                widgets::Slider::new(hash!(), 0.0..1.0)
                                    .ui(ui, &mut editing_settings.audio_volume);

                                widgets::Label::new("Bounciness").ui(ui);

                                widgets::Slider::new(hash!(), 0.0..1.0)
                                    .ui(ui, &mut editing_settings.bounciness);

                                widgets::Label::new("Ball radius").ui(ui);

                                widgets::Slider::new(
                                    hash!(),
                                    0.0..(WIDTH_F.min(HEIGHT_F) - wall_thickness - wall_depth),
                                )
                                .ui(ui, &mut editing_settings.ball_radius);

                                widgets::Label::new("Gravity strength").ui(ui);

                                widgets::Slider::new(hash!(), -30.0..30.0)
                                    .ui(ui, &mut editing_settings.gravity_strength);
                            }
                            1 => {
                                widgets::Label::new("Air friction").ui(ui);

                                widgets::Slider::new(hash!(), 0.0..1.00)
                                    .ui(ui, &mut editing_settings.air_friction);

                                widgets::Label::new("Terminal Velocity").ui(ui);

                                widgets::Slider::new(hash!(), 0.0..500.00)
                                    .ui(ui, &mut editing_settings.terminal_velocity);
                            }
                            _ => {
                                unimplemented!()
                            }
                        }
                        group.end(ui);

                        top_position.y = MENU_SIZE.y
                            - BUTTON_SIZE.y / SMALL_BUTTON_DIV
                            - MENU_PADDING
                            - BUTTONS_MARGIN;
                        if widgets::Button::new("Back")
                            .position(vec2(
                                top_position.x + BUTTON_SIZE.x / 2.
                                    - BUTTON_SIZE.x / SMALL_BUTTON_DIV
                                    - BUTTONS_MARGIN / 2.,
                                top_position.y,
                            ))
                            .size(BUTTON_SIZE / SMALL_BUTTON_DIV)
                            .ui(ui)
                        {
                            is_in_settings = false;
                        }

                        if widgets::Button::new("Apply")
                            .position(vec2(
                                top_position.x + BUTTON_SIZE.x / 2. + BUTTONS_MARGIN / 2.,
                                top_position.y,
                            ))
                            .size(BUTTON_SIZE / SMALL_BUTTON_DIV)
                            .ui(ui)
                        {
                            settings = editing_settings.clone();
                            write_settings_file(&settings);
                        }
                    },
                );
            } else {
                const MENU_PADDING: f32 = 45.;
                root_ui().window(
                    hash!(),
                    vec2(WIDTH_F - MENU_SIZE.x, HEIGHT_F - MENU_SIZE.y) / 2.,
                    MENU_SIZE,
                    |ui| {
                        let mut button_position = vec2(
                            (MENU_SIZE.x - BUTTON_SIZE.x) / 2.,
                            MENU_PADDING + BUTTONS_MARGIN,
                        );
                        if widgets::Button::new("Continue")
                            .position(button_position)
                            .size(BUTTON_SIZE)
                            .ui(ui)
                        {
                            close_menu = true;
                        }
                        button_position.y += BUTTON_SIZE.y + BUTTONS_MARGIN;
                        if widgets::Button::new("Options")
                            .position(button_position)
                            .size(BUTTON_SIZE)
                            .ui(ui)
                        {
                            editing_settings = settings.clone();
                            is_in_settings = true;
                            settings_page = 0;
                        }
                        button_position.y += BUTTON_SIZE.y + BUTTONS_MARGIN;
                        if widgets::Button::new("Quit")
                            .position(button_position)
                            .size(BUTTON_SIZE)
                            .ui(ui)
                        {
                            order_quit();
                        }
                    },
                );
            }
        }

        let wall_offset = settings.ball_radius + wall_thickness + wall_depth;

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

        clear_background(LIGHTGRAY);

        ball_velocity += Vec2::new(0., settings.gravity_strength * 1000. * get_frame_time());

        ball_velocity *= 1. - (settings.air_friction * get_frame_time().clamp(0., 1.));

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
            hit_wall_speed = hit_wall_speed.max(smoothed_total_velocity.y.abs());
            ball_position.y = HEIGHT_F - wall_offset;
            ball_velocity.y = -smoothed_total_velocity.y * settings.bounciness;

            (ball_rotation_velocity, ball_velocity.x) = calculate_bounce_spin(
                ball_velocity.x,
                maxed_delta.x,
                ball_rotation_velocity,
                settings.ball_radius,
                false,
            );
        }

        let mut distance_to_ceiling = ball_position.y + HEIGHT_F - wall_offset;
        if distance_to_ceiling <= 0. {
            // Ceiling
            distance_to_ceiling = 0.;
            hit_wall_speed = hit_wall_speed.max(smoothed_total_velocity.y.abs());
            ball_position.y = -HEIGHT_F + wall_offset;
            ball_velocity.y = -smoothed_total_velocity.y * settings.bounciness;

            (ball_rotation_velocity, ball_velocity.x) = calculate_bounce_spin(
                ball_velocity.x,
                maxed_delta.x,
                ball_rotation_velocity,
                settings.ball_radius,
                true,
            );
        }
        let mut distance_to_right_wall = WIDTH_F - wall_offset - ball_position.x;
        if distance_to_right_wall <= 0. {
            // Right
            distance_to_right_wall = 0.;
            hit_wall_speed = hit_wall_speed.max(smoothed_total_velocity.x.abs());
            ball_position.x = WIDTH_F - wall_offset;
            ball_velocity.x = -smoothed_total_velocity.x * settings.bounciness;

            (ball_rotation_velocity, ball_velocity.y) = calculate_bounce_spin(
                ball_velocity.y,
                maxed_delta.y,
                ball_rotation_velocity,
                settings.ball_radius,
                true,
            );
        }

        let mut distance_to_left_wall = ball_position.x + WIDTH_F - wall_offset;
        if distance_to_left_wall <= 0. {
            // Left
            distance_to_left_wall = 0.;
            hit_wall_speed = hit_wall_speed.max(smoothed_total_velocity.x.abs());
            ball_position.x = -WIDTH_F + wall_offset;
            ball_velocity.x = -smoothed_total_velocity.x * settings.bounciness;

            (ball_rotation_velocity, ball_velocity.y) = calculate_bounce_spin(
                ball_velocity.y,
                maxed_delta.y,
                ball_rotation_velocity,
                settings.ball_radius,
                false,
            );
        }

        if ball_velocity.length() > settings.terminal_velocity * 1000. {
            println!("Reached terminal velocity!");
            ball_velocity = ball_velocity.normalize() * settings.terminal_velocity * 1000.;
        }

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

        shadow_material.set_uniform(
            "in_shadow",
            distance_to_floor / settings.shadow_distance_strength,
        );
        draw_rectangle(
            ball_position.x - settings.ball_radius * settings.shadow_size,
            HEIGHT_F - wall_offset + settings.ball_radius - wall_depth,
            settings.ball_radius * settings.shadow_size * 2.,
            wall_depth * 2.,
            WHITE,
        );

        shadow_material.set_uniform(
            "in_shadow",
            distance_to_ceiling / settings.shadow_distance_strength,
        );
        draw_rectangle(
            ball_position.x - settings.ball_radius * settings.shadow_size,
            -HEIGHT_F + wall_thickness,
            settings.ball_radius * settings.shadow_size * 2.,
            wall_depth * 2.,
            WHITE,
        );

        shadow_material.set_uniform(
            "in_shadow",
            distance_to_right_wall / settings.shadow_distance_strength,
        );
        draw_rectangle(
            WIDTH_F - wall_offset + settings.ball_radius - wall_depth,
            ball_position.y - settings.ball_radius * settings.shadow_size,
            wall_depth * 2.,
            settings.ball_radius * settings.shadow_size * 2.,
            WHITE,
        );

        shadow_material.set_uniform(
            "in_shadow",
            distance_to_left_wall / settings.shadow_distance_strength,
        );
        draw_rectangle(
            -WIDTH_F + wall_thickness,
            ball_position.y - settings.ball_radius * settings.shadow_size,
            wall_depth * 2.,
            settings.ball_radius * settings.shadow_size * 2.,
            WHITE,
        );

        ball_material.set_uniform("rotation", ball_rotation);
        ball_material.set_uniform(
            "floor_distance",
            distance_to_floor / settings.shadow_distance_strength,
        );
        ball_material.set_uniform(
            "ceil_distance",
            distance_to_ceiling / settings.shadow_distance_strength,
        );
        ball_material.set_uniform(
            "left_distance",
            distance_to_left_wall / settings.shadow_distance_strength,
        );
        ball_material.set_uniform(
            "right_distance",
            distance_to_right_wall / settings.shadow_distance_strength,
        );

        gl_use_material(&ball_material);

        draw_texture_ex(
            &ball_texture,
            ball_position.x - settings.ball_radius,
            ball_position.y - settings.ball_radius,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(settings.ball_radius * 2., settings.ball_radius * 2.)),
                rotation: ball_rotation,
                ..Default::default()
            },
        );

        gl_use_default_material();

        if is_menu_open {
            draw_rectangle(
                -WIDTH_F,
                -HEIGHT_F,
                WIDTH_F * 2.,
                HEIGHT_F * 2.,
                Color::from_rgba(0, 0, 0, 100),
            );
        }

        if close_menu {
            close_menu = false;
            is_menu_open = false;
        }

        const DENSITY: f32 = 0.32;

        if hit_wall_speed > 30. && previous_hit_wall_speed == 0. {
            let inverted_distances_from_corners =
                ball_position.abs() + vec2(0., WIDTH_F - HEIGHT_F);

            let distance_from_corner = WIDTH_F - inverted_distances_from_corners.min_element();
            // The closer to the center it is, the louder the sound.
            hit_wall_speed /= 450.;
            hit_wall_speed *= 1. + distance_from_corner / 200.;
            let volume = 1. - 1. / E.powf(hit_wall_speed * hit_wall_speed * DENSITY * DENSITY);
            play_sound(
                &bonk_sounds[quad_rand::gen_range(0, bonk_sounds.len())],
                PlaySoundParams {
                    looped: false,
                    volume: volume * settings.audio_volume,
                },
            );
        }

        previous_hit_wall_speed = hit_wall_speed;

        next_frame().await
    }
}
