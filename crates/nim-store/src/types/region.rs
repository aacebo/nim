#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Region {
    Vision,
    Audio,
    Memory,
    Emotion,
    Executive,
    Action,
}
