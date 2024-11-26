use std::ops::Range;

use macroquad::{
    prelude::*,
    ui::{hash, root_ui, widgets, Skin},
};
use miniquad::*;
use window::{dpi_scale, order_quit};

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

pub async fn create_skin() -> Skin {
    // FrederickatheGreat-Regular
    let font_bytes = load_file("./assets/font.ttf")
        .await
        .expect("Couldn't find assets/font.ttf file");

    let font = load_ttf_font_from_bytes(&font_bytes).expect("Couldn't load assets/font.ttf");

    drop(font_bytes);

    let background_bytes = load_file("./assets/main_background.png")
        .await
        .expect("Couldn't find assets/main_background.png file");

    let window_style = root_ui()
        .style_builder()
        .background(
            Image::from_file_with_format(&background_bytes, None)
                .expect("Couldn't load assets/main_background.png"),
        )
        .build();

    drop(background_bytes);

    let button_bytes = load_file("./assets/cardboard_button.png")
        .await
        .expect("Couldn't find assets/cardboard_button.png file");

    let button_style = root_ui()
        .style_builder()
        .background(
            Image::from_file_with_format(&button_bytes, None)
                .expect("Couldn't load assets/cardboard_button.png"),
        )
        .with_font(&font)
        .unwrap()
        .font_size((28.0 / dpi_scale()) as u16)
        .text_color(Color::new(0.05, 0., 0.1, 1.))
        .color_hovered(Color::new(0.90, 0.90, 0.90, 1.0))
        .build();

    drop(button_bytes);

    let label_style = root_ui()
        .style_builder()
        .with_font(&font)
        .unwrap()
        .font_size((24.0 / dpi_scale()) as u16)
        .text_color(Color::new(0.05, 0., 0.1, 1.))
        .margin(RectOffset::new(0., 0., 10., 0.))
        .build();

    let editbox_style = root_ui()
        .style_builder()
        .with_font(&font)
        .unwrap()
        // Don't divide by dpi_scale() because it doesn't make a difference in spacing.
        .font_size(16)
        .text_color(Color::new(0., 0., 0., 1.))
        .color(Color::new(0.0, 0.90, 0.90, 0.0))
        .color_selected(Color::new(0.0, 0.90, 0.90, 0.0))
        .color_clicked(Color::new(0.0, 0.90, 0.90, 0.0))
        .build();

    let checkbox_style = root_ui()
        .style_builder()
        .font_size((18.0 / dpi_scale()) as u16)
        .color(Color::from_rgba(222, 185, 140, 255))
        .color_hovered(Color::from_rgba(138, 101, 56, 255))
        .color_clicked(Color::from_rgba(112, 77, 35, 255))
        .build();

    let group_style = root_ui()
        .style_builder()
        .color(Color::new(0., 0., 0., 0.))
        .build();

    Skin {
        window_style,
        button_style,
        label_style,
        editbox_style,
        checkbox_style,
        group_style,
        ..root_ui().default_skin()
    }
}

