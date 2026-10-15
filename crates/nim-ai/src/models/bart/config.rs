use std::collections::HashMap;

use crate::{Error, ParseError};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub vocab_size: usize,
    pub d_model: usize,
    pub encoder_layers: usize,
    pub decoder_layers: usize,
    pub encoder_attention_heads: usize,
    pub decoder_attention_heads: usize,
    pub encoder_ffn_dim: usize,
    pub decoder_ffn_dim: usize,
    pub activation_function: candle_nn::Activation,
    pub max_position_embeddings: usize,
    pub eos_token_id: u32,
    pub decoder_start_token_id: u32,
    pub forced_bos_token_id: Option<u32>,
    pub forced_eos_token_id: Option<u32>,
    #[serde(default)]
    pub scale_embedding: bool,
    #[serde(default = "default_beams")]
    pub num_beams: usize,
    #[serde(default = "default_max_length")]
    pub max_length: usize,
    #[serde(default)]
    pub min_length: usize,
    #[serde(default = "default_length_penalty")]
    pub length_penalty: f64,
    #[serde(default)]
    pub no_repeat_ngram_size: usize,
    #[serde(default)]
    pub id2label: HashMap<String, String>,
}

impl Config {
    pub fn vocab_size(mut self, vocab_size: usize) -> Self {
        self.vocab_size = vocab_size;
        self
    }

    pub fn d_model(mut self, d_model: usize) -> Self {
        self.d_model = d_model;
        self
    }

    pub fn encoder_layers(mut self, layers: usize) -> Self {
        self.encoder_layers = layers;
        self
    }

    pub fn decoder_layers(mut self, layers: usize) -> Self {
        self.decoder_layers = layers;
        self
    }

    pub fn max_position_embeddings(mut self, positions: usize) -> Self {
        self.max_position_embeddings = positions;
        self
    }

    pub fn scale_embedding(mut self, scale: bool) -> Self {
        self.scale_embedding = scale;
        self
    }

    pub fn num_beams(mut self, beams: usize) -> Self {
        self.num_beams = beams;
        self
    }

    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = max_length;
        self
    }

    pub fn min_length(mut self, min_length: usize) -> Self {
        self.min_length = min_length;
        self
    }

    pub fn length_penalty(mut self, penalty: f64) -> Self {
        self.length_penalty = penalty;
        self
    }

    pub fn no_repeat_ngram_size(mut self, size: usize) -> Self {
        self.no_repeat_ngram_size = size;
        self
    }

    pub fn labels(&self) -> Result<Vec<String>, Error> {
        let mut labels = vec![String::new(); self.id2label.len()];

        for (index, label) in &self.id2label {
            let index: usize = index.parse().map_err(ParseError::from)?;
            let slot = labels
                .get_mut(index)
                .ok_or_else(|| Error::custom("invalid", format!("label index out of range: {index}")))?;

            *slot = label.clone();
        }

        Ok(labels)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            vocab_size: 50_265,
            d_model: 1_024,
            encoder_layers: 12,
            decoder_layers: 12,
            encoder_attention_heads: 16,
            decoder_attention_heads: 16,
            encoder_ffn_dim: 4_096,
            decoder_ffn_dim: 4_096,
            activation_function: candle_nn::Activation::Gelu,
            max_position_embeddings: 1_024,
            eos_token_id: 2,
            decoder_start_token_id: 2,
            forced_bos_token_id: None,
            forced_eos_token_id: Some(2),
            scale_embedding: false,
            num_beams: default_beams(),
            max_length: default_max_length(),
            min_length: 0,
            length_penalty: default_length_penalty(),
            no_repeat_ngram_size: 0,
            id2label: HashMap::new(),
        }
    }
}

fn default_beams() -> usize {
    4
}

fn default_max_length() -> usize {
    142
}

fn default_length_penalty() -> f64 {
    2.0
}
