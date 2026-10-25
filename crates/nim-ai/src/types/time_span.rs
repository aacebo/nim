#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TimeSpan {
    pub begin: f64,
    pub end: f64,
}
