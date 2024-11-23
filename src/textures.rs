use std::{fs, path::PathBuf};

use macroquad::{rand, texture::Texture2D};

pub fn list_available_balls() -> Vec<(String, PathBuf)> {
    let read_dir = fs::read_dir("./balls").expect("Couldn't get the balls directory");

    read_dir
        .map(|entry| {
            let entry = entry
                .ok()
                .expect("Failed to get DirEntry looking for available balls");

            let path = entry.path();

            let filename = entry.file_name();

            let filename_str = filename.to_string_lossy();

            if !filename_str.ends_with(".png") {
                return None;
            }

            let filename_str = &filename_str[..filename_str.len() - 4];

            let filename_string = filename_str.to_string();

            Some((filename_string, path))
        })
        .flatten()
        .collect()
}

pub fn find_texture(current_string: &str) -> Option<(String, Texture2D)> {
    if current_string.is_empty() {
        return None;
    }

    for (ball_name, ball_path) in list_available_balls() {
        if current_string.ends_with(&ball_name) {
            let Ok(bytes) = fs::read(&ball_path) else {
                panic!("Failed to read bytes from {}", ball_path.to_string_lossy())
            };
            return Some((ball_name, Texture2D::from_file_with_format(&bytes, None)));
        }
    }

    None
}

pub fn get_random_texture() -> (String, Texture2D) {
    let available_balls = list_available_balls();

    if available_balls.is_empty() {
        panic!("available_balls is empty!");
    }

    let rand_index = rand::gen_range(0, available_balls.len());
    let (ball_name, ball_path) = unsafe {
        available_balls
            .into_iter()
            .nth(rand_index)
            .unwrap_unchecked()
    };

    let Ok(bytes) = fs::read(&ball_path) else {
        panic!("Failed to read bytes from {}", ball_path.to_string_lossy())
    };

    return (ball_name, Texture2D::from_file_with_format(&bytes, None));
}
