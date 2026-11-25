use candle_core::{DType, Tensor};
use tokenizers::Tokenizer;

use crate::models::Loader;
use crate::{Context, SpeechRate, SynthesizeOptions};

use super::config::ParlerConfig;
use super::voice;

const MAX_CHARS: usize = 160;

pub struct ParlerProcessor {
    description_tokenizer: Tokenizer,
}

impl ParlerProcessor {
    pub async fn new(loader: &Loader, config: &ParlerConfig, dtype: DType) -> nim_error::Result<Self> {
        let description_tokenizer = config
            .description_model
            .local_or_err()?
            .loader(loader.device().clone(), dtype)?
            .tokenizer()
            .await?;

        Ok(Self { description_tokenizer })
    }

    pub fn tensors(&self, ctx: &Context<'_>, text: &str, description: &str) -> nim_error::Result<(Tensor, Tensor)> {
        let prompt = ctx.tokenizer()?.encode(text, true).map_err(nim_error::Diagnostic::message)?;
        let description = self
            .description_tokenizer
            .encode(description, true)
            .map_err(nim_error::Diagnostic::message)?;
        let prompt = Tensor::new(prompt.get_ids(), ctx.device())?.unsqueeze(0)?;
        let description = Tensor::new(description.get_ids(), ctx.device())?.unsqueeze(0)?;

        Ok((prompt, description))
    }

    pub fn description(&self, options: &SynthesizeOptions) -> String {
        let voice = options.voice.as_deref().unwrap_or(voice::JON);
        let rate = match options.speed.unwrap_or_default() {
            SpeechRate::Slow => "slow",
            SpeechRate::Normal => "moderate",
            SpeechRate::Fast => "fast",
        };
        let language = match options.language.as_deref() {
            None | Some("en" | "en-us" | "en-gb") => "English",
            Some("es") => "Spanish",
            Some("fr") => "French",
            Some("it") => "Italian",
            Some("pt") => "Portuguese",
            Some("de") => "German",
            Some("nl") => "Dutch",
            Some("pl") => "Polish",
            Some(language) => language,
        };

        format!(
            "{voice}'s voice speaks {language} with a {rate} speaking rate and moderate pitch. The recording is of very high quality, with a very clear voice and almost no background noise."
        )
    }

    pub fn chunks<'a>(&self, mut text: &'a str) -> Vec<&'a str> {
        let mut chunks = Vec::new();

        while !text.trim().is_empty() {
            text = text.trim_start();
            let end = text.char_indices().nth(MAX_CHARS).map_or(text.len(), |(index, _)| index);
            let prefix = &text[..end];
            let split = if end == text.len() {
                end
            } else {
                prefix
                    .char_indices()
                    .rev()
                    .find(|(_, character)| character.is_whitespace())
                    .map_or(end, |(index, _)| index)
            };
            let chunk = text[..split].trim();

            if !chunk.is_empty() {
                chunks.push(chunk);
            }

            text = &text[split..];
        }

        chunks
    }
}
