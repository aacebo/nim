use std::io::Cursor;

use candle_core::{Device, Tensor};

use crate::Error;

use super::Config;

const SAMPLE_RATE: u32 = 16_000;
const CHUNK_SAMPLES: usize = 30 * SAMPLE_RATE as usize;

#[derive(serde::Deserialize)]
pub struct Processor {
    mel_filters: Vec<Vec<f32>>,
}

impl Processor {
    pub fn tensors(&self, bytes: &[u8], config: &Config, device: &Device) -> Result<Vec<Tensor>, Error> {
        let samples = resample(decode(bytes)?)?;
        let filters: Vec<f32> = self.mel_filters.iter().flatten().copied().collect();

        samples
            .chunks(CHUNK_SAMPLES)
            .map(|samples| {
                let mel = candle_transformers::models::whisper::audio::pcm_to_mel(config.model(), samples, &filters);
                let source_frames = mel.len() / config.model().num_mel_bins;
                let frames = source_frames.min(candle_transformers::models::whisper::N_FRAMES);
                let mel = mel
                    .chunks(source_frames)
                    .flat_map(|row| row[..frames].iter().copied())
                    .collect();

                Tensor::from_vec(mel, (1, config.model().num_mel_bins, frames), device).map_err(Into::into)
            })
            .collect()
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
    let samples = samples
        .chunks_exact(channels)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect();

    Ok((samples, spec.sample_rate))
}

fn resample((samples, source_rate): (Vec<f32>, u32)) -> Result<Vec<f32>, Error> {
    if source_rate == 0 {
        return Err(Error::custom("ai::audio", "audio sample rate is zero"));
    }

    if source_rate == SAMPLE_RATE {
        return Ok(samples);
    }

    let length = samples.len() * SAMPLE_RATE as usize / source_rate as usize;

    Ok((0..length)
        .map(|index| {
            let position = index as f64 * source_rate as f64 / SAMPLE_RATE as f64;
            let before = position.floor() as usize;
            let after = (before + 1).min(samples.len() - 1);
            let amount = (position - before as f64) as f32;

            samples[before] * (1.0 - amount) + samples[after] * amount
        })
        .collect())
}
