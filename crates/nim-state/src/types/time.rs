use std::ops::Bound;

#[derive(Debug, Default, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub struct TimeRange {
    pub start: Option<chrono::DateTime<chrono::Utc>>,
    pub end: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<TimeRange> for nim_store::TimeRange {
    fn from(value: TimeRange) -> Self {
        Self {
            start: value.start.map(Bound::Included).unwrap_or(Bound::Unbounded),
            end: value.end.map(Bound::Excluded).unwrap_or(Bound::Unbounded),
        }
    }
}
