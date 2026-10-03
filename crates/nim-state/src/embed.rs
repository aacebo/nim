#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Embedding(Vec<f32>);
