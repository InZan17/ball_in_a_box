use std::{fs, path::Path};

use macroquad::texture::Texture2D;

pub(crate) struct BallTextures {
    textures: Vec<(&'static str, &'static [u8])>,
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
                ("distress", include_bytes!("../assets/distress.png")),
                ("earth", include_bytes!("../assets/earth.png")),
                ("grinning", include_bytes!("../assets/grinning.png")),
                ("white", include_bytes!("../assets/white.png")),
            ],
        }
    }
    pub fn find_custom(&self, current_string: &str) -> Option<(String, Vec<u8>)> {
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
                    return Some((filename_str.to_string(), bytes));
                }
            }
        }
        None
    }

    pub fn find(&self, current_string: &str) -> Option<(&'static str, &'static [u8])> {
        if current_string.is_empty() {
            return None;
        }

        for (name, ball) in self.textures.iter() {
            if current_string.ends_with(name) {
                return Some((name, ball));
            }
        }
        None
    }

    pub fn get_first(&self) -> (&'static str, &'static [u8]) {
        let ball = &self.textures[0];
        (ball.0, ball.1)
    }

    pub fn get_texture(&self, current_string: &str) -> Texture2D {
        if let Some(ball) = self.find_custom(current_string) {
            Texture2D::from_file_with_format(&ball.1, None)
        } else if let Some(ball) = self.find(current_string) {
            Texture2D::from_file_with_format(ball.1, None)
        } else {
            let ball = self.get_first();
            Texture2D::from_file_with_format(ball.1, None)
        }
    }
}
