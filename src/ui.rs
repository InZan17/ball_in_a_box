use std::ops::Range;

use macroquad::{prelude::*, ui::hash};
use miniquad::*;
use window::{order_quit, set_mouse_cursor};

use crate::{assets::GameAssets, settings, Settings, FPS_LIMIT};

const RELATIVE_BOX_SIZE: Vec2 = vec2(372., 450.);

pub const MENU_SIZE: Vec2 = vec2(310., 400.);
const BUTTON_SIZE: Vec2 = vec2(160., 75.);
const BUTTONS_MARGIN: f32 = 20.;

const MENU_PADDING: f32 = 10.;
const SMALL_BUTTON_DIV: f32 = 1.5;
const SMALLER_BUTTON_DIV: f32 = 1.75;

const DEFAULT_TEXT_COLOR: Color = Color::new(0.05, 0., 0.1, 1.);
const ACTIVE_TEXT_COLOR: Color = Color::new(0.3, 0., 0.6, 1.);
const CHANGED_TEXT_COLOR: Color = Color::new(0.2, 0., 0.4, 1.);
const DARKRED_TEXT_COLOR: Color = Color::new(0.3, 0., 0.0, 1.);

#[derive(Debug, PartialEq, Clone)]
pub enum SettingsState {
    Closed,
    Open,
    Settings,
    Audio(u8),
    Visuals(u8),
    Box(u8),
    Physics(u8),
    FpsDelay(u8),
    Misc(u8),
}

impl SettingsState {
    pub fn is_open(&self) -> bool {
        match self {
            SettingsState::Closed => false,
            _ => true,
        }
    }
    pub fn is_settings(&self) -> bool {
        match self {
            SettingsState::Closed | SettingsState::Open => false,
            _ => true,
        }
    }
    // Returns the current page and the last available page index.
    pub fn get_page_info_mut(&mut self) -> Option<(&mut u8, u8)> {
        match self {
            SettingsState::Audio(page) => Some((page, 0)),
            SettingsState::Visuals(page) => Some((page, 1)),
            SettingsState::Box(page) => Some((page, 1)),
            SettingsState::Physics(page) => Some((page, 1)),
            SettingsState::FpsDelay(page) => Some((page, 0)),
            SettingsState::Misc(page) => Some((page, 0)),
            _ => None,
        }
    }

    // Returns the current page and the last available page index.
    pub fn back(&mut self) {
        match self {
            SettingsState::Settings => *self = SettingsState::Open,
            SettingsState::Open | SettingsState::Closed => *self = SettingsState::Closed,
            _ => *self = SettingsState::Settings,
        }
    }
}

pub struct UiRenderer {
    pub user_input: String,
    pub mult: f32,
    pub reset_field: bool,
    default_settings: Settings,
    slider_follow: bool,
    active_id: u64,
}

pub fn get_changed_color(changed: bool) -> Color {
    if changed {
        CHANGED_TEXT_COLOR
    } else {
        BLACK
    }
}

pub fn get_changed_default_color(changed: bool) -> Color {
    if changed {
        CHANGED_TEXT_COLOR
    } else {
        DEFAULT_TEXT_COLOR
    }
}

impl UiRenderer {
    pub async fn new() -> Self {
        Self {
            user_input: String::new(),
            mult: 1.,
            slider_follow: false,
            reset_field: false,
            default_settings: Settings::default(),
            active_id: 0,
        }
    }

