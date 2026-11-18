/// A whole word: sub-word pieces already merged, carrying BYTE offsets into the source text.
#[derive(Debug, Clone)]
pub struct Word {
    pub label: String,
    pub score: f64,
    pub start: usize,
    pub end: usize,
}
