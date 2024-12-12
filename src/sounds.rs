use macroquad::audio::{load_sound_from_bytes, Sound};

use std::{fs, path::PathBuf};

use macroquad::rand;

use crate::log_panic;

pub fn list_available_sounds() -> Vec<(String, PathBuf)> {
    let Ok(read_dir) = fs::read_dir("./sounds") else {
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

pub async fn load_sounds(path: PathBuf) -> Vec<Sound> {
    let lossy_path = path.to_string_lossy();
    let read_dir = match fs::read_dir(&path) {
        Ok(read_dir) => read_dir,
        Err(err) => {
            log_panic(&format!(
                "Failed to read directory: \"{lossy_path}\" when loading sounds. {err}"
            ));
            unreachable!()
        }
    };

    let sounds_bytes = read_dir
        .map(|entry| {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    log_panic(&format!(
                        "Failed to get DirEntry when loading sounds. {err}"
                    ));
                    unreachable!()
                }
            };

            let path = entry.path();

            if !path.is_file() {
                return None;
            }

            let is_ogg = entry
                .file_name()
                .to_string_lossy()
                .to_ascii_lowercase()
                .ends_with(".ogg");

            if !is_ogg {
                return None;
            }

            let bytes = match fs::read(&path) {
                Ok(bytes) => bytes,
                Err(err) => {
                    log_panic(&format!(
                        "Failed to read bytes from: \"{lossy_path}\" when loading sounds. {err}"
                    ));
                    unreachable!()
                }
            };

            Some(bytes)
        })
        .flatten()
        .collect::<Vec<Vec<u8>>>();

    let mut sounds = Vec::with_capacity(sounds_bytes.len());

    for bytes in sounds_bytes {
        let sound = match load_sound_from_bytes(&bytes).await {
            Ok(sound) => sound,
            Err(err) => {
                log_panic(&format!(
                    "Couldn't create a sound from bytes from one of the sounds in: \"{lossy_path}\". {err}"
                ));
                unreachable!()
            }
        };

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
        if current_string.ends_with(&sounds_name.to_ascii_lowercase()) {
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

pub async fn get_random_sounds() -> Option<(String, Vec<Sound>)> {
    let available_sounds = list_available_sounds();

    if available_sounds.is_empty() {
        return None;
    }

    let rand_index = rand::gen_range(0, available_sounds.len());
    let (sounds_name, sounds_path) = unsafe {
        available_sounds
            .into_iter()
            .nth(rand_index)
            .unwrap_unchecked()
    };

    return Some((sounds_name, load_sounds(sounds_path).await));
}
