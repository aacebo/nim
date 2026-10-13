use std::io::Cursor;

use candle_core::{Device, Tensor};
use image::{GenericImageView, ImageReader, imageops::FilterType};

use crate::Error;

#[derive(serde::Deserialize)]
pub struct Processor {
    size: Size,
    crop_size: Size,
    image_mean: [f32; 3],
    image_std: [f32; 3],
    rescale_factor: f32,
    resample: u8,
}

#[derive(serde::Deserialize)]
struct Size {
    height: Option<usize>,
    width: Option<usize>,
    shortest_edge: Option<usize>,
}

impl Processor {
    pub fn tensor(&self, bytes: &[u8], device: &Device) -> Result<Tensor, Error> {
        let image = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .map_err(|e| Error::custom("ai::image", e))?
            .decode()
            .map_err(|e| Error::custom("ai::image", e))?;
        let shortest = self
            .size
            .shortest_edge
            .ok_or_else(|| Error::custom("ai::image", "missing shortest_edge"))?;
        let (source_width, source_height) = image.dimensions();
        let scale = shortest as f64 / source_width.min(source_height) as f64;
        let image = image.resize_exact(
            (source_width as f64 * scale).round() as u32,
            (source_height as f64 * scale).round() as u32,
            filter(self.resample),
        );
        let width = self
            .crop_size
            .width
            .ok_or_else(|| Error::custom("ai::image", "missing crop width"))? as u32;
        let height = self
            .crop_size
            .height
            .ok_or_else(|| Error::custom("ai::image", "missing crop height"))? as u32;
        let image = image
            .crop_imm((image.width() - width) / 2, (image.height() - height) / 2, width, height)
            .to_rgb8();
        let mut values = vec![0f32; (width * height * 3) as usize];
        for (x, y, pixel) in image.enumerate_pixels() {
            let index = (y * width + x) as usize;
            for channel in 0..3 {
                values[channel * (width * height) as usize + index] =
                    (pixel[channel] as f32 * self.rescale_factor - self.image_mean[channel]) / self.image_std[channel];
            }
        }
        Ok(Tensor::from_vec(values, (1, 3, height as usize, width as usize), device)?)
    }
}

fn filter(resample: u8) -> FilterType {
    match resample {
        0 => FilterType::Nearest,
        2 => FilterType::Triangle,
        3 => FilterType::CatmullRom,
        _ => FilterType::Lanczos3,
    }
}