pub fn render_ui(
    editing_settings: &mut Settings,
    settings_state: &mut SettingsState,
    box_size: (f32, f32),
) -> bool {
    if *settings_state == SettingsState::Closed {
        return false;
    }

    let mut save = false;

    let menu_position = (vec2(box_size.0, box_size.1) - MENU_SIZE) / 2.;

    root_ui().window(
        hash!(),
        menu_position / dpi_scale(),
        MENU_SIZE / dpi_scale(),
        |ui| {
            if let SettingsState::Settings(settings_page) = settings_state.clone() {
                let mut top_position = vec2(
                    (MENU_SIZE.x - BUTTON_SIZE.x) / 2.,
                    MENU_PADDING + BUTTONS_MARGIN,
                );

                if settings_page > 0 {
                    if widgets::Button::new("Prev")
                        .position(
                            vec2(
                                top_position.x + BUTTON_SIZE.x / 2.
                                    - BUTTON_SIZE.x / SMALLER_BUTTON_DIV
                                    - BUTTONS_MARGIN / 2.,
                                top_position.y,
                            ) / dpi_scale(),
                        )
                        .size(BUTTON_SIZE / SMALLER_BUTTON_DIV / dpi_scale())
                        .ui(ui)
                    {
                        *settings_state = SettingsState::Settings(settings_page - 1);
                    }
                }

                if settings_page < LAST_PAGE_INDEX {
                    if widgets::Button::new("Next")
                        .position(
                            vec2(
                                top_position.x + BUTTON_SIZE.x / 2. + BUTTONS_MARGIN / 2.,
                                top_position.y,
                            ) / dpi_scale(),
                        )
                        .size(BUTTON_SIZE / SMALLER_BUTTON_DIV / dpi_scale())
                        .ui(ui)
                    {
                        *settings_state = SettingsState::Settings(settings_page + 1);
                    }
                }

                const GROUP_OFFSET: Vec2 = vec2(50., 30.);

                let group = widgets::Group::new(
                    hash!(),
                    (MENU_SIZE - GROUP_OFFSET + vec2(40., -BUTTON_SIZE.y / SMALLER_BUTTON_DIV))
                        / dpi_scale(),
                )
                .position(
                    (GROUP_OFFSET + vec2(0., BUTTON_SIZE.y / SMALLER_BUTTON_DIV)) / dpi_scale(),
                )
                .begin(ui);

                match settings_page {
                    0 => {
                        widgets::Label::new("Audio volume").ui(ui);

                        widgets::Slider::new(hash!(), 0.0..1.0)
                            .ui(ui, &mut editing_settings.audio_volume);

                        widgets::Label::new("Gravity strength").ui(ui);

                        widgets::Slider::new(hash!(), -30.0..30.0)
                            .ui(ui, &mut editing_settings.gravity_strength);

                        widgets::Label::new("Air friction").ui(ui);

                        widgets::Slider::new(hash!(), 0.0..1.00)
                            .ui(ui, &mut editing_settings.air_friction);

                        widgets::Label::new("Terminal Velocity").ui(ui);

                        widgets::Slider::new(hash!(), 0.0..500.00)
                            .ui(ui, &mut editing_settings.terminal_velocity);
                    }
                    1 => {
                        widgets::Label::new("Ball bounciness").ui(ui);

                        widgets::Slider::new(hash!(), 0.0..1.0)
                            .ui(ui, &mut editing_settings.ball_bounciness);

                        widgets::Label::new("Ball radius").ui(ui);

                        widgets::Slider::new(hash!(), 0.0..(box_size.0.min(box_size.1) - 50.))
                            .ui(ui, &mut editing_settings.ball_radius);

                        widgets::Label::new("Ball weight").ui(ui);

                        widgets::Slider::new(hash!(), 0.0..1.0)
                            .ui(ui, &mut editing_settings.ball_weight);

                        widgets::Label::new("Ball friction").ui(ui);

                        widgets::Slider::new(hash!(), 0.0..1.0)
                            .ui(ui, &mut editing_settings.ball_friction);
                    }
                    _ => {
                        unimplemented!()
                    }
                }
                group.end(ui);

                top_position.y =
                    MENU_SIZE.y - BUTTON_SIZE.y / SMALL_BUTTON_DIV - MENU_PADDING - BUTTONS_MARGIN;

                if widgets::Button::new("Back")
                    .position(
                        vec2(
                            top_position.x + BUTTON_SIZE.x / 2.
                                - BUTTON_SIZE.x / SMALL_BUTTON_DIV
                                - BUTTONS_MARGIN / 2.,
                            top_position.y,
                        ) / dpi_scale(),
                    )
                    .size(BUTTON_SIZE / SMALL_BUTTON_DIV / dpi_scale())
                    .ui(ui)
                {
                    *settings_state = SettingsState::Open;
                }

                if widgets::Button::new("Apply")
                    .position(
                        vec2(
                            top_position.x + BUTTON_SIZE.x / 2. + BUTTONS_MARGIN / 2.,
                            top_position.y,
                        ) / dpi_scale(),
                    )
                    .size(BUTTON_SIZE / SMALL_BUTTON_DIV / dpi_scale())
                    .ui(ui)
                {
                    save = true;
                }
            } else {
                const MENU_PADDING: f32 = 45.;

                let mut button_position = vec2(
                    (MENU_SIZE.x - BUTTON_SIZE.x) / 2.,
                    MENU_PADDING + BUTTONS_MARGIN,
                );
                if widgets::Button::new("Continue")
                    .position(button_position / dpi_scale())
                    .size(BUTTON_SIZE / dpi_scale())
                    .ui(ui)
                {
                    *settings_state = SettingsState::Closed;
                }
                button_position.y += BUTTON_SIZE.y + BUTTONS_MARGIN;
                if widgets::Button::new("Options")
                    .position(button_position / dpi_scale())
                    .size(BUTTON_SIZE / dpi_scale())
                    .ui(ui)
                {
                    *settings_state = SettingsState::Settings(0);
                }
                button_position.y += BUTTON_SIZE.y + BUTTONS_MARGIN;
                if widgets::Button::new("Quit")
                    .position(button_position / dpi_scale())
                    .size(BUTTON_SIZE / dpi_scale())
                    .ui(ui)
                {
                    order_quit();
                }
            }
        },
    );
    return save;
}

