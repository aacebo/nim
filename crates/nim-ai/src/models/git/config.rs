#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub vocab_size: usize,
    pub hidden_size: usize,
    pub num_hidden_layers: usize,
    pub num_attention_heads: usize,
    pub intermediate_size: usize,
    pub max_position_embeddings: usize,
    pub bos_token_id: u32,
    pub eos_token_id: u32,
    pub layer_norm_eps: f64,
    pub vision_config: VisionConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VisionConfig {
    pub hidden_size: usize,
    pub num_hidden_layers: usize,
    pub num_attention_heads: usize,
    pub intermediate_size: usize,
    pub image_size: usize,
    pub patch_size: usize,
    pub num_channels: usize,
    pub layer_norm_eps: f64,
}
