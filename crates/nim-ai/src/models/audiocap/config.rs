#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub sample_rate: usize,
    pub encoder_rnn_bidirectional: bool,
    pub encoder_rnn_hidden_size: usize,
    pub encoder_rnn_num_layers: usize,
    pub decoder_emb_dim: usize,
    pub vocab_size: usize,
    pub fc_emb_dim: usize,
    pub attn_emb_dim: usize,
    pub decoder_num_layers: usize,
    pub decoder_d_model: usize,
}
