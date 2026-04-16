use image::{DynamicImage, GenericImageView, ImageEncoder, ImageReader};
use std::path::Path;

pub const CROP_SIZE: u32 = 252;

pub struct LoadedImage {
    pub width: u32,
    pub height: u32,
    pub rgba_data: Vec<u8>,
}

pub fn load_image(path: &Path) -> Result<LoadedImage, String> {
    let img = ImageReader::open(path)
        .map_err(|e| format!("ファイルを開けません: {e}"))?
        .decode()
        .map_err(|e| format!("画像のデコードに失敗: {e}"))?;

    let (w, h) = img.dimensions();
    if w < CROP_SIZE || h < CROP_SIZE {
        return Err(format!(
            "画像が小さすぎます ({w}x{h})。{CROP_SIZE}x{CROP_SIZE} 以上必要です。"
        ));
    }

    let rgba = img.to_rgba8();
    let rgba_data = rgba.as_raw().clone();

    Ok(LoadedImage {
        width: w,
        height: h,
        rgba_data,
    })
}

pub fn crop_and_encode(img: &DynamicImage, x: u32, y: u32) -> Result<Vec<u8>, String> {
    let cropped = img.crop_imm(x, y, CROP_SIZE, CROP_SIZE);
    let rgba = cropped.to_rgba8();
    let (cw, ch) = rgba.dimensions();

    let mut buf = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut buf);
    encoder
        .write_image(rgba.as_raw(), cw, ch, image::ExtendedColorType::Rgba8)
        .map_err(|e| format!("PNG エンコードに失敗: {e}"))?;
    Ok(buf)
}
