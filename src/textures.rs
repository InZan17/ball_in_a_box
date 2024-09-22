use std::{fs, path::Path};

use macroquad::texture::Texture2D;

pub(crate) struct BallTextures {
    textures: Vec<(&'static str, Texture2D)>,
}

impl BallTextures {
    pub fn new() -> Self {
        if !Path::new("./balls").exists() {
            if fs::create_dir("./balls").is_ok() {
                let _ = fs::write("./balls/custom.png", include_bytes!("../assets/custom.png"));
            }
        }

        Self {
            textures: vec![
                (
                    "distress",
                    Texture2D::from_file_with_format(
                        include_bytes!("../assets/distress.png"),
                        None,
                    ),
                ),
                (
                    "earth",
                    Texture2D::from_file_with_format(include_bytes!("../assets/earth.png"), None),
                ),
                (
                    "grinning",
                    Texture2D::from_file_with_format(
                        include_bytes!("../assets/grinning.png"),
                        None,
                    ),
                ),
                (
                    "white",
                    Texture2D::from_file_with_format(include_bytes!("../assets/white.png"), None),
                ),
            ],
        }
    }

    pub fn find(&self, current_string: &str) -> Option<(String, Texture2D)> {
        if current_string.is_empty() {
            return None;
        }

        if let Ok(read_dir) = fs::read_dir("./balls") {
            for entry in read_dir {
                let Ok(entry) = entry else {
                    continue;
                };
                let path = entry.path();
                let Some(filename) = path.file_name() else {
                    continue;
                };
                let Some(filename_str) = filename.to_str() else {
                    continue;
                };

                if !filename_str.ends_with(".png") {
                    continue;
                }

                let filename_str = &filename_str[..filename_str.len() - 4];

                if current_string.ends_with(filename_str) {
                    let Ok(bytes) = fs::read(&path) else {
                        break;
                    };
                    return Some((
                        filename_str.to_string(),
                        Texture2D::from_file_with_format(&bytes, None),
                    ));
                }
            }
        }

        for (name, ball) in self.textures.iter() {
            if current_string.ends_with(name) {
                return Some((name.to_string(), ball.clone()));
            }
        }
        None
    }

    pub fn get_first(&self) -> (String, Texture2D) {
        let ball = &self.textures[0];
        (ball.0.to_string(), ball.1.clone())
    }
}
