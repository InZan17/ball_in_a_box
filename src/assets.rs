use std::{fs, io::ErrorKind, path::PathBuf};

use macroquad::{
    prelude::*,
    quad_gl::shader::{FRAGMENT, VERTEX},
    texture::Texture2D,
};
use miniquad::{BlendFactor, BlendState, BlendValue, Equation};

use crate::error_log::ErrorLogs;

pub struct GameAssets {
    pub missing_texture: Texture2D,
    pub box_background_texture: Texture2D,
    pub box_side_texture: Texture2D,
    pub menu_background: Texture2D,
    pub menu_button: Texture2D,
    pub slider_background: Texture2D,
    pub slider_bar: Texture2D,
    pub mouse_normal: Texture2D,
    pub mouse_normal_move: Texture2D,
    pub mouse_hold: Texture2D,
    pub mouse_hold_move: Texture2D,
    pub esc_normal: Texture2D,
    pub esc_hold: Texture2D,
    pub ball_material: Material,
    pub shadow_material: Material,
    pub font: Option<Font>,
}

pub fn load_texture(
    asset_name: &str,
    mut assets_path: PathBuf,
    pack_path: Option<PathBuf>,
    missing_texture: &Texture2D,
    error_logs: &mut ErrorLogs,
) -> Texture2D {
    if let Some(mut pack_path) = pack_path {
        pack_path.push(asset_name);
        if let Some(bytes) = match fs::read(&pack_path) {
            Ok(bytes) => Some(bytes),
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    None
                } else {
                    error_logs.display_error(format!(
                        "Failed to read texture bytes from \"{}\": {err}",
                        pack_path.to_string_lossy()
                    ));
                    return missing_texture.clone();
                }
            }
        } {
            return Texture2D::from_file_with_format(&bytes, None).unwrap_or_else(|err| {
                error_logs.display_error(format!(
                    "Failed to read texture data from \"{}\": {err}",
                    pack_path.to_string_lossy()
                ));
                missing_texture.clone()
            });
        }
    }
    assets_path.push(asset_name);
    match fs::read(&assets_path) {
        Ok(bytes) => {
            return Texture2D::from_file_with_format(&bytes, None).unwrap_or_else(|err| {
                error_logs.display_error(format!(
                    "Failed to read texture data from \"{}\": {err}",
                    assets_path.to_string_lossy()
                ));
                missing_texture.clone()
            });
        }
        Err(err) => {
            error_logs.display_error(format!(
                "Failed to read texture bytes from \"{}\": {err}",
                assets_path.to_string_lossy()
            ));
            missing_texture.clone()
        }
    }
}

pub fn load_assets_string(
    asset_name: &str,
    mut assets_path: PathBuf,
    pack_path: Option<PathBuf>,
    error_logs: &mut ErrorLogs,
) -> Option<String> {
    if let Some(mut pack_path) = pack_path {
        pack_path.push(asset_name);
        if let Some(string) = match fs::read_to_string(&pack_path) {
            Ok(string) => Some(string),
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    None
                } else {
                    error_logs.display_error(format!(
                        "Failed to read string from \"{}\": {err}",
                        pack_path.to_string_lossy()
                    ));
                    return None;
                }
            }
        } {
            return Some(string);
        }
    }

    assets_path.push(asset_name);
    match fs::read_to_string(&assets_path) {
        Ok(string) => Some(string),
        Err(err) => {
            error_logs.display_error(format!(
                "Failed to read string from \"{}\": {err}",
                assets_path.to_string_lossy()
            ));
            return None;
        }
    }
}

pub fn load_assets_font(
    asset_name: &str,
    mut assets_path: PathBuf,
    pack_path: Option<PathBuf>,
    error_logs: &mut ErrorLogs,
) -> Option<Font> {
    if let Some(mut pack_path) = pack_path {
        pack_path.push(asset_name);
        if let Some(bytes) = match fs::read(&pack_path) {
            Ok(bytes) => Some(bytes),
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    None
                } else {
                    error_logs.display_error(format!(
                        "Failed to read font bytes from \"{}\": {err}",
                        pack_path.to_string_lossy()
                    ));
                    return None;
                }
            }
        } {
            return match load_ttf_font_from_bytes(&bytes) {
                Ok(font) => Some(font),
                Err(err) => {
                    error_logs.display_error(format!(
                        "Failed to read font data from \"{}\": {err}",
                        pack_path.to_string_lossy()
                    ));
                    None
                }
            };
        }
    }
    assets_path.push(asset_name);
    match fs::read(&assets_path) {
        Ok(bytes) => {
            return match load_ttf_font_from_bytes(&bytes) {
                Ok(font) => Some(font),
                Err(err) => {
                    error_logs.display_error(format!(
                        "Failed to read font data from \"{}\": {err}",
                        assets_path.to_string_lossy()
                    ));
                    None
                }
            };
        }
        Err(err) => {
            error_logs.display_error(format!(
                "Failed to read font bytes from \"{}\": {err}",
                assets_path.to_string_lossy()
            ));
            None
        }
    }
}

