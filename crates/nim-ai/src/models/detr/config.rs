use std::collections::BTreeMap;

use crate::Error;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub backbone: String,
    pub dilation: bool,
    pub position_embedding_type: String,
    pub activation_function: String,
    pub d_model: usize,
    pub encoder_layers: usize,
    pub decoder_layers: usize,
    pub encoder_attention_heads: usize,
    pub decoder_attention_heads: usize,
    pub encoder_ffn_dim: usize,
    pub decoder_ffn_dim: usize,
    pub num_queries: usize,
    pub id2label: BTreeMap<usize, String>,
    #[serde(default = "default_channels")]
    pub num_channels: usize,
    #[serde(default = "default_timm")]
    pub use_timm_backbone: bool,
    #[serde(default)]
    pub backbone_config: Option<serde_json::Value>,
    #[serde(default)]
    pub backbone_kwargs: Option<serde_json::Value>,
    #[serde(default)]
    pub auxiliary_loss: bool,
}

fn default_channels() -> usize {
    3
}

fn default_timm() -> bool {
    true
}

impl Config {
    pub fn validate(&self) -> Result<(), Error> {
        if self.backbone != "resnet50"
            || self.dilation
            || !self.use_timm_backbone
            || self.num_channels != 3
            || self.backbone_config.is_some()
            || self
                .backbone_kwargs
                .as_ref()
                .is_some_and(|value| !value.is_null() && value.as_object().is_none_or(|map| !map.is_empty()))
            || self.position_embedding_type != "sine"
            || self.activation_function != "relu"
            || self.auxiliary_loss
        {
            return Err(Error::custom(
                "ai::config",
                "DETR supports the ResNet-50 backbone, sine positions, ReLU, and inference without auxiliary outputs",
            ));
        }
        if self.d_model == 0
            || !self.d_model.is_multiple_of(4)
            || self.encoder_attention_heads == 0
            || self.decoder_attention_heads == 0
            || !self.d_model.is_multiple_of(self.encoder_attention_heads)
            || !self.d_model.is_multiple_of(self.decoder_attention_heads)
            || self.encoder_layers == 0
            || self.decoder_layers == 0
            || self.encoder_ffn_dim == 0
            || self.decoder_ffn_dim == 0
            || self.num_queries == 0
            || self.id2label.is_empty()
            || self
                .id2label
                .last_key_value()
                .is_some_and(|(id, _)| id.checked_add(2).is_none())
        {
            return Err(Error::custom("ai::config", "invalid DETR dimensions or label map"));
        }
        Ok(())
    }

    pub fn num_classes(&self) -> usize {
        self.id2label.last_key_value().map_or(0, |(id, _)| id + 1)
    }
}
