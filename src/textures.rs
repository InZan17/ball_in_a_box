use std::{fs, path::PathBuf};

use macroquad::{rand, texture::Texture2D};

use crate::error_log::ErrorLogs;

pub fn list_available_balls(error_logs: &mut ErrorLogs) -> Vec<(String, PathBuf)> {
    let read_dir = match fs::read_dir("./balls") {
        Ok(read_dir) => read_dir,
        Err(err) => {
            error_logs.display_error(format!("Failed to read the \"balls\" folder: {err}"));
            return Vec::new();
        }
    };

    read_dir
        .map(|entry| {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    error_logs.display_error(format!(
                        "Failed to get DirEntry looking for available balls. {err}"
                    ));
                    return None;
                }
            };

            let path = entry.path();

            let filename = entry.file_name();

            let filename_str = filename.to_string_lossy();

            if !filename_str.to_ascii_lowercase().ends_with(".png") {
                error_logs.display_error(
                    "Image with unsupported format found. Please use PNG.".to_string(),
                );
                return None;
            }

            let filename_str = &filename_str[..filename_str.len() - 4];

            let filename_string = filename_str.to_string();

            Some((filename_string, path))
        })
        .flatten()
        .collect()
}

/// Returns info for a ball texture in which the input ends with its name.
///
/// Picks the texture with the longer name.
pub fn find_texture(
    current_string: &str,
    error_logs: &mut ErrorLogs,
) -> Option<(String, Texture2D)> {
    if current_string.is_empty() {
        return None;
    }

    let mut selected_ball: Option<(String, PathBuf)> = None;

    for (ball_name, ball_path) in list_available_balls(error_logs) {
        if current_string.ends_with(&ball_name.to_ascii_lowercase()) {
            if let Some((selected_ball_name, _)) = &selected_ball {
                if selected_ball_name.len() > ball_name.len() {
                    continue;
                }
            }
            selected_ball = Some((ball_name, ball_path));
        }
    }

    let (ball_name, ball_path) = selected_ball?;

    let bytes = match fs::read(&ball_path) {
        Ok(bytes) => bytes,
        Err(err) => {
            error_logs.display_error(format!(
                "Failed to read texture bytes from \"{}\": {err}",
                ball_path.to_string_lossy()
            ));
            return None;
        }
    };

    let ball_texture = match Texture2D::from_file_with_format(&bytes, None) {
        Ok(texture) => texture,
        Err(err) => {
            error_logs.display_error(format!(
                "Failed to read texture data from \"{}\": {err}",
                ball_path.to_string_lossy()
            ));
            return None;
        }
    };

    return Some((ball_name, ball_texture));
}

pub fn get_random_texture(error_logs: &mut ErrorLogs) -> Option<(String, Texture2D)> {
    let available_balls = list_available_balls(error_logs);

    if available_balls.is_empty() {
        return None;
    }

    let rand_index = rand::gen_range(0, available_balls.len());
    let (ball_name, ball_path) = unsafe {
        available_balls
            .into_iter()
            .nth(rand_index)
            .unwrap_unchecked()
    };

    let bytes = match fs::read(&ball_path) {
        Ok(bytes) => bytes,
        Err(err) => {
            error_logs.display_error(format!(
                "Failed to read texture bytes from \"{}\": {err}",
                ball_path.to_string_lossy()
            ));
            return None;
        }
    };

    let ball_texture = match Texture2D::from_file_with_format(&bytes, None) {
        Ok(texture) => texture,
        Err(err) => {
            error_logs.display_error(format!(
                "Failed to read texture data from \"{}\": {err}",
                ball_path.to_string_lossy()
            ));
            return None;
        }
    };

    return Some((ball_name, ball_texture));
}
