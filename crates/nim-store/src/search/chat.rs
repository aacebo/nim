#[derive(Debug, Default)]
pub struct ChatFilter {
    pub limit: Option<usize>,
    pub min_score: Option<f32>,
}
