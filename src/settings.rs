use core::str;
use std::fs;

use macroquad::prelude::*;
use nanoserde::{DeJson, SerJson};

#[derive(Debug, DeJson)]
#[nserde(serialize_none_as_null)]
pub struct DeserializeSettings {
    audio_volume: Option<f32>,
    gravity_strength: Option<f32>,
    air_friction: Option<f32>,
    max_velocity: Option<f32>,
    ball_bounciness: Option<f32>,
    ball_radius: Option<f32>,
    ball_weight: Option<f32>,
    ball_friction: Option<f32>,
    box_width: Option<f32>,
    box_height: Option<f32>,
    box_thickness: Option<f32>,
    box_depth: Option<f32>,
    ambient_occlusion_focus: Option<f32>,
    ambient_occlusion_strength: Option<f32>,
    specular_focus: Option<f32>,
    specular_strength: Option<f32>,
    ambient_light: Option<f32>,
    shadow_size: Option<f32>,
    shadow_distance_strength: Option<f32>,
    shadow_strength: Option<f32>,
    last_ball: Option<String>,
    last_sounds: Option<String>,
}

impl DeserializeSettings {
    pub fn contains_none(&self) -> bool {
        self.audio_volume.is_none()
            || self.gravity_strength.is_none()
            || self.air_friction.is_none()
            || self.max_velocity.is_none()
            || self.ball_bounciness.is_none()
            || self.ball_radius.is_none()
            || self.ball_weight.is_none()
            || self.ball_friction.is_none()
            || self.box_width.is_none()
            || self.box_height.is_none()
            || self.box_thickness.is_none()
            || self.box_depth.is_none()
            || self.ambient_occlusion_focus.is_none()
            || self.ambient_occlusion_strength.is_none()
            || self.specular_focus.is_none()
            || self.ambient_light.is_none()
            || self.shadow_strength.is_none()
            || self.shadow_size.is_none()
            || self.shadow_distance_strength.is_none()
            || self.last_ball.is_none()
            || self.last_sounds.is_none()
    }

    pub fn to_settings(self) -> (Settings, bool) {
        let default_settings = Settings::default();
        let has_none = self.contains_none();
        let settings = Settings {
            audio_volume: self.audio_volume.unwrap_or(default_settings.audio_volume),
            gravity_strength: self
                .gravity_strength
                .unwrap_or(default_settings.gravity_strength),
            air_friction: self.air_friction.unwrap_or(default_settings.air_friction),
            max_velocity: self.max_velocity.unwrap_or(default_settings.max_velocity),
            ball_bounciness: self
                .ball_bounciness
                .unwrap_or(default_settings.ball_bounciness),
            ball_radius: self
                .ball_radius
                .and_then(|ball_radius| {
                    if ball_radius < 1. {
                        return None;
                    } else {
                        return Some(ball_radius as u32);
                    }
                })
                .unwrap_or(default_settings.ball_radius),
            ball_weight: self.ball_weight.unwrap_or(default_settings.ball_weight),
            ball_friction: self.ball_friction.unwrap_or(default_settings.ball_friction),
            box_width: self
                .box_width
                .and_then(|box_width| {
                    if box_width < 0. {
                        return None;
                    } else {
                        return Some(box_width as u32);
                    }
                })
                .unwrap_or(default_settings.box_width),
            box_height: self
                .box_height
                .and_then(|box_height| {
                    if box_height < 0. {
                        return None;
                    } else {
                        return Some(box_height as u32);
                    }
                })
                .unwrap_or(default_settings.box_height),
            box_thickness: self
                .box_thickness
                .and_then(|box_thickness| {
                    if box_thickness < 1. {
                        return None;
                    } else {
                        return Some(box_thickness as u32);
                    }
                })
                .unwrap_or(default_settings.box_thickness),
            box_depth: self
                .box_depth
                .and_then(|box_depth| {
                    if box_depth < 1. {
                        return None;
                    } else {
                        return Some(box_depth as u32);
                    }
                })
                .unwrap_or(default_settings.box_depth),
            ambient_occlusion_focus: self
                .ambient_occlusion_focus
                .unwrap_or(default_settings.ambient_occlusion_focus),
            ambient_occlusion_strength: self
                .ambient_occlusion_strength
                .unwrap_or(default_settings.ambient_occlusion_strength),
            specular_focus: self
                .specular_focus
                .unwrap_or(default_settings.specular_focus),
            specular_strength: self
                .specular_strength
                .unwrap_or(default_settings.specular_strength),
            ambient_light: self.ambient_light.unwrap_or(default_settings.ambient_light),
            shadow_strength: self
                .shadow_strength
                .unwrap_or(default_settings.shadow_strength),
            shadow_size: self.shadow_size.unwrap_or(default_settings.shadow_size),
            shadow_distance_strength: self
                .shadow_distance_strength
                .unwrap_or(default_settings.shadow_distance_strength),
            last_ball: self.last_ball.unwrap_or(default_settings.last_ball),
            last_sounds: self.last_sounds.unwrap_or(default_settings.last_sounds),
        };
        (settings, has_none)
    }
}

#[derive(Debug, SerJson, Clone)]
#[nserde(serialize_none_as_null)]
pub struct Settings {
    pub audio_volume: f32,
    pub gravity_strength: f32,
    pub air_friction: f32,
    pub max_velocity: f32,

    pub ball_bounciness: f32,
    pub ball_radius: u32,
    pub ball_weight: f32,
    pub ball_friction: f32,

    pub box_width: u32,
    pub box_height: u32,
    pub box_thickness: u32,
    pub box_depth: u32,

    pub ambient_occlusion_focus: f32,
    pub ambient_occlusion_strength: f32,
    pub specular_focus: f32,
    pub specular_strength: f32,

    pub ambient_light: f32,
    pub shadow_size: f32,
    pub shadow_distance_strength: f32,
    pub shadow_strength: f32,

    pub last_ball: String,
    pub last_sounds: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            audio_volume: 0.6,
            gravity_strength: 3.,
            air_friction: 0.17,
            max_velocity: 100.,

            ball_bounciness: 0.9,
            ball_radius: 90,
            ball_weight: 0.65,
            ball_friction: 0.75,

            box_width: 640,
            box_height: 480,
            box_thickness: 20,
            box_depth: 20,

            ambient_occlusion_focus: 1.1,
            ambient_occlusion_strength: 0.7,
            specular_focus: 32.0,
            specular_strength: 0.3,

            ambient_light: 0.5,
            shadow_size: 1.2,
            shadow_distance_strength: 0.55,
            shadow_strength: 1.1,

            last_ball: "grinning".to_string(),
            last_sounds: "thud".to_string(),
        }
    }
}

pub fn read_settings_file() -> Option<Settings> {
    let bytes = fs::read("./settings_in_a.json").ok()?;
    let string = str::from_utf8(&bytes).ok()?;
    let de_settings = DeserializeSettings::deserialize_json(string).ok()?;

    let (settings, is_incomplete) = de_settings.to_settings();

    if is_incomplete {
        write_settings_file(&settings);
    }

    return Some(settings);
}

pub fn write_settings_file(settings: &Settings) {
    let _ = fs::write("./settings_in_a.json", settings.serialize_json_pretty());
}