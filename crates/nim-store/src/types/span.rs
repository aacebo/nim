#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum Span {
    Text { start: u32, end: u32 },
    Image { xmin: f32, ymin: f32, xmax: f32, ymax: f32 },
    Audio { start: f64, end: f64 },
}
