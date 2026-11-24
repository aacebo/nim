mod action;
mod button;
mod container;
mod form;
mod grid;
mod icon;
mod link;
mod media;
mod text;

pub use action::*;
pub use button::*;
pub use container::*;
pub use form::*;
pub use grid::*;
pub use icon::*;
pub use link::*;
pub use media::*;
pub use text::*;

#[non_exhaustive]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Size {
    Small,
    #[default]
    Medium,
    Large,
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Surface {
    Chat,
    Message,
    Dialog,
}

/// a alphanumeric identifier that must be unique of peer element ids,
/// auto generated when not provided.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ElementId(String);

impl ElementId {
    pub fn new(value: impl std::fmt::Display) -> Self {
        Self(value.to_string())
    }
}

impl Default for ElementId {
    fn default() -> Self {
        use rand::distr::SampleString;
        Self(rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 5))
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Widget {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(default)]
    pub padding: Size,

    #[serde(default)]
    pub margin: Size,

    pub body: Vec<Element>,

    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub state: std::collections::BTreeMap<String, serde_json::Value>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change: Option<Action>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub submit: Option<Action>,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Element {
    /// Containers
    Container(Container),
    Grid(Grid),

    /// Media
    Text(Text),
    Audio(Audio),
    Image(Image),

    /// Buttons
    Button(Button),
    Link(Link),

    /// Forms
    Form(Form),
    TextInput(TextInput),
    NumberInput(NumberInput),
    Select(Select),
    MultiSelect(MultiSelect),
}

impl Element {
    pub fn id(&self) -> &ElementId {
        match self {
            Self::Container(v) => &v.id,
            Self::Grid(v) => &v.id,
            Self::Text(v) => &v.id,
            Self::Audio(v) => &v.id,
            Self::Image(v) => &v.id,
            Self::Button(v) => &v.id,
            Self::Link(v) => &v.id,
            Self::Form(v) => &v.id,
            Self::TextInput(v) => &v.id,
            Self::NumberInput(v) => &v.id,
            Self::Select(v) => &v.id,
            Self::MultiSelect(v) => &v.id,
        }
    }
}
