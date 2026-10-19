use std::io::Cursor;

use candle_core::{Device, Tensor};
use image::{GenericImageView, ImageReader, imageops::FilterType};

use crate::Error;

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct Processor {
    pub size: Size,
    pub do_resize: bool,
    pub do_rescale: bool,
    pub do_normalize: bool,
    pub rescale_factor: f64,
    pub image_mean: [f32; 3],
    pub image_std: [f32; 3],
    pub resample: u8,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Size {
    pub shortest_edge: usize,
    pub longest_edge: usize,
}

impl Default for Processor {
    fn default() -> Self {
        Self {
            size: Size {
                shortest_edge: 800,
                longest_edge: 1333,
            },
            do_resize: true,
            do_rescale: true,
            do_normalize: true,
            rescale_factor: 1.0 / 255.0,
            image_mean: [0.485, 0.456, 0.406],
            image_std: [0.229, 0.224, 0.225],
            resample: 2,
        }
    }
}

impl Processor {
    pub fn tensor(&self, bytes: &[u8], device: &Device) -> Result<(Tensor, u32, u32), Error> {
        if !self.rescale_factor.is_finite()
            || self.image_mean.iter().any(|value| !value.is_finite())
            || self.image_std.iter().any(|value| !value.is_finite() || *value <= 0.0)
            || self.size.shortest_edge == 0
            || self.size.longest_edge < self.size.shortest_edge
            || self.size.longest_edge > u32::MAX as usize
        {
            return Err(Error::custom("ai::config", "invalid DETR image processor configuration"));
        }

        let filter = match self.resample {
            0 => FilterType::Nearest,
            1 => FilterType::Lanczos3,
            2 => FilterType::Triangle,
            3 => FilterType::CatmullRom,
            _ => return Err(Error::custom("ai::config", "unsupported DETR resampling filter")),
        };

        let image = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()?
            .decode()
            .map_err(|error| Error::custom("ai::image", error))?;
        let (source_width, source_height) = image.dimensions();

        if source_width == 0 || source_height == 0 {
            return Err(Error::custom("ai::image", "image dimensions must be positive"));
        }

        let image = image.to_rgb8();
        let image = if self.do_resize {
            let short = source_width.min(source_height) as f64;
            let long = source_width.max(source_height) as f64;
            let (new_short, new_long) = if long / short * self.size.shortest_edge as f64 > self.size.longest_edge as f64 {
                let raw_short = self.size.longest_edge as f64 * short / long;
                (raw_short.round_ties_even() as u32, (raw_short * long / short) as u32)
            } else {
                (
                    self.size.shortest_edge as u32,
                    (self.size.shortest_edge as f64 * long / short) as u32,
                )
            };

            let (width, height) = if source_width.min(source_height) == new_short {
                (source_width, source_height)
            } else if source_width < source_height {
                (new_short, new_long)
            } else {
                (new_long, new_short)
            };

            if width == 0 || height == 0 {
                return Err(Error::custom(
                    "ai::image",
                    "image aspect ratio produces an empty resized image",
                ));
            }

            if self.resample == 2 {
                bilinear(&image, width, height)
            } else {
                image::imageops::resize(&image, width, height, filter)
            }
        } else {
            image
        };

        let (width, height) = image.dimensions();
        let plane = width as usize * height as usize;
        let mut values = vec![0f32; plane * 3];

        for (index, pixel) in image.pixels().enumerate() {
            for channel in 0..3 {
                let value = if self.do_rescale {
                    (pixel[channel] as f64 * self.rescale_factor) as f32
                } else {
                    pixel[channel] as f32
                };

                values[channel * plane + index] = if self.do_normalize {
                    (value - self.image_mean[channel]) / self.image_std[channel]
                } else {
                    value
                };
            }
        }

        let tensor = Tensor::from_vec(values, (1, 3, height as usize, width as usize), device)?;
        Ok((tensor, source_width, source_height))
    }
}

fn coefficients(input: u32, output: u32) -> Vec<Vec<(u32, i32)>> {
    let scale = input as f64 / output as f64;
    let support = scale.max(1.0);

    (0..output)
        .map(|index| {
            let center = (index as f64 + 0.5) * scale;
            let start = ((center - support + 0.5) as i64).max(0) as u32;
            let end = ((center + support + 0.5) as u32).min(input);
            let weights: Vec<_> = (start..end)
                .map(|source| {
                    let distance = ((source as f64 - center + 0.5) / support).abs();
                    (source, (1.0 - distance).max(0.0))
                })
                .collect();
            let sum: f64 = weights.iter().map(|(_, weight)| weight).sum();

            weights
                .into_iter()
                .map(|(source, weight)| (source, (weight / sum * (1 << 22) as f64 + 0.5) as i32))
                .collect()
        })
        .collect()
}

fn bilinear(input: &image::RgbImage, width: u32, height: u32) -> image::RgbImage {
    let horizontal = coefficients(input.width(), width);
    let intermediate = image::RgbImage::from_fn(width, input.height(), |x, y| {
        let mut pixel = [0u8; 3];

        for (channel, value) in pixel.iter_mut().enumerate() {
            let sum = horizontal[x as usize].iter().fold(1i64 << 21, |sum, (source, weight)| {
                sum + input.get_pixel(*source, y)[channel] as i64 * *weight as i64
            });

            *value = (sum >> 22).clamp(0, 255) as u8;
        }

        image::Rgb(pixel)
    });

    let vertical = coefficients(input.height(), height);

    image::RgbImage::from_fn(width, height, |x, y| {
        let mut pixel = [0u8; 3];

        for (channel, value) in pixel.iter_mut().enumerate() {
            let sum = vertical[y as usize].iter().fold(1i64 << 21, |sum, (source, weight)| {
                sum + intermediate.get_pixel(x, *source)[channel] as i64 * *weight as i64
            });

            *value = (sum >> 22).clamp(0, 255) as u8;
        }

        image::Rgb(pixel)
    })
}
