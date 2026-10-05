#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct Embedding(Vec<f32>);

impl From<Vec<f32>> for Embedding {
    fn from(value: Vec<f32>) -> Self {
        Self(value)
    }
}

impl<const N: usize> From<[f32; N]> for Embedding {
    fn from(value: [f32; N]) -> Self {
        Self(value.to_vec())
    }
}
