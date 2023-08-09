use image::Rgba;
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use std::path::Path;
use image::io::Reader as ImageReader;

pub fn generate_cover(volume_title: String, output_path: String) {
    let mut img = ImageReader::open("src/assets/cover.png").unwrap().decode().unwrap();
    let out_path = Path::new(&output_path);

    let font = Vec::from(include_bytes!("font/RobotoSlab-VariableFont_wght.ttf") as &[u8]);
    let font = Font::try_from_vec(font).unwrap();

    draw_text_mut(&mut img, Rgba([255, 255, 60, 255]), 15,112, Scale::uniform(30.0), &font, &volume_title);
    let _ = img.save(out_path).unwrap();
}