pub fn load_shadow_material(
    assets_path: PathBuf,
    pack_path: Option<PathBuf>,
    error_logs: &mut ErrorLogs,
) -> Material {
    if let Some(fragment) = load_assets_string("shadow.frag", assets_path, pack_path, error_logs) {
        match load_material(
            ShaderSource::Glsl {
                vertex: VERTEX,
                fragment: &fragment,
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
        ) {
            Ok(material) => return material,
            Err(err) => {
                error_logs.display_error(format!("Failed to create shadow material: {err}"));
            }
        };
    }

    match load_material(
        ShaderSource::Glsl {
            vertex: VERTEX,
            fragment: FRAGMENT,
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
    ) {
        Ok(material) => return material,
        Err(err) => {
            error_logs.panic_error(&format!("Failed to create shadow material: {err}"));
            unreachable!()
        }
    };
}

pub fn load_ball_material(
    assets_path: PathBuf,
    pack_path: Option<PathBuf>,
    error_logs: &mut ErrorLogs,
) -> Material {
    if let Some(fragment) = load_assets_string("ball.frag", assets_path, pack_path, error_logs) {
        match load_material(
            ShaderSource::Glsl {
                vertex: VERTEX,
                fragment: &fragment,
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
        ) {
            Ok(material) => return material,
            Err(err) => {
                error_logs.display_error(format!("Failed to create ball material: {err}"));
            }
        };
    }

    match load_material(
        ShaderSource::Glsl {
            vertex: VERTEX,
            fragment: FRAGMENT,
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
    ) {
        Ok(material) => return material,
        Err(err) => {
            error_logs.panic_error(&format!("Failed to create ball material: {err}"));
            unreachable!()
        }
    };
}

impl GameAssets {
    pub fn new(
        pack_path: Option<PathBuf>,
        missing_texture: Texture2D,
        error_logs: &mut ErrorLogs,
    ) -> Self {
        let assets_path = PathBuf::from("./assets");
        Self {
            box_background_texture: load_texture(
                "box_background.png",
                assets_path.clone(),
                pack_path.clone(),
                &missing_texture,
                error_logs,
            ),
            box_side_texture: load_texture(
                "box_side.png",
                assets_path.clone(),
                pack_path.clone(),
                &missing_texture,
                error_logs,
            ),
            menu_background: load_texture(
                "menu_background.png",
                assets_path.clone(),
                pack_path.clone(),
                &missing_texture,
                error_logs,
            ),
            menu_button: load_texture(
                "menu_button.png",
                assets_path.clone(),
                pack_path.clone(),
                &missing_texture,
                error_logs,
            ),
            slider_background: load_texture(
                "slider_background.png",
                assets_path.clone(),
                pack_path.clone(),
                &missing_texture,
                error_logs,
            ),
            slider_bar: load_texture(
                "slider_bar.png",
                assets_path.clone(),
                pack_path.clone(),
                &missing_texture,
                error_logs,
            ),
            mouse_normal: load_texture(
                "mouse_normal.png",
                assets_path.clone(),
                pack_path.clone(),
                &missing_texture,
                error_logs,
            ),
            mouse_normal_move: load_texture(
                "mouse_normal_move.png",
                assets_path.clone(),
                pack_path.clone(),
                &missing_texture,
                error_logs,
            ),
            mouse_hold: load_texture(
                "mouse_hold.png",
                assets_path.clone(),
                pack_path.clone(),
                &missing_texture,
                error_logs,
            ),
            mouse_hold_move: load_texture(
                "mouse_hold_move.png",
                assets_path.clone(),
                pack_path.clone(),
                &missing_texture,
                error_logs,
            ),
            esc_normal: load_texture(
                "esc_normal.png",
                assets_path.clone(),
                pack_path.clone(),
                &missing_texture,
                error_logs,
            ),
            esc_hold: load_texture(
                "esc_hold.png",
                assets_path.clone(),
                pack_path.clone(),
                &missing_texture,
                error_logs,
            ),
            ball_material: load_ball_material(assets_path.clone(), pack_path.clone(), error_logs),
            shadow_material: load_shadow_material(
                assets_path.clone(),
                pack_path.clone(),
                error_logs,
            ),
            font: load_assets_font("font.ttf", assets_path, pack_path, error_logs),
            missing_texture,
        }
    }
}

pub fn list_available_packs(error_logs: &mut ErrorLogs) -> Vec<(String, PathBuf)> {
    let read_dir = match fs::read_dir("./asset_packs") {
        Ok(read_dir) => read_dir,
        Err(err) => {
            error_logs.display_error(format!("Failed to read the \"asset_packs\" folder: {err}"));
            return Vec::new();
        }
    };

    read_dir
        .map(|entry| {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    error_logs.display_error(format!(
                        "Failed to get DirEntry looking for available sounds. {err}"
                    ));
                    return None;
                }
            };

            let path = entry.path();

            if !path.is_dir() {
                return None;
            }

            let filename = entry.file_name();

            let filename_string = filename.to_string_lossy().to_string();

            Some((filename_string, path))
        })
        .flatten()
        .collect()
}

pub fn find_pack(current_string: &str, error_logs: &mut ErrorLogs) -> Option<(String, PathBuf)> {
    if current_string.is_empty() {
        return None;
    }

    let mut selected_pack: Option<(String, PathBuf)> = None;

    for (pack_name, pack_path) in list_available_packs(error_logs) {
        if current_string.ends_with(&pack_name.to_ascii_lowercase()) {
            if let Some((selected_pack_name, _)) = &selected_pack {
                if selected_pack_name.len() > pack_name.len() {
                    continue;
                }
            }
            selected_pack = Some((pack_name, pack_path));
        }
    }

    return selected_pack;
}
