use crate::widgets::ElementId;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Text {
    #[serde(default)]
    pub id: ElementId,
    #[serde(default)]
    pub style: TextStyle,
    pub text: String,
}

#[non_exhaustive]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextStyle {
    #[default]
    Plain,
    Markdown,
}