pub struct UiAssets {
    menu_background: Texture2D,
    button: Texture2D,
    font: Font,
}

impl UiAssets {
    pub async fn new() -> Self {
        Self {
            menu_background: load_texture("./assets/main_background.png")
                .await
                .expect("Failed to load assets/main_background.png file"),

            button: load_texture("./assets/cardboard_button.png")
                .await
                .expect("Failed to load assets/cardboard_button.png file"),
            font: load_ttf_font("./assets/font.ttf")
                .await
                .expect("Failed to load assets/font.ttf file"),
        }
    }

    pub fn render_ui(
        &self,
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
                        mouse_pos,
                        vec2(0., start + lower_down * 0.),
                        vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                        "Audio volume",
                        TITLE_SIZE,
                        0.0..1.0,
                        &mut editing_settings.audio_volume,
                    );

                    self.render_slider(
                        mouse_pos,
                        vec2(0., start + lower_down * 1.),
                        vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                        "Gravity strength",
                        TITLE_SIZE,
                        -30.0..30.0,
                        &mut editing_settings.gravity_strength,
                    );

                    self.render_slider(
                        mouse_pos,
                        vec2(0., start + lower_down * 2.),
                        vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                        "Air friction",
                        TITLE_SIZE,
                        0.0..1.0,
                        &mut editing_settings.air_friction,
                    );

                    self.render_slider(
                        mouse_pos,
                        vec2(0., start + lower_down * 3.),
                        vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                        "Max velocity",
                        TITLE_SIZE,
                        0.0..500.0,
                        &mut editing_settings.terminal_velocity,
                    );
                }
                1 => {
                    self.render_slider(
                        mouse_pos,
                        vec2(0., start + lower_down * 0.),
                        vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                        "Ball bounciness",
                        TITLE_SIZE,
                        0.0..1.0,
                        &mut editing_settings.ball_bounciness,
                    );

                    self.render_slider(
                        mouse_pos,
                        vec2(0., start + lower_down * 1.),
                        vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                        "Ball radius",
                        TITLE_SIZE,
                        1.0..400.0,
                        &mut editing_settings.ball_radius,
                    );

                    self.render_slider(
                        mouse_pos,
                        vec2(0., start + lower_down * 2.),
                        vec2(SLIDER_WIDTH, SLIDER_HEIGHT),
                        "Ball weight",
                        TITLE_SIZE,
                        0.0..1.0,
                        &mut editing_settings.ball_weight,
                    );

                    self.render_slider(
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
                mouse_pos,
                vec2(center_offset_x, -y_offset),
                BUTTON_SIZE / SMALL_BUTTON_DIV,
                "Back",
                28,
            ) {
                *settings_state = SettingsState::Open;
            }

            if self.render_button(
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
                mouse_pos,
                vec2(0., -button_y_offsets),
                BUTTON_SIZE,
                "Continue",
                28,
            ) {
                *settings_state = SettingsState::Closed;
            }

            if self.render_button(mouse_pos, vec2(0., 0.), BUTTON_SIZE, "Settings", 28) {
                *settings_state = SettingsState::Settings(0);
            }

            if self.render_button(
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
        &self,
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
        let mouse_is_down = is_mouse_button_down(MouseButton::Left) || mouse_is_released;

        let color = if contains_mouse && mouse_is_down {
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

        return contains_mouse && mouse_is_released;
    }

    pub fn render_slider(
        &self,
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
        let mouse_is_released = is_mouse_button_released(MouseButton::Left);
        let mouse_is_down = is_mouse_button_down(MouseButton::Left) || mouse_is_released;

        let bar_width_pct = 0.15;
        let bar_width = slider_rect.w * bar_width_pct;

        if contains_mouse && mouse_is_down {
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
            slider_rect.y,
            slider_rect.w * bar_width_pct,
            slider_rect.h,
        );

        draw_rectangle(
            slider_rect.x,
            slider_rect.y,
            slider_rect.w,
            slider_rect.h,
            Color::from_rgba(112, 77, 35, 255),
        );

        draw_rectangle(
            bar_rect.x,
            bar_rect.y,
            bar_rect.w,
            bar_rect.h,
            Color::from_rgba(222, 185, 140, 255),
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
