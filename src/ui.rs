use macroquad::{
    prelude::*,
    ui::{hash, root_ui, widgets},
};
use miniquad::*;
use window::{dpi_scale, order_quit};

use crate::{Settings, HEIGHT_F, WIDTH_F};

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

pub fn render_ui(editing_settings: &mut Settings, settings_state: &mut SettingsState) -> bool {
    if *settings_state == SettingsState::Closed {
        return false;
    }

    let mut save = false;

    let menu_position = (vec2(WIDTH_F, HEIGHT_F) / dpi_scale() - MENU_SIZE) / 2.;

    if let SettingsState::Settings(settings_page) = settings_state.clone() {
        root_ui().window(hash!(), menu_position, MENU_SIZE, |ui| {
            let mut top_position = vec2(
                (MENU_SIZE.x - BUTTON_SIZE.x) / 2.,
                MENU_PADDING + BUTTONS_MARGIN,
            );

            if settings_page > 0 {
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
                    *settings_state = SettingsState::Settings(settings_page - 1);
                }
            }

            if settings_page < LAST_PAGE_INDEX {
                if widgets::Button::new("Next")
                    .position(vec2(
                        top_position.x + BUTTON_SIZE.x / 2. + BUTTONS_MARGIN / 2.,
                        top_position.y,
                    ))
                    .size(BUTTON_SIZE / SMALLER_BUTTON_DIV)
                    .ui(ui)
                {
                    *settings_state = SettingsState::Settings(settings_page + 1);
                }
            }

            const GROUP_OFFSET: Vec2 = vec2(50., 30.);

            let group = widgets::Group::new(
                hash!(),
                MENU_SIZE - GROUP_OFFSET + vec2(40., -BUTTON_SIZE.y / SMALLER_BUTTON_DIV),
            )
            .position(GROUP_OFFSET + vec2(0., BUTTON_SIZE.y / SMALLER_BUTTON_DIV))
            .begin(ui);

            match settings_page {
                0 => {
                    widgets::Label::new("Audio volume").ui(ui);

                    widgets::Slider::new(hash!(), 0.0..1.0)
                        .ui(ui, &mut editing_settings.audio_volume);

                    widgets::Label::new("Bounciness").ui(ui);

                    widgets::Slider::new(hash!(), 0.0..1.0)
                        .ui(ui, &mut editing_settings.bounciness);

                    widgets::Label::new("Ball radius").ui(ui);

                    widgets::Slider::new(hash!(), 0.0..(WIDTH_F.min(HEIGHT_F) - 50.))
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

            top_position.y =
                MENU_SIZE.y - BUTTON_SIZE.y / SMALL_BUTTON_DIV - MENU_PADDING - BUTTONS_MARGIN;

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
                *settings_state = SettingsState::Open;
            }

            if widgets::Button::new("Apply")
                .position(vec2(
                    top_position.x + BUTTON_SIZE.x / 2. + BUTTONS_MARGIN / 2.,
                    top_position.y,
                ))
                .size(BUTTON_SIZE / SMALL_BUTTON_DIV)
                .ui(ui)
            {
                save = true;
            }
        });
    } else {
        const MENU_PADDING: f32 = 45.;
        root_ui().window(hash!(), menu_position, MENU_SIZE, |ui| {
            let mut button_position = vec2(
                (MENU_SIZE.x - BUTTON_SIZE.x) / 2.,
                MENU_PADDING + BUTTONS_MARGIN,
            );
            if widgets::Button::new("Continue")
                .position(button_position)
                .size(BUTTON_SIZE)
                .ui(ui)
            {
                *settings_state = SettingsState::Closed;
            }
            button_position.y += BUTTON_SIZE.y + BUTTONS_MARGIN;
            if widgets::Button::new("Options")
                .position(button_position)
                .size(BUTTON_SIZE)
                .ui(ui)
            {
                *settings_state = SettingsState::Settings(0);
            }
            button_position.y += BUTTON_SIZE.y + BUTTONS_MARGIN;
            if widgets::Button::new("Quit")
                .position(button_position)
                .size(BUTTON_SIZE)
                .ui(ui)
            {
                order_quit();
            }
        });
    };
    return save;
}
