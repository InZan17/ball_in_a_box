use std::fs;

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
    assets_path: &str,
    pack_path: &Option<String>,
    missing_texture: &Texture2D,
) -> Texture2D {
    if let Some(pack_path) = pack_path {
        if let Ok(bytes) = fs::read(format!("{pack_path}/{asset_name}")) {
            return Texture2D::from_file_with_format(&bytes, None);
        }
    }

    if let Ok(bytes) = fs::read(format!("{assets_path}/{asset_name}")) {
        return Texture2D::from_file_with_format(&bytes, None);
    }

    missing_texture.clone()
}

pub fn load_assets_string(
    asset_name: &str,
    assets_path: &str,
    pack_path: &Option<String>,
    missing_string: &str,
) -> String {
    if let Some(pack_path) = pack_path {
        if let Ok(string) = fs::read_to_string(format!("{pack_path}/{asset_name}")) {
            return string;
        }
    }

    if let Ok(string) = fs::read_to_string(format!("{assets_path}/{asset_name}")) {
        return string;
    }

    missing_string.to_string()
}

pub fn load_assets_font(
    asset_name: &str,
    assets_path: &str,
    pack_path: &Option<String>,
) -> Option<Font> {
    if let Some(pack_path) = pack_path {
        if let Ok(bytes) = fs::read(format!("{pack_path}/{asset_name}")) {
            return Some(match load_ttf_font_from_bytes(&bytes) {
                Ok(font) => font,
                Err(err) => {
                    log_panic(&format!(
                        "Failed to load {pack_path}/{asset_name} font. {err}"
                    ));
                    unreachable!()
                }
            });
        }
    }

    if let Ok(bytes) = fs::read(format!("{assets_path}/{asset_name}")) {
        return Some(match load_ttf_font_from_bytes(&bytes) {
            Ok(font) => font,
            Err(err) => {
                log_panic(&format!(
                    "Failed to load {assets_path}/{asset_name} font. {err}"
                ));
                unreachable!()
            }
        });
    }

    None
}

impl GameAssets {
    pub fn new(pack: Option<&str>, missing_texture: Texture2D) -> Self {
        let assets_path = "./assets";
        let pack_path = pack.and_then(|pack| Some(format!("./texture_packs/{pack}")));
        Self {
            box_background_texture: load_texture(
                "box_background.png",
                assets_path,
                &pack_path,
                &missing_texture,
            ),
            box_side_texture: load_texture(
                "box_side.png",
                assets_path,
                &pack_path,
                &missing_texture,
            ),
            menu_background: load_texture(
                "menu_background.png",
                assets_path,
                &pack_path,
                &missing_texture,
            ),
            menu_button: load_texture("menu_button.png", assets_path, &pack_path, &missing_texture),
            slider_background: load_texture(
                "slider_background.png",
                assets_path,
                &pack_path,
                &missing_texture,
            ),
            slider_bar: load_texture("slider_bar.png", assets_path, &pack_path, &missing_texture),
            ball_material: match load_material(
                ShaderSource::Glsl {
                    vertex: VERTEX,
                    fragment: &load_assets_string("ball.frag", &assets_path, &pack_path, FRAGMENT),
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
                        assets_path,
                        &pack_path,
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
            font: load_assets_font("font.ttf", assets_path, &pack_path),
            missing_texture,
        }
    }
}
