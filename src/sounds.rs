use macroquad::audio::{load_sound_from_bytes, Sound};

use std::{fs, path::PathBuf};

use macroquad::rand;

pub fn list_available_sounds() -> Vec<(String, PathBuf)> {
    let read_dir = fs::read_dir("./sounds").expect("Couldn't get the sounds directory");

    read_dir
        .map(|entry| {
            let entry = entry
                .ok()
                .expect("Failed to get DirEntry looking for available sounds");

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

pub async fn load_sounds(path: PathBuf) -> Vec<Sound> {
    let lossy_path = path.to_string_lossy();
    let read_dir =
        fs::read_dir(&path).expect(&format!("Failed to read directory: \"{lossy_path}\"",));

    let sounds_bytes = read_dir
        .map(|entry| {
            let entry = entry.expect("Failed to get DirEntry when loading sounds");

            let path = entry.path();

            if !path.is_file() {
                return None;
            }

            let is_ogg = entry.file_name().to_string_lossy().ends_with(".ogg");

            if !is_ogg {
                return None;
            }

            let bytes = fs::read(&path).expect(&format!("Failed to read bytes from {lossy_path}"));

            Some(bytes)
        })
        .flatten()
        .collect::<Vec<Vec<u8>>>();

    let mut sounds = Vec::with_capacity(sounds_bytes.len());

    for bytes in sounds_bytes {
        let sound = load_sound_from_bytes(&bytes)
            .await
            .expect("Couldn't read bytes from a sound");

        sounds.push(sound);
    }

    sounds
}

/// Returns info for a folder with sounds in which the input ends with the folders name.
///
/// Picks the folder with the longer name.
pub async fn find_sounds(current_string: &str) -> Option<(String, Vec<Sound>)> {
    if current_string.is_empty() {
        return None;
    }

    let mut selected_sounds: Option<(String, PathBuf)> = None;

    for (sounds_name, sounds_path) in list_available_sounds() {
        if current_string.ends_with(&sounds_name) {
            if let Some((selected_sounds_name, _)) = &selected_sounds {
                if selected_sounds_name.len() > sounds_name.len() {
                    continue;
                }
            }
            selected_sounds = Some((sounds_name, sounds_path));
        }
    }

    let (sounds_name, sounds_path) = selected_sounds?;

    return Some((sounds_name, load_sounds(sounds_path).await));
}

pub async fn get_random_sounds() -> (String, Vec<Sound>) {
    let available_sounds = list_available_sounds();

    if available_sounds.is_empty() {
        panic!("There are no available sounds to use!");
    }

    let rand_index = rand::gen_range(0, available_sounds.len());
    let (sounds_name, sounds_path) = unsafe {
        available_sounds
            .into_iter()
            .nth(rand_index)
            .unwrap_unchecked()
    };

    return (sounds_name, load_sounds(sounds_path).await);
}
