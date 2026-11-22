#[derive(Debug, Clone, serde::Deserialize)]
pub struct Cnn8RnnConfig {
    pub classes_num: usize,
    pub sample_rate: usize,
}
