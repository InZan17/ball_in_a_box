use macroquad::audio::{load_sound_from_bytes, Sound};

use std::{fs, path::PathBuf};

use macroquad::rand;

use crate::error_log::ErrorLogs;

pub fn list_available_sounds(error_logs: &mut ErrorLogs) -> Vec<(String, PathBuf)> {
    let read_dir = match fs::read_dir("./sounds") {
        Ok(read_dir) => read_dir,
        Err(err) => {
            error_logs.display_error(format!("Failed to read the \"sounds\" folder: {err}"));
            return Vec::new();
        }
    };

    read_dir
        .map(|entry| {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    error_logs.display_error(format!(
                        "Failed to get a DirEntry looking for available sounds. {err}"
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

pub async fn load_sounds(path: PathBuf, error_logs: &mut ErrorLogs) -> Vec<Sound> {
    let lossy_path = path.to_string_lossy();
    let read_dir = match fs::read_dir(&path) {
        Ok(read_dir) => read_dir,
        Err(err) => {
            error_logs.display_error(format!(
                "Failed to read directory: \"{lossy_path}\" when loading sounds. {err}"
            ));
            return Vec::new();
        }
    };

    let sounds_bytes = read_dir
        .map(|entry| {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    error_logs.display_error(format!(
                        "Failed to get DirEntry when loading sounds. {err}"
                    ));
                    return None;
                }
            };

            let path = entry.path();

            if !path.is_file() {
                return None;
            }

            let filename_lossy = entry.file_name().to_string_lossy().to_ascii_lowercase();

            if !filename_lossy.ends_with(".ogg") && !filename_lossy.ends_with(".wav") {
                error_logs.display_error(
                    "Unsupported audio format. Please use either OGG or WAV.".to_string(),
                );
                return None;
            }

            let bytes = match fs::read(&path) {
                Ok(bytes) => bytes,
                Err(err) => {
                    error_logs.display_error(format!(
                        "Failed to read sound bytes from: \"{lossy_path}\": {err}"
                    ));
                    return None;
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
                error_logs.display_error(format!(
                    "Failed to read sound data from one of the sounds in \"{lossy_path}\": {err}"
                ));
                continue;
            }
        };

        sounds.push(sound);
    }

    sounds
}

/// Returns info for a folder with sounds in which the input ends with the folders name.
///
/// Picks the folder with the longer name.
pub async fn find_sounds(
    current_string: &str,
    error_logs: &mut ErrorLogs,
) -> Option<(String, Vec<Sound>)> {
    if current_string.is_empty() {
        return None;
    }

    let mut selected_sounds: Option<(String, PathBuf)> = None;

    for (sounds_name, sounds_path) in list_available_sounds(error_logs) {
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

    return Some((sounds_name, load_sounds(sounds_path, error_logs).await));
}

pub async fn get_random_sounds(error_logs: &mut ErrorLogs) -> Option<(String, Vec<Sound>)> {
    let available_sounds = list_available_sounds(error_logs);

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

    return Some((sounds_name, load_sounds(sounds_path, error_logs).await));
}
