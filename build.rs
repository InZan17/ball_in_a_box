// This code was taken from this project made by jumbledFox.
// https://github.com/jumbledFox/minesweeper/blob/master/build.rs

use std::{env, fs::File, io::Write, path::Path};

use image::{imageops::FilterType, ImageFormat};
use winresource::WindowsResource;

fn main() {
    let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR not set");
    let dest_path = Path::new(&out_dir).join("icon_data.rs");
    let ico_path = Path::new(&out_dir).join("icon.ico");
    let mut f = File::create(&dest_path).expect("Failed to create file");

    for (name, path, size) in [
        ("SMALL", "dev_assets/icon.png", 16),
        ("MEDIUM", "dev_assets/icon.png", 32),
        ("BIG", "dev_assets/icon.png", 64),
    ] {
        let img = image::open(path).expect("Failed to open image");
        let resized = img.resize(size, size, FilterType::Gaussian);
        let img_bytes = resized.as_bytes();
        write!(
            f,
            "pub const ICON_{}: [u8; {:?}] = {:?};",
            name,
            size * size * 4,
            img_bytes
        )
        .expect("Failed to write into image");

        if size == 64 {
            if env::var_os("CARGO_CFG_WINDOWS").is_some() {
                resized
                    .save_with_format(&ico_path, ImageFormat::Ico)
                    .expect("Failed to save icon.ico");
                WindowsResource::new()
                    .set_icon(
                        ico_path
                            .to_str()
                            .expect("Failed to convert icon path to a str"),
                    )
                    .compile()
                    .expect("Failed to compile WIndowsResource");
            }
        }
    }
}
