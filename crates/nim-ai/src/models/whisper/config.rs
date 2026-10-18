#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    #[serde(flatten)]
    model: candle_transformers::models::whisper::Config,
    pub decoder_start_token_id: u32,
    pub eos_token_id: u32,
    pub begin_suppress_tokens: Vec<u32>,
    pub forced_decoder_ids: Vec<(usize, u32)>,
}

impl Config {
    pub fn model(&self) -> &candle_transformers::models::whisper::Config {
        &self.model
    }
}