    pub fn render_ui(
        &mut self,
        game_assets: &GameAssets,
        editing_settings: &mut Settings,
        current_settings: &Settings,
        settings_state: &mut SettingsState,
        mouse_pos: Vec2,
        box_size: Vec2,
    ) -> bool {
        set_mouse_cursor(CursorIcon::Default);
        if *settings_state == SettingsState::Closed {
            return false;
        }

        let mult = box_size / RELATIVE_BOX_SIZE;
        self.mult = mult.min_element();

        let mouse_pos = mouse_pos * 2. - box_size;

        draw_rectangle(
            -box_size.x,
            -box_size.y,
            box_size.x * 2.,
            box_size.y * 2.,
            Color::from_rgba(0, 0, 0, 100),
        );

        let mut save = false;

        let menu_position = -MENU_SIZE;

        let menu_rect = Rect::new(
            menu_position.x * self.mult,
            menu_position.y * self.mult,
            MENU_SIZE.x * 2. * self.mult,
            MENU_SIZE.y * 2. * self.mult,
        );

        draw_texture_ex(
            &game_assets.menu_background,
            menu_rect.x,
            menu_rect.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(menu_rect.w, menu_rect.h)),
                ..Default::default()
            },
        );

        if settings_state.is_settings() {
            const SLIDER_HEIGHT: f32 = 24.;
            const SLIDER_WIDTH: f32 = MENU_SIZE.x * 0.65;
            const TITLE_SIZE: u16 = 24;
            const OPTIONS_SPACING: f32 = 13.;

            let lower_down = SLIDER_HEIGHT + TITLE_SIZE as f32 + OPTIONS_SPACING;

            let center_offset_x =
                -MENU_SIZE.x / 2. + BUTTON_SIZE.x / SMALLER_BUTTON_DIV + BUTTONS_MARGIN / 2. - 4.;

            let y_offset = -MENU_SIZE.y / 2.
                + MENU_PADDING
                + BUTTONS_MARGIN
                + BUTTON_SIZE.y / SMALLER_BUTTON_DIV / 2.;

            let start =
                -MENU_SIZE.y / 2. + 5. + BUTTON_SIZE.y / SMALLER_BUTTON_DIV * 2. + SLIDER_HEIGHT;

            if let Some((page, last_page)) = settings_state.get_page_info_mut() {
                if last_page != 0 {
                    self.render_text(
                        &game_assets,
                        vec2(0., y_offset - 4.),
                        vec2(10., 10.),
                        &format!("{}", *page + 1),
                        28,
                    );

                    if *page > 0 {
                        if self.render_button(
                            game_assets,
                            hash!(),
                            mouse_pos,
                            vec2(center_offset_x, y_offset),
                            BUTTON_SIZE / SMALLER_BUTTON_DIV,
                            "Prev",
                            DEFAULT_TEXT_COLOR,
                            28,
                        ) {
                            *page -= 1;
                        }
                    }

                    if *page < last_page {
                        if self.render_button(
                            game_assets,
                            hash!(),
                            mouse_pos,
                            vec2(-center_offset_x, y_offset),
                            BUTTON_SIZE / SMALLER_BUTTON_DIV,
                            "Next",
                            DEFAULT_TEXT_COLOR,
                            28,
                        ) {
                            *page += 1;
                        }
                    }
                }

                match settings_state {
                    SettingsState::Audio(page) => match *page {
                        0 => {
                            self.render_slider(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., start + lower_down * 0.3),
                                vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                                "Audio volume",
                                TITLE_SIZE,
                                0.0..1.0,
                                self.default_settings.audio_volume,
                                current_settings.audio_volume,
                                &mut editing_settings.audio_volume,
                            );

                            self.render_slider(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., start + lower_down * 1.5),
                                vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                                "Hit density",
                                TITLE_SIZE,
                                0.0..1.0,
                                self.default_settings.hit_density,
                                current_settings.hit_density,
                                &mut editing_settings.hit_density,
                            );

                            self.render_slider(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., start + lower_down * 2.7),
                                vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                                "Minimum hit speed",
                                TITLE_SIZE,
                                0.0..500.0,
                                self.default_settings.min_hit_speed,
                                current_settings.min_hit_speed,
                                &mut editing_settings.min_hit_speed,
                            );
                        }
                        _ => unreachable!(),
                    },
                    SettingsState::Visuals(page) => match *page {
                        0 => {
                            self.render_slider(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., start + lower_down * 0.),
                                vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                                "AO focus",
                                TITLE_SIZE,
                                0.0..5.0,
                                self.default_settings.ambient_occlusion_focus,
                                current_settings.ambient_occlusion_focus,
                                &mut editing_settings.ambient_occlusion_focus,
                            );

                            self.render_slider(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., start + lower_down * 1.),
                                vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                                "AO strength",
                                TITLE_SIZE,
                                0.0..5.0,
                                self.default_settings.ambient_occlusion_strength,
                                current_settings.ambient_occlusion_strength,
                                &mut editing_settings.ambient_occlusion_strength,
                            );

                            self.render_slider(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., start + lower_down * 2.),
                                vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                                "Specular focus",
                                TITLE_SIZE,
                                0.0..100.0,
                                self.default_settings.specular_focus,
                                current_settings.specular_focus,
                                &mut editing_settings.specular_focus,
                            );

                            self.render_slider(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., start + lower_down * 3.),
                                vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                                "Specular strength",
                                TITLE_SIZE,
                                0.0..10.0,
                                self.default_settings.specular_strength,
                                current_settings.specular_strength,
                                &mut editing_settings.specular_strength,
                            );
                        }
                        1 => {
                            self.render_slider(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., start + lower_down * 0.),
                                vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                                "Ambient light",
                                TITLE_SIZE,
                                0.0..1.0,
                                self.default_settings.ambient_light,
                                current_settings.ambient_light,
                                &mut editing_settings.ambient_light,
                            );

                            self.render_slider(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., start + lower_down * 1.),
                                vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                                "Shadow size",
                                TITLE_SIZE,
                                0.0..10.0,
                                self.default_settings.shadow_size,
                                current_settings.shadow_size,
                                &mut editing_settings.shadow_size,
                            );

                            self.render_slider(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., start + lower_down * 2.),
                                vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                                "Shadow dist strength",
                                TITLE_SIZE - 2,
                                0.0..10.0,
                                self.default_settings.shadow_distance_strength,
                                current_settings.shadow_distance_strength,
                                &mut editing_settings.shadow_distance_strength,
                            );

                            self.render_slider(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., start + lower_down * 3.),
                                vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                                "Shadow strength",
                                TITLE_SIZE,
                                0.0..10.0,
                                self.default_settings.shadow_strength,
                                current_settings.shadow_strength,
                                &mut editing_settings.shadow_strength,
                            );
                        }
                        _ => unreachable!(),
                    },
                    SettingsState::Box(page) => match *page {
                        0 => {
                            self.render_slider(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., start + lower_down * 0.1),
                                vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                                "Box weight",
                                TITLE_SIZE,
                                0.0..1.0,
                                self.default_settings.box_weight,
                                current_settings.box_weight,
                                &mut editing_settings.box_weight,
                            );

                            if self.render_button(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., 0. + lower_down * -0.3),
                                BUTTON_SIZE * vec2(1.2, 0.9),
                                &format!(
                                    "Hide weight: {}",
                                    if editing_settings.hide_smoothing {
                                        "On"
                                    } else {
                                        "Off"
                                    }
                                ),
                                get_changed_color(
                                    editing_settings.hide_smoothing
                                        != current_settings.hide_smoothing,
                                ),
                                20,
                            ) {
                                editing_settings.hide_smoothing = !editing_settings.hide_smoothing;
                            }

                            if self.render_button(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., 0. + lower_down * 1.0),
                                BUTTON_SIZE * vec2(1.1, 0.9),
                                &format!(
                                    "Quick turn: {}",
                                    if editing_settings.quick_turn {
                                        "On"
                                    } else {
                                        "Off"
                                    }
                                ),
                                get_changed_color(
                                    editing_settings.quick_turn != current_settings.quick_turn,
                                ),
                                20,
                            ) {
                                editing_settings.quick_turn = !editing_settings.quick_turn;
                            }
                        }
                        1 => {
                            self.render_slider_uint(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., start + lower_down * 0.),
                                vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                                "Box width",
                                TITLE_SIZE,
                                300..1000,
                                self.default_settings.box_width,
                                current_settings.box_width,
                                &mut editing_settings.box_width,
                            );

                            self.render_slider_uint(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., start + lower_down * 1.),
                                vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                                "Box height",
                                TITLE_SIZE,
                                400..1000,
                                self.default_settings.box_height,
                                current_settings.box_height,
                                &mut editing_settings.box_height,
                            );

                            self.render_slider_uint(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., start + lower_down * 2.),
                                vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                                "Box thickness",
                                TITLE_SIZE,
                                0..100,
                                self.default_settings.box_thickness,
                                current_settings.box_thickness,
                                &mut editing_settings.box_thickness,
                            );

                            self.render_slider_uint(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., start + lower_down * 3.),
                                vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                                "Box depth",
                                TITLE_SIZE,
                                1..100,
                                self.default_settings.box_depth,
                                current_settings.box_depth,
                                &mut editing_settings.box_depth,
                            );
                        }
                        _ => unreachable!(),
                    },
                    SettingsState::Physics(page) => match *page {
                        0 => {
                            self.render_slider(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., start + lower_down * 0.3),
                                vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                                "Gravity strength",
                                TITLE_SIZE,
                                -30.0..30.0,
                                self.default_settings.gravity_strength,
                                current_settings.gravity_strength,
                                &mut editing_settings.gravity_strength,
                            );

                            self.render_slider(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., start + lower_down * 1.5),
                                vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                                "Air friction",
                                TITLE_SIZE,
                                0.0..1.0,
                                self.default_settings.air_friction,
                                current_settings.air_friction,
                                &mut editing_settings.air_friction,
                            );

                            self.render_slider(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., start + lower_down * 2.7),
                                vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                                "Max velocity",
                                TITLE_SIZE,
                                0.0..500.0,
                                self.default_settings.max_velocity,
                                current_settings.max_velocity,
                                &mut editing_settings.max_velocity,
                            );
                        }
                        1 => {
                            self.render_slider(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., start + lower_down * 0.3),
                                vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                                "Ball bounciness",
                                TITLE_SIZE,
                                0.0..1.0,
                                self.default_settings.ball_bounciness,
                                current_settings.ball_bounciness,
                                &mut editing_settings.ball_bounciness,
                            );

                            self.render_slider(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., start + lower_down * 1.5),
                                vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                                "Ball weight",
                                TITLE_SIZE,
                                0.0..1.0,
                                self.default_settings.ball_weight,
                                current_settings.ball_weight,
                                &mut editing_settings.ball_weight,
                            );

                            self.render_slider(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., start + lower_down * 2.7),
                                vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                                "Ball friction",
                                TITLE_SIZE,
                                0.0..1.0,
                                self.default_settings.ball_friction,
                                current_settings.ball_friction,
                                &mut editing_settings.ball_friction,
                            );
                        }
                        _ => unreachable!(),
                    },
                    SettingsState::FpsDelay(page) => match *page {
                        0 => {
                            self.render_slider_uint(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., start + lower_down * 0.1),
                                vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                                "Delay frames",
                                TITLE_SIZE,
                                0..10,
                                self.default_settings.delay_frames,
                                current_settings.delay_frames,
                                &mut editing_settings.delay_frames,
                            );

                            self.render_maxed_slider_uint(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., start + lower_down * 1.2),
                                vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                                "Max FPS",
                                "None",
                                TITLE_SIZE,
                                0..FPS_LIMIT,
                                self.default_settings.max_fps,
                                current_settings.max_fps,
                                &mut editing_settings.max_fps,
                            );

                            if self.render_button(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., 0. + lower_down * 0.8),
                                BUTTON_SIZE * vec2(0.8, 0.75),
                                &format!(
                                    "VSync: {}",
                                    if editing_settings.vsync { "On" } else { "Off" }
                                ),
                                get_changed_color(editing_settings.vsync != current_settings.vsync),
                                21,
                            ) {
                                editing_settings.vsync = !editing_settings.vsync;
                            }
                        }
                        _ => unreachable!(),
                    },
                    SettingsState::Misc(page) => match *page {
                        0 => {
                            self.render_slider_uint(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., start + lower_down * 0.55),
                                vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                                "Ball radius",
                                TITLE_SIZE,
                                1..400,
                                self.default_settings.ball_radius,
                                current_settings.ball_radius,
                                &mut editing_settings.ball_radius,
                            );

                            self.render_slider(
                                game_assets,
                                hash!(),
                                mouse_pos,
                                vec2(0., start + lower_down * 1.75),
                                vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                                "Game speed",
                                TITLE_SIZE,
                                0.1..3.0,
                                self.default_settings.speed_mul,
                                current_settings.speed_mul,
                                &mut editing_settings.speed_mul,
                            );
                        }
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                }
            } else {
                let section_button_size = BUTTON_SIZE * vec2(0.725, 0.8);
                let seperate = section_button_size.x / 1.95;
                if self.render_button(
                    game_assets,
                    hash!(),
                    mouse_pos,
                    vec2(-seperate, lower_down * -2.),
                    section_button_size,
                    "Audio",
                    get_changed_default_color(editing_settings.audio_changed(current_settings)),
                    22,
                ) {
                    *settings_state = SettingsState::Audio(0);
                }

                if self.render_button(
                    game_assets,
                    hash!(),
                    mouse_pos,
                    vec2(seperate, lower_down * -2.),
                    section_button_size,
                    "Visuals",
                    get_changed_default_color(editing_settings.visual_changed(current_settings)),
                    22,
                ) {
                    *settings_state = SettingsState::Visuals(0);
                }

                if self.render_button(
                    game_assets,
                    hash!(),
                    mouse_pos,
                    vec2(-seperate, lower_down * -0.9),
                    section_button_size,
                    "Box",
                    get_changed_default_color(editing_settings.box_changed(current_settings)),
                    22,
                ) {
                    *settings_state = SettingsState::Box(0);
                }

                if self.render_button(
                    game_assets,
                    hash!(),
                    mouse_pos,
                    vec2(seperate, lower_down * -0.9),
                    section_button_size,
                    "Physics",
                    get_changed_default_color(editing_settings.physics_changed(current_settings)),
                    22,
                ) {
                    *settings_state = SettingsState::Physics(0);
                }

                if self.render_button(
                    game_assets,
                    hash!(),
                    mouse_pos,
                    vec2(-seperate, lower_down * 0.2),
                    section_button_size,
                    "FPS/delay",
                    get_changed_default_color(editing_settings.fps_delay_changed(current_settings)),
                    20,
                ) {
                    *settings_state = SettingsState::FpsDelay(0);
                }

                if self.render_button(
                    game_assets,
                    hash!(),
                    mouse_pos,
                    vec2(seperate, lower_down * 0.2),
                    section_button_size,
                    "Misc",
                    get_changed_default_color(editing_settings.misc_changed(current_settings)),
                    22,
                ) {
                    *settings_state = SettingsState::Misc(0);
                }

                if self.render_button(
                    game_assets,
                    hash!(),
                    mouse_pos,
                    vec2(0., 0. + lower_down * 1.35),
                    BUTTON_SIZE * vec2(1.05, 0.8),
                    "Reset settings",
                    DARKRED_TEXT_COLOR,
                    20,
                ) {
                    *editing_settings = self.default_settings.clone();
                }
            }

            let center_offset_x = -MENU_SIZE.x / 2. + BUTTON_SIZE.x / 2. + BUTTONS_MARGIN / 2.;

            let y_offset = -MENU_SIZE.y / 2.
                + MENU_PADDING
                + BUTTONS_MARGIN
                + BUTTON_SIZE.y / SMALL_BUTTON_DIV / 2.;

            if self.render_button(
                game_assets,
                hash!(),
                mouse_pos,
                vec2(center_offset_x, -y_offset),
                BUTTON_SIZE / SMALL_BUTTON_DIV,
                "Back",
                DEFAULT_TEXT_COLOR,
                28,
            ) {
                settings_state.back();
            }

            if self.render_button(
                game_assets,
                hash!(),
                mouse_pos,
                vec2(-center_offset_x, -y_offset),
                BUTTON_SIZE / SMALL_BUTTON_DIV,
                "Apply",
                get_changed_default_color(current_settings != editing_settings),
                28,
            ) {
                save = true;
            }
        } else {
            let button_y_offsets = BUTTONS_MARGIN + BUTTON_SIZE.y;

            if self.render_button(
                game_assets,
                hash!(),
                mouse_pos,
                vec2(0., -button_y_offsets),
                BUTTON_SIZE,
                "Continue",
                DEFAULT_TEXT_COLOR,
                28,
            ) {
                *settings_state = SettingsState::Closed;
            }

            if self.render_button(
                game_assets,
                hash!(),
                mouse_pos,
                vec2(0., 0.),
                BUTTON_SIZE,
                "Settings",
                DEFAULT_TEXT_COLOR,
                28,
            ) {
                *settings_state = SettingsState::Settings;
            }

            if self.render_button(
                game_assets,
                hash!(),
                mouse_pos,
                vec2(0., button_y_offsets),
                BUTTON_SIZE,
                "Quit",
                DEFAULT_TEXT_COLOR,
                28,
            ) {
                order_quit();
            }
        }

        self.reset_field = false;

        return save;
    }

    pub fn render_text(
        &mut self,
        game_assets: &GameAssets,
        center_pos: Vec2,
        size: Vec2,
        text: &str,
        font_size: u16,
    ) {
        let rect = Rect::new(
            (center_pos.x * 2. - size.x) * self.mult,
            (center_pos.y * 2. - size.y) * self.mult,
            size.x * 2. * self.mult,
            size.y * 2. * self.mult,
        );

        let size = measure_text(text, game_assets.font.as_ref(), font_size, 2.0 * self.mult);

        draw_text_ex(
            text,
            rect.x + rect.w / 2. - size.width / 2.,
            rect.y + rect.h / 2. + font_size as f32 / 2. * self.mult,
            TextParams {
                color: DEFAULT_TEXT_COLOR,
                font: game_assets.font.as_ref(),
                font_size,
                font_scale: 2.0 * self.mult,
                ..Default::default()
            },
        );
    }

    pub fn render_button(
        &mut self,
        game_assets: &GameAssets,
        id: u64,
        mouse_pos: Vec2,
        center_pos: Vec2,
        size: Vec2,
        text: &str,
        text_color: Color,
        font_size: u16,
    ) -> bool {
        let rect = Rect::new(
            (center_pos.x * 2. - size.x) * self.mult,
            (center_pos.y * 2. - size.y) * self.mult,
            size.x * 2. * self.mult,
            size.y * 2. * self.mult,
        );

        let contains_mouse = rect.contains(mouse_pos);
        let mouse_is_released = is_mouse_button_released(MouseButton::Left);
        let mouse_is_pressed = is_mouse_button_pressed(MouseButton::Left);
        let mouse_is_down = is_mouse_button_down(MouseButton::Left) || mouse_is_released;

        if contains_mouse {
            set_mouse_cursor(CursorIcon::Pointer);
            if mouse_is_pressed {
                self.active_id = id;
            }
        } else if self.active_id == id {
            self.active_id = 0;
        }

        let button_is_active = self.active_id == id;

        let color = if button_is_active && mouse_is_down {
            Color::new(0.80, 0.80, 0.80, 1.0)
        } else if contains_mouse {
            Color::new(0.90, 0.90, 0.90, 1.0)
        } else {
            WHITE
        };

        draw_texture_ex(
            &game_assets.menu_button,
            rect.x,
            rect.y,
            color,
            DrawTextureParams {
                dest_size: Some(vec2(rect.w, rect.h)),
                ..Default::default()
            },
        );

        let size = measure_text(text, game_assets.font.as_ref(), font_size, 2.0 * self.mult);

        draw_text_ex(
            text,
            rect.x + rect.w / 2. - size.width / 2.,
            rect.y + rect.h / 2. + font_size as f32 / 2. * self.mult,
            TextParams {
                color: text_color,
                font: game_assets.font.as_ref(),
                font_size,
                font_scale: 2.0 * self.mult,
                ..Default::default()
            },
        );

        return button_is_active && mouse_is_released;
    }

    pub fn render_slider(
        &mut self,
        game_assets: &GameAssets,
        id: u64,
        mouse_pos: Vec2,
        center_pos: Vec2,
        size: Vec2,
        title: &str,
        font_size: u16,
        range: Range<f32>,
        default_value: f32,
        prev_value: f32,
        value: &mut f32,
    ) -> bool {
        let slider_size = 0.85;

        let full_rect = Rect::new(
            (center_pos.x * 2. - size.x) * self.mult,
            (center_pos.y * 2. - size.y) * self.mult,
            size.x * 2. * self.mult,
            size.y * 2. * self.mult,
        );

        let slider_rect = Rect::new(
            full_rect.x + full_rect.w * (1. - slider_size),
            full_rect.y,
            full_rect.w * slider_size,
            full_rect.h,
        );
        let number_rect = Rect::new(
            full_rect.x,
            full_rect.y,
            full_rect.w * (1. - slider_size),
            full_rect.h,
        );

        let contains_mouse = full_rect.contains(mouse_pos);
        let slider_contains_mouse = slider_rect.contains(mouse_pos);
        let mouse_is_pressed = is_mouse_button_pressed(MouseButton::Left);
        let mouse_is_down = is_mouse_button_down(MouseButton::Left);

        if contains_mouse {
            set_mouse_cursor(CursorIcon::Pointer);
        }

        if !contains_mouse && mouse_is_pressed && self.active_id == id {
            self.active_id = 0;
            self.user_input = String::new()
        } else if contains_mouse && mouse_is_pressed {
            self.active_id = id;
            self.slider_follow = slider_contains_mouse;
            self.user_input = String::new()
        } else if contains_mouse && mouse_is_down && self.active_id == id {
            self.slider_follow = self.slider_follow || slider_contains_mouse;
        } else if is_key_pressed(KeyCode::Enter) && self.active_id == id {
            self.active_id = 0;
            self.user_input = String::new()
        }

        let is_active = self.active_id == id;
        let will_follow = is_active && mouse_is_down && self.slider_follow;

        let bar_width_pct = 0.1;
        let bar_height_pct = 1.25;
        let bar_width = slider_rect.w * bar_width_pct;
        let bar_height = slider_rect.h * bar_height_pct;

        let value_string = if will_follow {
            let amount = ((mouse_pos.x - slider_rect.x - bar_width / 2.)
                / (slider_rect.w - bar_width))
                .clamp(0., 1.);
            let ranged_amount = range.start + amount * (range.end - range.start);
            *value = ranged_amount;
            self.user_input = String::new();
            &format!("{:.2}", *value)
        } else if is_active && !self.user_input.is_empty() {
            if let Ok(parsed_value) = self.user_input.parse::<f32>() {
                *value = parsed_value.clamp(range.start, range.end)
            }
            &self.user_input
        } else if is_active && self.reset_field {
            *value = default_value;
            &format!("{:.2}", *value)
        } else {
            &format!("{:.2}", *value)
        };

        let zero_to_one = (*value - range.start) / (range.end - range.start);
        let zero_to_width = zero_to_one * slider_rect.w * (1. - bar_width_pct);

        let bar_rect = Rect::new(
            slider_rect.x + zero_to_width,
            slider_rect.y - bar_height / 2. + slider_rect.h / 2.,
            bar_width,
            bar_height,
        );

        draw_texture_ex(
            &game_assets.slider_background,
            slider_rect.x,
            slider_rect.y,
            Color::from_hex(0xCCCCCC),
            DrawTextureParams {
                dest_size: Some(vec2(slider_rect.w, slider_rect.h)),
                ..Default::default()
            },
        );

        draw_texture_ex(
            &game_assets.slider_bar,
            bar_rect.x,
            bar_rect.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(bar_rect.w, bar_rect.h)),
                ..Default::default()
            },
        );

        let font_size_mult = 0.4;

        let centered_y_offset =
            number_rect.y + number_rect.h * ((1. - (0.65 - font_size_mult) / 0.65) / 2. + 0.5);

        let value_font_size_f = number_rect.h * font_size_mult;

        let value_font_size = (value_font_size_f / self.mult) as u16;

        let size = measure_text(
            &value_string,
            game_assets.font.as_ref(),
            value_font_size,
            2.0 * self.mult,
        );

        draw_text_ex(
            &value_string,
            number_rect.x + number_rect.w - size.width - value_font_size_f * 0.5,
            centered_y_offset,
            TextParams {
                color: if is_active {
                    ACTIVE_TEXT_COLOR
                } else if prev_value != *value {
                    CHANGED_TEXT_COLOR
                } else {
                    BLACK
                },
                font: game_assets.font.as_ref(),
                font_size: value_font_size,
                font_scale: 2.0 * self.mult,
                ..Default::default()
            },
        );

        draw_text_ex(
            title,
            full_rect.x,
            full_rect.y - font_size as f32 * 0.65 * self.mult,
            TextParams {
                color: DEFAULT_TEXT_COLOR,
                font: game_assets.font.as_ref(),
                font_size,
                font_scale: 2.0 * self.mult,
                ..Default::default()
            },
        );

        return false;
    }

    pub fn render_slider_uint(
        &mut self,
        game_assets: &GameAssets,
        id: u64,
        mouse_pos: Vec2,
        center_pos: Vec2,
        size: Vec2,
        title: &str,
        font_size: u16,
        range: Range<u32>,
        default_value: u32,
        prev_value: u32,
        value: &mut u32,
    ) -> bool {
        let slider_size = 0.85;

        let full_rect = Rect::new(
            (center_pos.x * 2. - size.x) * self.mult,
            (center_pos.y * 2. - size.y) * self.mult,
            size.x * 2. * self.mult,
            size.y * 2. * self.mult,
        );

        let slider_rect = Rect::new(
            full_rect.x + full_rect.w * (1. - slider_size),
            full_rect.y,
            full_rect.w * slider_size,
            full_rect.h,
        );
        let number_rect = Rect::new(
            full_rect.x,
            full_rect.y,
            full_rect.w * (1. - slider_size),
            full_rect.h,
        );

        let contains_mouse = full_rect.contains(mouse_pos);
        let slider_contains_mouse = slider_rect.contains(mouse_pos);
        let mouse_is_pressed = is_mouse_button_pressed(MouseButton::Left);
        let mouse_is_down = is_mouse_button_down(MouseButton::Left);

        if contains_mouse {
            set_mouse_cursor(CursorIcon::Pointer);
        }

        if !contains_mouse && mouse_is_pressed && self.active_id == id {
            self.active_id = 0;
            self.user_input = String::new()
        } else if contains_mouse && mouse_is_pressed {
            self.active_id = id;
            self.slider_follow = slider_contains_mouse;
            self.user_input = String::new()
        } else if contains_mouse && mouse_is_down && self.active_id == id {
            self.slider_follow = self.slider_follow || slider_contains_mouse;
        } else if is_key_pressed(KeyCode::Enter) && self.active_id == id {
            self.active_id = 0;
            self.user_input = String::new()
        }

        let is_active = self.active_id == id;
        let will_follow = is_active && mouse_is_down && self.slider_follow;

        let bar_width_pct = 0.1;
        let bar_height_pct = 1.25;
        let bar_width = slider_rect.w * bar_width_pct;
        let bar_height = slider_rect.h * bar_height_pct;

        let value_string = if will_follow {
            let amount = ((mouse_pos.x - slider_rect.x - bar_width / 2.)
                / (slider_rect.w - bar_width))
                .clamp(0., 1.);
            let ranged_amount =
                range.start as f32 + amount * (range.end as f32 - range.start as f32);
            *value = ranged_amount.round() as u32;
            &format!("{}", *value)
        } else if is_active && !self.user_input.is_empty() {
            if let Ok(parsed_value) = self.user_input.parse::<u32>() {
                *value = parsed_value.clamp(range.start, range.end)
            }
            &self.user_input
        } else if is_active && self.reset_field {
            *value = default_value;
            &format!("{}", *value)
        } else {
            &format!("{}", *value)
        };

        let zero_to_one =
            (*value as f32 - range.start as f32) / (range.end as f32 - range.start as f32);
        let zero_to_width = zero_to_one * slider_rect.w * (1. - bar_width_pct);

        let bar_rect = Rect::new(
            slider_rect.x + zero_to_width,
            slider_rect.y - bar_height / 2. + slider_rect.h / 2.,
            bar_width,
            bar_height,
        );

        draw_texture_ex(
            &game_assets.slider_background,
            slider_rect.x,
            slider_rect.y,
            Color::from_hex(0xCCCCCC),
            DrawTextureParams {
                dest_size: Some(vec2(slider_rect.w, slider_rect.h)),
                ..Default::default()
            },
        );

        draw_texture_ex(
            &game_assets.slider_bar,
            bar_rect.x,
            bar_rect.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(bar_rect.w, bar_rect.h)),
                ..Default::default()
            },
        );

        let font_size_mult = 0.4;

        let centered_y_offset =
            number_rect.y + number_rect.h * ((1. - (0.65 - font_size_mult) / 0.65) / 2. + 0.5);

        let value_font_size_f = number_rect.h * font_size_mult;

        let value_font_size = (value_font_size_f / self.mult) as u16;

        let size = measure_text(
            &value_string,
            game_assets.font.as_ref(),
            value_font_size,
            2.0 * self.mult,
        );

        draw_text_ex(
            &value_string,
            number_rect.x + number_rect.w - size.width - value_font_size_f * 0.5,
            centered_y_offset,
            TextParams {
                color: if is_active {
                    ACTIVE_TEXT_COLOR
                } else if prev_value != *value {
                    CHANGED_TEXT_COLOR
                } else {
                    BLACK
                },
                font: game_assets.font.as_ref(),
                font_size: value_font_size,
                font_scale: 2.0 * self.mult,
                ..Default::default()
            },
        );

        draw_text_ex(
            title,
            full_rect.x,
            full_rect.y - font_size as f32 * 0.65 * self.mult,
            TextParams {
                color: DEFAULT_TEXT_COLOR,
                font: game_assets.font.as_ref(),
                font_size,
                font_scale: 2.0 * self.mult,
                ..Default::default()
            },
        );

        return false;
    }

    pub fn render_maxed_slider_uint(
        &mut self,
        game_assets: &GameAssets,
        id: u64,
        mouse_pos: Vec2,
        center_pos: Vec2,
        size: Vec2,
        title: &str,
        maxed_text: &str,
        font_size: u16,
        range: Range<u32>,
        default_value: u32,
        prev_value: u32,
        value: &mut u32,
    ) -> bool {
        let slider_size = 0.85;

        let full_rect = Rect::new(
            (center_pos.x * 2. - size.x) * self.mult,
            (center_pos.y * 2. - size.y) * self.mult,
            size.x * 2. * self.mult,
            size.y * 2. * self.mult,
        );

        let slider_rect = Rect::new(
            full_rect.x + full_rect.w * (1. - slider_size),
            full_rect.y,
            full_rect.w * slider_size,
            full_rect.h,
        );
        let number_rect = Rect::new(
            full_rect.x,
            full_rect.y,
            full_rect.w * (1. - slider_size),
            full_rect.h,
        );

        let contains_mouse = full_rect.contains(mouse_pos);
        let slider_contains_mouse = slider_rect.contains(mouse_pos);
        let mouse_is_pressed = is_mouse_button_pressed(MouseButton::Left);
        let mouse_is_down = is_mouse_button_down(MouseButton::Left);

        if contains_mouse {
            set_mouse_cursor(CursorIcon::Pointer);
        }

        if !contains_mouse && mouse_is_pressed && self.active_id == id {
            self.active_id = 0;
            self.user_input = String::new()
        } else if contains_mouse && mouse_is_pressed {
            self.active_id = id;
            self.slider_follow = slider_contains_mouse;
            self.user_input = String::new()
        } else if contains_mouse && mouse_is_down && self.active_id == id {
            self.slider_follow = self.slider_follow || slider_contains_mouse;
        } else if is_key_pressed(KeyCode::Enter) && self.active_id == id {
            self.active_id = 0;
            self.user_input = String::new()
        }

        let is_active = self.active_id == id;
        let will_follow = is_active && mouse_is_down && self.slider_follow;

        let bar_width_pct = 0.1;
        let bar_height_pct = 1.25;
        let bar_width = slider_rect.w * bar_width_pct;
        let bar_height = slider_rect.h * bar_height_pct;

        let value_string = if will_follow {
            let amount = ((mouse_pos.x - slider_rect.x - bar_width / 2.)
                / (slider_rect.w - bar_width))
                .clamp(0., 1.);
            let ranged_amount =
                range.start as f32 + amount * (range.end as f32 - range.start as f32);
            *value = ranged_amount.round() as u32;

            if *value >= range.end {
                maxed_text
            } else {
                &format!("{}", *value)
            }
        } else if is_active && !self.user_input.is_empty() {
            if let Ok(parsed_value) = self.user_input.parse::<u32>() {
                *value = parsed_value.clamp(range.start, range.end)
            }
            &self.user_input
        } else if is_active && self.reset_field {
            *value = default_value;
            if *value >= range.end {
                maxed_text
            } else {
                &format!("{}", *value)
            }
        } else {
            if *value >= range.end {
                maxed_text
            } else {
                &format!("{}", *value)
            }
        };

        let zero_to_one = (*value - range.start) as f32 / (range.end - range.start) as f32;
        let zero_to_width = zero_to_one * slider_rect.w * (1. - bar_width_pct);

        let bar_rect = Rect::new(
            slider_rect.x + zero_to_width,
            slider_rect.y - bar_height / 2. + slider_rect.h / 2.,
            bar_width,
            bar_height,
        );

        draw_texture_ex(
            &game_assets.slider_background,
            slider_rect.x,
            slider_rect.y,
            Color::from_hex(0xCCCCCC),
            DrawTextureParams {
                dest_size: Some(vec2(slider_rect.w, slider_rect.h)),
                ..Default::default()
            },
        );

        draw_texture_ex(
            &game_assets.slider_bar,
            bar_rect.x,
            bar_rect.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(bar_rect.w, bar_rect.h)),
                ..Default::default()
            },
        );

        let font_size_mult = 0.4;

        let centered_y_offset =
            number_rect.y + number_rect.h * ((1. - (0.65 - font_size_mult) / 0.65) / 2. + 0.5);

        let value_font_size_f = number_rect.h * font_size_mult;

        let value_font_size = (value_font_size_f / self.mult) as u16;

        let size = measure_text(
            &value_string,
            game_assets.font.as_ref(),
            value_font_size,
            2.0 * self.mult,
        );

        draw_text_ex(
            &value_string,
            number_rect.x + number_rect.w - size.width - value_font_size_f * 0.5,
            centered_y_offset,
            TextParams {
                color: if is_active {
                    ACTIVE_TEXT_COLOR
                } else if prev_value != *value {
                    CHANGED_TEXT_COLOR
                } else {
                    BLACK
                },
                font: game_assets.font.as_ref(),
                font_size: value_font_size,
                font_scale: 2.0 * self.mult,
                ..Default::default()
            },
        );

        draw_text_ex(
            title,
            full_rect.x,
            full_rect.y - font_size as f32 * 0.65 * self.mult,
            TextParams {
                color: DEFAULT_TEXT_COLOR,
                font: game_assets.font.as_ref(),
                font_size,
                font_scale: 2.0 * self.mult,
                ..Default::default()
            },
        );

        return false;
    }
}
