use std::collections::HashMap;

use candle_transformers::models::distilbert;

use crate::{Error, models::ModelType};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub vocab_size: usize,
    pub dim: usize,
    pub n_layers: usize,
    pub n_heads: usize,
    pub hidden_dim: usize,
    pub activation: Activation,
    pub max_position_embeddings: usize,
    pub initializer_range: f64,
    pub pad_token_id: usize,
    #[serde(default)]
    pub model_type: ModelType,
    #[serde(default)]
    pub id2label: HashMap<String, String>,
}

impl Config {
    pub fn hidden_size(&self) -> usize {
        self.dim
    }

    pub fn vocab_size(mut self, vocab_size: usize) -> Self {
        self.vocab_size = vocab_size;
        self
    }

    pub fn dim(mut self, dim: usize) -> Self {
        self.dim = dim;
        self
    }

    pub fn n_layers(mut self, n_layers: usize) -> Self {
        self.n_layers = n_layers;
        self
    }

    pub fn n_heads(mut self, n_heads: usize) -> Self {
        self.n_heads = n_heads;
        self
    }

    pub fn hidden_dim(mut self, hidden_dim: usize) -> Self {
        self.hidden_dim = hidden_dim;
        self
    }

    pub fn activation(mut self, activation: Activation) -> Self {
        self.activation = activation;
        self
    }

    pub fn max_position_embeddings(mut self, positions: usize) -> Self {
        self.max_position_embeddings = positions;
        self
    }

    pub fn labels(&self) -> Result<Vec<String>, Error> {
        let mut labels = vec![String::new(); self.id2label.len()];

        for (index, label) in &self.id2label {
            let index: usize = index.parse().map_err(crate::ParseError::from)?;
            let slot = labels
                .get_mut(index)
                .ok_or_else(|| Error::custom("invalid", format!("label index out of range: {index}")))?;

            *slot = label.clone();
        }

        Ok(labels)
    }

    pub fn to_candle(&self) -> Result<distilbert::Config, Error> {
        let json = serde_json::to_value(self)?;
        Ok(serde_json::from_value(json)?)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            vocab_size: 30_522,
            dim: 768,
            n_layers: 6,
            n_heads: 12,
            hidden_dim: 3_072,
            activation: Activation::Gelu,
            max_position_embeddings: 512,
            initializer_range: 0.02,
            pad_token_id: 0,
            model_type: ModelType::DistilBert,
            id2label: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Activation {
    Gelu,
    #[serde(rename = "gelu_approximate")]
    GeluApproximate,
    Relu,
}
