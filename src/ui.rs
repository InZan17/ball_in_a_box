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
