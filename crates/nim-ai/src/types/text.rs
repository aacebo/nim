#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TextOutput {
    pub text: String,
    pub confidence: Option<f64>,
}
