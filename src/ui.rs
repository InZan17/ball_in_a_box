use std::ops::Range;

use macroquad::{prelude::*, ui::hash};
use miniquad::*;
use window::order_quit;

use crate::Settings;

pub const MENU_SIZE: Vec2 = vec2(310., 400.);
const BUTTON_SIZE: Vec2 = vec2(160., 75.);
const BUTTONS_MARGIN: f32 = 20.;

const MENU_PADDING: f32 = 10.;
const SMALL_BUTTON_DIV: f32 = 1.5;
const SMALLER_BUTTON_DIV: f32 = 1.75;

const LAST_PAGE_INDEX: u8 = 1;

#[derive(Debug, PartialEq, Clone)]
pub enum SettingsState {
    Closed,
    Open,
    Settings(u8),
}

impl SettingsState {
    pub fn is_open(&self) -> bool {
        match self {
            SettingsState::Closed => false,
            _ => true,
        }
    }
}

pub struct UiRenderer {
    menu_background: Texture2D,
    button: Texture2D,
    slider_background: Texture2D,
    slider_bar: Texture2D,
    font: Font,
    active_id: u64,
}

impl UiRenderer {
    pub async fn new() -> Self {
        Self {
            menu_background: load_texture("./assets/main_background.png")
                .await
                .expect("Failed to load assets/main_background.png file"),

            button: load_texture("./assets/cardboard_button.png")
                .await
                .expect("Failed to load assets/cardboard_button.png file"),
            slider_background: load_texture("./assets/slider_background.png")
                .await
                .expect("Failed to load assets/slider_background.png file"),
            slider_bar: load_texture("./assets/slider_bar.png")
                .await
                .expect("Failed to load assets/slider_bar.png file"),
            font: load_ttf_font("./assets/font.ttf")
                .await
                .expect("Failed to load assets/font.ttf file"),
            active_id: 0,
        }
    }

