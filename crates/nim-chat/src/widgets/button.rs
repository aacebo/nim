use crate::widgets::{Action, ElementId, Icon, IconPosition, Size};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Button {
    #[serde(default)]
    pub id: ElementId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<ButtonIcon>,
    #[serde(default)]
    pub size: Size,
    #[serde(default)]
    pub style: ButtonStyle,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<Action>,
}

#[non_exhaustive]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ButtonStyle {
    Primary,
    #[default]
    Secondary,
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum ButtonIcon {
    Name(Icon),
    Object { name: Icon, position: IconPosition },
}
