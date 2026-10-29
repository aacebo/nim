use crate::MemoryType;

#[derive(Debug, Default)]
pub struct MemoryFilter {
    pub ty: Option<MemoryType>,
    pub limit: Option<usize>,
    pub min_score: Option<f32>,
}
