#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub classes_num: usize,
    pub sample_rate: usize,
}
