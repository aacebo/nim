use std::io::Cursor;

use rustfft::FftPlanner;
use rustfft::num_complex::Complex32;

use crate::Error;

use super::config::Config;

pub struct Processor {
    filters: Vec<Vec<f32>>,
    window: Vec<f32>,
}

impl Processor {
    pub fn new(filters: Vec<Vec<f32>>, window: Vec<f32>) -> Self {
        Self { filters, window }
    }

    pub fn features(&self, bytes: &[u8], config: &Config) -> Result<Vec<Vec<f32>>, Error> {
        let samples = resample(decode(bytes)?, config.sample_rate)?;
        let padding = self.window.len() / 2;
        let mut padded = Vec::with_capacity(samples.len() + padding * 2);

        for index in (1..=padding).rev() {
            padded.push(samples[index.min(samples.len() - 1)]);
        }
        padded.extend(samples);
        for index in 0..padding {
            padded.push(padded[padded.len() - 2 - index]);
        }

        let mut fft = FftPlanner::new();
        let fft = fft.plan_fft_forward(self.window.len());
        let mut frames = Vec::new();

        for offset in (0..=padded.len() - self.window.len()).step_by(config.sample_rate / 100) {
            let mut input: Vec<_> = self
                .window
                .iter()
                .enumerate()
                .map(|(index, window)| Complex32::new(padded[offset + index] * window, 0.0))
                .collect();
            fft.process(&mut input);
            let power: Vec<_> = input[..=self.window.len() / 2].iter().map(|value| value.norm_sqr()).collect();
            let mut mel = vec![0.0; self.filters[0].len()];

            for (bin, weights) in self.filters.iter().enumerate() {
                for (mel, weight) in mel.iter_mut().zip(weights) {
                    *mel += power[bin] * weight;
                }
            }
            frames.push(mel);
        }

        let maximum = frames.iter().flatten().copied().fold(f32::MIN, f32::max).max(1e-10).log10() * 10.0;

        for value in frames.iter_mut().flatten() {
            *value = (10.0 * value.max(1e-10).log10()).max(maximum - 80.0);
        }

        Ok(frames)
    }
}

fn decode(bytes: &[u8]) -> Result<(Vec<f32>, u32), Error> {
    let mut reader = hound::WavReader::new(Cursor::new(bytes)).map_err(|err| Error::custom("ai::audio", err))?;
    let spec = reader.spec();

    if spec.sample_format != hound::SampleFormat::Int || spec.bits_per_sample != 16 || !(1..=2).contains(&spec.channels) {
        return Err(Error::custom("ai::audio", "expected mono or stereo 16-bit PCM WAV"));
    }

    let samples: Vec<f32> = reader
        .samples::<i16>()
        .map(|sample| sample.map(|sample| sample as f32 / i16::MAX as f32))
        .collect::<Result<_, _>>()
        .map_err(|err| Error::custom("ai::audio", err))?;

    if samples.is_empty() {
        return Err(Error::custom("ai::audio", "audio is empty"));
    }

    let channels = spec.channels as usize;
    Ok((
        samples
            .chunks_exact(channels)
            .map(|frame| frame.iter().sum::<f32>() / channels as f32)
            .collect(),
        spec.sample_rate,
    ))
}

fn resample((samples, source): (Vec<f32>, u32), target: usize) -> Result<Vec<f32>, Error> {
    if source == 0 {
        return Err(Error::custom("ai::audio", "audio sample rate is zero"));
    }

    if source as usize == target {
        return Ok(samples);
    }

    let length = samples.len() * target / source as usize;
    Ok((0..length)
        .map(|index| {
            let position = index as f64 * source as f64 / target as f64;
            let before = position.floor() as usize;
            let after = (before + 1).min(samples.len() - 1);
            let amount = (position - before as f64) as f32;

            samples[before] * (1.0 - amount) + samples[after] * amount
        })
        .collect())
}