    pub fn render_ui(
        &mut self,
        editing_settings: &mut Settings,
        settings_state: &mut SettingsState,
        mouse_pos: Vec2,
        box_size: (f32, f32),
    ) -> bool {
        if *settings_state == SettingsState::Closed {
            return false;
        }

        let mouse_pos = mouse_pos * 2. - vec2(box_size.0, box_size.1);

        draw_rectangle(
            -box_size.0,
            -box_size.1,
            box_size.0 * 2.,
            box_size.1 * 2.,
            Color::from_rgba(0, 0, 0, 100),
        );

        let mut save = false;

        let menu_position = -MENU_SIZE;

        let menu_rect = Rect::new(
            menu_position.x,
            menu_position.y,
            MENU_SIZE.x * 2.,
            MENU_SIZE.y * 2.,
        );

        draw_texture_ex(
            &self.menu_background,
            menu_rect.x,
            menu_rect.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(menu_rect.w, menu_rect.h)),
                ..Default::default()
            },
        );

        if let SettingsState::Settings(settings_page) = settings_state.clone() {
            let center_offset_x =
                -MENU_SIZE.x / 2. + BUTTON_SIZE.x / SMALLER_BUTTON_DIV + BUTTONS_MARGIN / 2.;

            let y_offset = -MENU_SIZE.y / 2.
                + MENU_PADDING
                + BUTTONS_MARGIN
                + BUTTON_SIZE.y / SMALLER_BUTTON_DIV / 2.;

            if settings_page > 0 {
                if self.render_button(
                    hash!(),
                    mouse_pos,
                    vec2(center_offset_x, y_offset),
                    BUTTON_SIZE / SMALLER_BUTTON_DIV,
                    "Prev",
                    28,
                ) {
                    *settings_state = SettingsState::Settings(settings_page - 1);
                }
            }

            if settings_page < LAST_PAGE_INDEX {
                if self.render_button(
                    hash!(),
                    mouse_pos,
                    vec2(-center_offset_x, y_offset),
                    BUTTON_SIZE / SMALLER_BUTTON_DIV,
                    "Next",
                    28,
                ) {
                    *settings_state = SettingsState::Settings(settings_page + 1);
                }
            }

            const SLIDER_HEIGHT: f32 = 24.;
            const SLIDER_WIDTH: f32 = MENU_SIZE.x * 0.65;
            const TITLE_SIZE: u16 = 24;
            const OPTIONS_SPACING: f32 = 13.;

            let start =
                -MENU_SIZE.y / 2. + 5. + BUTTON_SIZE.y / SMALLER_BUTTON_DIV * 2. + SLIDER_HEIGHT;

            let lower_down = SLIDER_HEIGHT + TITLE_SIZE as f32 + OPTIONS_SPACING;

            match settings_page {
                0 => {
                    self.render_slider(
                        hash!(),
                        mouse_pos,
                        vec2(0., start + lower_down * 0.),
                        vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                        "Audio volume",
                        TITLE_SIZE,
                        0.0..1.0,
                        &mut editing_settings.audio_volume,
                    );

                    self.render_slider(
                        hash!(),
                        mouse_pos,
                        vec2(0., start + lower_down * 1.),
                        vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                        "Gravity strength",
                        TITLE_SIZE,
                        -30.0..30.0,
                        &mut editing_settings.gravity_strength,
                    );

                    self.render_slider(
                        hash!(),
                        mouse_pos,
                        vec2(0., start + lower_down * 2.),
                        vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                        "Air friction",
                        TITLE_SIZE,
                        0.0..1.0,
                        &mut editing_settings.air_friction,
                    );

                    self.render_slider(
                        hash!(),
                        mouse_pos,
                        vec2(0., start + lower_down * 3.),
                        vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                        "Max velocity",
                        TITLE_SIZE,
                        0.0..500.0,
                        &mut editing_settings.max_velocity,
                    );
                }
                1 => {
                    self.render_slider(
                        hash!(),
                        mouse_pos,
                        vec2(0., start + lower_down * 0.),
                        vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                        "Ball bounciness",
                        TITLE_SIZE,
                        0.0..1.0,
                        &mut editing_settings.ball_bounciness,
                    );

                    self.render_slider(
                        hash!(),
                        mouse_pos,
                        vec2(0., start + lower_down * 1.),
                        vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                        "Ball radius",
                        TITLE_SIZE,
                        1.0..400.0,
                        &mut editing_settings.ball_radius,
                    );

                    self.render_slider(
                        hash!(),
                        mouse_pos,
                        vec2(0., start + lower_down * 2.),
                        vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                        "Ball weight",
                        TITLE_SIZE,
                        0.0..1.0,
                        &mut editing_settings.ball_weight,
                    );

                    self.render_slider(
                        hash!(),
                        mouse_pos,
                        vec2(0., start + lower_down * 3.),
                        vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                        "Ball friction",
                        TITLE_SIZE,
                        0.0..1.0,
                        &mut editing_settings.ball_friction,
                    );
                }
                _ => {
                    unimplemented!()
                }
            }

            let center_offset_x = -MENU_SIZE.x / 2. + BUTTON_SIZE.x / 2. + BUTTONS_MARGIN / 2.;

            let y_offset = -MENU_SIZE.y / 2.
                + MENU_PADDING
                + BUTTONS_MARGIN
                + BUTTON_SIZE.y / SMALL_BUTTON_DIV / 2.;

            if self.render_button(
                hash!(),
                mouse_pos,
                vec2(center_offset_x, -y_offset),
                BUTTON_SIZE / SMALL_BUTTON_DIV,
                "Back",
                28,
            ) {
                *settings_state = SettingsState::Open;
            }

            if self.render_button(
                hash!(),
                mouse_pos,
                vec2(-center_offset_x, -y_offset),
                BUTTON_SIZE / SMALL_BUTTON_DIV,
                "Apply",
                28,
            ) {
                save = true;
            }
        } else {
            let button_y_offsets = BUTTONS_MARGIN + BUTTON_SIZE.y;

            if self.render_button(
                hash!(),
                mouse_pos,
                vec2(0., -button_y_offsets),
                BUTTON_SIZE,
                "Continue",
                28,
            ) {
                *settings_state = SettingsState::Closed;
            }

            if self.render_button(
                hash!(),
                mouse_pos,
                vec2(0., 0.),
                BUTTON_SIZE,
                "Settings",
                28,
            ) {
                *settings_state = SettingsState::Settings(0);
            }

            if self.render_button(
                hash!(),
                mouse_pos,
                vec2(0., button_y_offsets),
                BUTTON_SIZE,
                "Quit",
                28,
            ) {
                order_quit();
            }
        }

        return save;
    }

    pub fn render_button(
        &mut self,
        id: u64,
        mouse_pos: Vec2,
        center_pos: Vec2,
        size: Vec2,
        text: &str,
        font_size: u16,
    ) -> bool {
        let rect = Rect::new(
            center_pos.x * 2. - size.x,
            center_pos.y * 2. - size.y,
            size.x * 2.,
            size.y * 2.,
        );

        let contains_mouse = rect.contains(mouse_pos);
        let mouse_is_released = is_mouse_button_released(MouseButton::Left);
        let mouse_is_pressed = is_mouse_button_pressed(MouseButton::Left);
        let mouse_is_down = is_mouse_button_down(MouseButton::Left) || mouse_is_released;

        if contains_mouse {
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
            &self.button,
            rect.x,
            rect.y,
            color,
            DrawTextureParams {
                dest_size: Some(vec2(rect.w, rect.h)),
                ..Default::default()
            },
        );

        let size = measure_text(text, Some(&self.font), font_size, 2.0);

        draw_text_ex(
            text,
            rect.x + rect.w / 2. - size.width / 2.,
            rect.y + rect.h / 2. + font_size as f32 / 2.,
            TextParams {
                color: Color::new(0.05, 0., 0.1, 1.),
                font: Some(&self.font),
                font_size,
                font_scale: 2.0,
                ..Default::default()
            },
        );

        return button_is_active && mouse_is_released;
    }

    pub fn render_slider(
        &mut self,
        id: u64,
        mouse_pos: Vec2,
        center_pos: Vec2,
        size: Vec2,
        title: &str,
        font_size: u16,
        range: Range<f32>,
        value: &mut f32,
    ) -> bool {
        let slider_size = 0.85;

        let full_rect = Rect::new(
            center_pos.x * 2. - size.x,
            center_pos.y * 2. - size.y,
            size.x * 2.,
            size.y * 2.,
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

        let contains_mouse = slider_rect.contains(mouse_pos);
        let mouse_is_pressed = is_mouse_button_pressed(MouseButton::Left);
        let mouse_is_down = is_mouse_button_down(MouseButton::Left);

        if !mouse_is_down && self.active_id == id {
            self.active_id = 0;
        } else if contains_mouse && mouse_is_pressed {
            self.active_id = id;
        }

        let is_active = self.active_id == id;

        let bar_width_pct = 0.1;
        let bar_height_pct = 1.25;
        let bar_width = slider_rect.w * bar_width_pct;
        let bar_height = slider_rect.h * bar_height_pct;

        if is_active {
            let amount = ((mouse_pos.x - slider_rect.x - bar_width / 2.)
                / (slider_rect.w - bar_width))
                .clamp(0., 1.);
            let ranged_amount = range.start + amount * (range.end - range.start);
            *value = ranged_amount;
        }

        let zero_to_one = (*value - range.start) / (range.end - range.start);
        let zero_to_width = zero_to_one * slider_rect.w * (1. - bar_width_pct);

        let bar_rect = Rect::new(
            slider_rect.x + zero_to_width,
            slider_rect.y - bar_height / 2. + slider_rect.h / 2.,
            bar_width,
            bar_height,
        );

        draw_texture_ex(
            &self.slider_background,
            slider_rect.x,
            slider_rect.y,
            Color::from_hex(0xCCCCCC),
            DrawTextureParams {
                dest_size: Some(vec2(slider_rect.w, slider_rect.h)),
                ..Default::default()
            },
        );

        draw_texture_ex(
            &self.slider_bar,
            bar_rect.x,
            bar_rect.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(bar_rect.w, bar_rect.h)),
                ..Default::default()
            },
        );

        let value_string = format!("{:.2}", *value);

        let font_size_mult = 0.4;

        let centered_y_offset =
            number_rect.y + number_rect.h * ((1. - (0.65 - font_size_mult) / 0.65) / 2. + 0.5);

        let value_font_size_f = number_rect.h * font_size_mult;

        let value_font_size = value_font_size_f as u16;

        let size = measure_text(&value_string, Some(&self.font), value_font_size, 2.0);

        draw_text_ex(
            &value_string,
            number_rect.x + number_rect.w - size.width - value_font_size_f * 0.5,
            centered_y_offset,
            TextParams {
                color: Color::new(0.05, 0., 0.1, 1.),
                font: Some(&self.font),
                font_size: value_font_size,
                font_scale: 2.0,
                ..Default::default()
            },
        );

        draw_text_ex(
            title,
            full_rect.x,
            full_rect.y - font_size as f32 * 0.65,
            TextParams {
                color: Color::new(0.05, 0., 0.1, 1.),
                font: Some(&self.font),
                font_size,
                font_scale: 2.0,
                ..Default::default()
            },
        );

        return false;
    }
}
