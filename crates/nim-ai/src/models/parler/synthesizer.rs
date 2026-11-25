use std::io::Cursor;
use std::sync::Mutex;

use candle_nn::VarBuilder;

use crate::{Context, Synthesize, SynthesizeOptions};

use super::config::ParlerConfig;
use super::model::ParlerModel;
use super::processor::ParlerProcessor;

pub struct ParlerSynthesizer {
    model: Mutex<ParlerModel>,
    processor: ParlerProcessor,
    config: ParlerGenerationConfig,
}

#[derive(Clone, Copy)]
struct ParlerGenerationConfig {
    sample_rate: u32,
}

impl ParlerSynthesizer {
    pub fn new(config: ParlerConfig, processor: ParlerProcessor, vars: VarBuilder<'static>) -> nim_error::Result<Self> {
        let decoding = ParlerGenerationConfig {
            sample_rate: config.model.audio_encoder.sampling_rate,
        };

        Ok(Self {
            model: Mutex::new(ParlerModel::new(&config, vars)?),
            processor,
            config: decoding,
        })
    }
}

impl Synthesize for ParlerSynthesizer {
    fn synthesize<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a str],
        options: SynthesizeOptions,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<u8>>> {
        Box::pin(async move {
            options.validate()?;
            let description = self.processor.description(&options);
            let spec = hound::WavSpec {
                channels: 1,
                sample_rate: self.config.sample_rate,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            };
            let mut output = Cursor::new(Vec::new());
            let mut writer = hound::WavWriter::new(&mut output, spec).map_err(nim_error::Diagnostic::message)?;

            for text in input {
                for chunk in self.processor.chunks(text) {
                    let (prompt, description) = self.processor.tensors(ctx, chunk, &description)?;
                    let samples = self
                        .model
                        .lock()
                        .map_err(|_| nim_error::Diagnostic::message("synthesis model lock poisoned"))?
                        .generate(&prompt, &description)?;

                    for sample in samples {
                        let sample = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                        writer.write_sample(sample).map_err(nim_error::Diagnostic::message)?;
                    }
                }
            }

            writer.finalize().map_err(nim_error::Diagnostic::message)?;
            Ok(output.into_inner())
        })
    }
}
