use std::{fs, path::PathBuf, str::FromStr};

use macroquad::{prelude::*, texture::Texture2D};
use miniquad::{BlendFactor, BlendState, BlendValue, Equation};

use crate::log_panic;

const VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
attribute vec4 normal;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
gl_Position=Projection*Model*vec4(position, 1);
color=color0/255.0;
uv=texcoord;
}"#;

pub const FRAGMENT: &str = r#"#version 100
varying lowp vec4 color;
varying lowp vec2 uv;
uniform sampler2D Texture;
void main() {
gl_FragColor=color * texture2D(Texture, uv);
}"#;

pub struct GameAssets {
    pub missing_texture: Texture2D,
    pub box_background_texture: Texture2D,
    pub box_side_texture: Texture2D,
    pub menu_background: Texture2D,
    pub menu_button: Texture2D,
    pub slider_background: Texture2D,
    pub slider_bar: Texture2D,
    pub ball_material: Material,
    pub shadow_material: Material,
    pub font: Option<Font>,
}

pub fn load_texture(
    asset_name: &str,
    mut assets_path: PathBuf,
    pack_path: Option<PathBuf>,
    missing_texture: &Texture2D,
) -> Texture2D {
    if let Some(mut pack_path) = pack_path {
        pack_path.push(asset_name);
        if let Ok(bytes) = fs::read(pack_path) {
            return Texture2D::from_file_with_format(&bytes, None);
        }
    }
    assets_path.push(asset_name);
    if let Ok(bytes) = fs::read(assets_path) {
        return Texture2D::from_file_with_format(&bytes, None);
    }

    missing_texture.clone()
}

pub fn load_assets_string(
    asset_name: &str,
    mut assets_path: PathBuf,
    pack_path: Option<PathBuf>,
    missing_string: &str,
) -> String {
    if let Some(mut pack_path) = pack_path {
        pack_path.push(asset_name);
        if let Ok(string) = fs::read_to_string(pack_path) {
            return string;
        }
    }

    assets_path.push(asset_name);
    if let Ok(string) = fs::read_to_string(assets_path) {
        return string;
    }

    missing_string.to_string()
}

pub fn load_assets_font(
    asset_name: &str,
    mut assets_path: PathBuf,
    pack_path: Option<PathBuf>,
) -> Option<Font> {
    if let Some(mut pack_path) = pack_path {
        pack_path.push(asset_name);
        if let Ok(bytes) = fs::read(&pack_path) {
            return Some(match load_ttf_font_from_bytes(&bytes) {
                Ok(font) => font,
                Err(err) => {
                    log_panic(&format!(
                        "Failed to load {} font. {err}",
                        pack_path.to_string_lossy()
                    ));
                    unreachable!()
                }
            });
        }
    }

    assets_path.push(asset_name);
    if let Ok(bytes) = fs::read(&assets_path) {
        return Some(match load_ttf_font_from_bytes(&bytes) {
            Ok(font) => font,
            Err(err) => {
                log_panic(&format!(
                    "Failed to load {} font. {err}",
                    assets_path.to_string_lossy()
                ));
                unreachable!()
            }
        });
    }

    None
}

impl GameAssets {
    pub fn new(pack_path: Option<PathBuf>, missing_texture: Texture2D) -> Self {
        let Ok(assets_path) = PathBuf::from_str("./assets") else {
            log_panic("Failed to get assets path.");
            unreachable!()
        };
        Self {
            box_background_texture: load_texture(
                "box_background.png",
                assets_path.clone(),
                pack_path.clone(),
                &missing_texture,
            ),
            box_side_texture: load_texture(
                "box_side.png",
                assets_path.clone(),
                pack_path.clone(),
                &missing_texture,
            ),
            menu_background: load_texture(
                "menu_background.png",
                assets_path.clone(),
                pack_path.clone(),
                &missing_texture,
            ),
            menu_button: load_texture(
                "menu_button.png",
                assets_path.clone(),
                pack_path.clone(),
                &missing_texture,
            ),
            slider_background: load_texture(
                "slider_background.png",
                assets_path.clone(),
                pack_path.clone(),
                &missing_texture,
            ),
            slider_bar: load_texture(
                "slider_bar.png",
                assets_path.clone(),
                pack_path.clone(),
                &missing_texture,
            ),
            ball_material: match load_material(
                ShaderSource::Glsl {
                    vertex: VERTEX,
                    fragment: &load_assets_string(
                        "ball.frag",
                        assets_path.clone(),
                        pack_path.clone(),
                        FRAGMENT,
                    ),
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
                Ok(material) => material,
                Err(err) => {
                    log_panic(&format!("Failed to create ball material. {err}"));
                    unreachable!()
                }
            },
            shadow_material: match load_material(
                ShaderSource::Glsl {
                    vertex: VERTEX,
                    fragment: &&load_assets_string(
                        "shadow.frag",
                        assets_path.clone(),
                        pack_path.clone(),
                        FRAGMENT,
                    ),
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
                Ok(material) => material,
                Err(err) => {
                    log_panic(&format!("Failed to create shadow material. {err}"));
                    unreachable!()
                }
            },
            font: load_assets_font("font.ttf", assets_path, pack_path),
            missing_texture,
        }
    }
}

pub fn list_available_packs() -> Vec<(String, PathBuf)> {
    let Ok(read_dir) = fs::read_dir("./packs") else {
        return Vec::new();
    };

    read_dir
        .map(|entry| {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    log_panic(&format!(
                        "Failed to get DirEntry looking for available sounds. {err}"
                    ));
                    unreachable!()
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

pub fn find_pack(current_string: &str) -> Option<(String, PathBuf)> {
    if current_string.is_empty() {
        return None;
    }

    let mut selected_pack: Option<(String, PathBuf)> = None;

    for (pack_name, pack_path) in list_available_packs() {
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
