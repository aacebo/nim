use candle_transformers::models::parler_tts::Config;

use crate::models::{Loader, ModelRef};

#[derive(Debug, Clone)]
pub struct ParlerConfig {
    pub model: Config,
    pub description_model: ModelRef,
}

impl ParlerConfig {
    pub async fn new(loader: &Loader) -> nim_error::Result<Self> {
        let mut value: serde_json::Value = loader.config().await?;
        let encoder = value
            .get_mut("audio_encoder")
            .and_then(serde_json::Value::as_object_mut)
            .ok_or_else(|| nim_error::Diagnostic::message("Parler configuration has no audio encoder"))?;
        let count = encoder.get("n_codebooks").cloned().unwrap_or(serde_json::Value::from(9));
        let hidden = encoder.get("hidden_size").cloned().unwrap_or(serde_json::Value::from(1024));
        encoder.entry("num_codebooks").or_insert(count);
        encoder.entry("latent_dim").or_insert(hidden);
        encoder.entry("model_bitrate").or_insert(serde_json::Value::from(0));
        encoder.entry("frame_rate").or_insert(serde_json::Value::from(0));
        let description_model = value
            .pointer("/text_encoder/_name_or_path")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| nim_error::Diagnostic::message("Parler configuration has no description tokenizer"))?;
        let description_model = description_model.parse()?;
        let model = serde_json::from_value(value)?;

        Ok(Self {
            model,
            description_model,
        })
    }
}
