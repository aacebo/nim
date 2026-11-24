use crate::widgets::ElementId;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Link {
    #[serde(default)]
    pub id: ElementId,
    pub url: url::Url,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}
